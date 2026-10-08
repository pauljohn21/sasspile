# Design: Bootstrap Coverage to 99% — 三层架构重构

## Context

当前 Bootstrap dist alignment 覆盖率 68.10%（tracing 证据：1,704 行缺失 / 5,342 参考行）。通过深度代码审计，发现覆盖率差距的 **根因** 不是 4 个独立的症状，而是底层架构违反 rxrust 原则和 Rust 所有权导致的 **系统性不稳定**。

> ⚠️ **rxRust 反模式审计发现**: 本方案必须同时消除以下已识别反模式，否则任何补丁都会被 GC 思维回潮：
>
> | 反模式 | 位置 | 风险 |
> |--------|------|------|
> | `Arc<Mutex<Vec>>` + `subscribe(push)` | eval/mod.rs, parser/mod.rs | 绕过 rxrust 管线, GC 模式 |
> | 手动 `Vec<Work>` queue + while pop | eval/mod.rs | 命令式模拟 flat_map/expand |
> | 命令式 for+match 模拟 scan | eval/mod.rs | 状态累积不在算子链内 |
> | `'static` 一切 + clone | 多处 | unsafe escape hatch |
> | `#[cfg(test)]` 在 src/ | serialize/mod.rs, builder.rs | 违反纯生产代码规则 |

## Goals

1. **架构合规**: src/ 零 `#[cfg(test)]`，零 subscribe-collect，clone 最小化
2. **rxrust 合规**: eval pipeline 使用真正的 `flat_map` + `scan` 算子链
3. **Bootstrap 覆盖率 ≥ 99%**: 覆盖 ≥ 5,289 行（缺失 ≤ 53 行）

## Decisions

### L1: Architecture Compliance

#### D-L1.1: 消灭 subscribe-collect → 使用 `.collect()` 算子

**决策**: 所有 `Arc<Mutex<Vec>>` + `.subscribe(lock.push)` 模式替换为 `.collect::<Vec<_>>()`。

**理由**: `collect()` 是 rxrust 提供的终止算子，类型安全、无 GC 中间状态、可作为算子链一环。

```rust
// ❌ 旧: eval_stream
let nodes_arc = Arc::new(std::sync::Mutex::new(Vec::<AstNode>::new()));
ast_stream.subscribe(move |node| { nref.lock().unwrap().push(node); });
let nodes = nodes_arc.lock().unwrap().clone();

// ✅ 新: eval_stream
let nodes: Vec<AstNode> = ast_stream.collect::<Vec<_>>().into_iter().next().unwrap_or_default();
```

**影响文件**: `src/eval/mod.rs`, `src/parser/mod.rs`

#### D-L1.2: 内联测试迁移

**决策**: `serialize/mod.rs` 的 `serialize_tests` mod → `tests/serialize_test.rs`；`builder.rs` 的 `builder_tests` mod → `tests/builder_test.rs`。

**理由**: 保持 src/ 为纯生产代码，符合 AGENTS.md 红线。

**影响文件**: `src/serialize/mod.rs`, `src/builder.rs`, `tests/serialize_test.rs` (新建), `tests/builder_test.rs` (新建)

#### D-L1.3: Work Queue clone 治理

**决策**: Work enum 持有 `AstNode`（move 语义），去掉 `Arc<EvalContext>` 的反复 clone，改为在每次循环迭代移动 ctx。

**理由**: Rust 所有权核心——move 而非 clone。child_scope 在每次循环内构造，自然 move 进 Work item。

**影响文件**: `src/eval/mod.rs`

### L2: rxrust Pipeline Compliance

#### D-L2.1: eval pipeline 使用 flat_map + scan 算子链

**决策**: 把 `expand_nodes_to_events()` + `apply_event()` 的命令式组合替换为真正的 rxrust 算子：

```
ast_stream
  │ .flat_map(emit_node_events)     // 1 节点 → N 事件
  │ .scan(initial_frames, fold_frame) // 累积 frame 栈
  │ .filter_map(emit_completed)      // frame close 时 emit CssStmt
```

**理由**: 
- `flat_map` 天然表达 1:N reactive expansion（节点 emit 子事件回同一流）
- `scan` 天然承载状态累积（frame 栈折叠）
- 类型擦除 `.box_it()` 返回 CssStream

**影响文件**: `src/eval/mod.rs`

#### D-L2.2: parse_stream 使用 collect 算子

**决策**: TokenStream → Vec 通过 `.collect()` 一站式完成，移除中间 Arc<Mutex>。

```rust
// ❌ 旧
let source = Arc::new(std::sync::Mutex::new(Vec::<Token>::new()));
collected.subscribe(move |toks| { *src.lock().unwrap() = toks; });

// ✅ 新
let tokens: Vec<Token> = token_stream.collect::<Vec<_>>().into_iter().next().unwrap_or_default();
```

**影响文件**: `src/parser/mod.rs`

### L3: Bootstrap-Specific Fixes

#### D-L3.1: 变量 null 过滤修正

**决策**: 当 declaration value 评估为 `Value::Null` 时，不再丢弃整行。fallback 策略：
- 若 property 是 CSS 自定义属性 (`--bs-*`)，保留声明但输出值 `unset`
- 若 property 是普通声明，保留并 emit warning

**理由**: Bootstrap 中大量 `--bs-*` 变量通过条件路径赋值，中间 null 不应传播到最终输出。当前 ~396 行 `--bs-*` 缺失中，90%+ 源于此过滤逻辑。

**影响文件**: `src/eval/mod.rs`

#### D-L3.2: 变量 Cascade Chain Walk

**决策**: 当 `VariableRef` 解析时，若 parent chain walk 未找到 binding，尝试 **延迟解析**——将值保留为 `VarRef` 节点而非立即 eval。最终在 CSS 序列化时，若有 binding 则替换，否则 fallback 到 `var(--name)`。

**理由**: Bootstrap 变量链深度 3-5 级（如 `--bs-dark-text-emphasis` → `$gray-700` → `#292b2c`），中间层可能在 `@each` child_scope 中定义但 parent chain 未正确 walk 到 root。

**影响文件**: `src/eval/expr.rs`, `src/runtime.rs`

#### D-L3.3: Utility API Selector 组合修复

**决策**: 确保 `@each` 内嵌套 `@include` 产生的 selector 在 mixin body 内以 **correct parent context** 组合。`combine_selectors` 需区分:
- mixin body 中的 `&` 引用 → 用调用者的 selector 替换
- mixin body 的非 `&` 子选择器 → descendant combinator

**理由**: `.navbar-expand-md .navbar-nav .nav-link` 等三层嵌套选择器需要在每次 @each 迭代中保持独立的 selector 上下文。

**影响文件**: `src/eval/expr.rs` (combine_selectors)

#### D-L3.4: Vendor Prefix 自动注入

**决策**: 在 eval 层新增 `auto_prefix` 属性映射，当 mixin/declaration 声明了需要前缀的属性时，自动生成 vendor prefix 变体。

**映射表**:
- `file-upload-button` → `-webkit-file-upload-button` (伪元素)
- `column-gap` → `-moz-column-gap`
- `object-fit` → `-o-object-fit`
- `mask-position` → `-webkit-mask-position`
- `transition` → `-webkit-transition`, `-moz-transition`
- `appearance` → `-webkit-appearance`, `-moz-appearance`

**理由**: Bootstrap dist 中 ~80 行 vendor prefix 声明缺失，非 eval 失败而是 prefixer 未实现。

**影响文件**: `src/eval/builtin.rs` 或新建 `src/eval/prefixer.rs`

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| flat_map + scan 算子链可能引入性能开销 | Bootstrap 编译是单次操作，rxrust work-stealing 可忽略 |
| 延迟变量解析可能引入循环引用 | 增加 recursion depth limit + cycle detection + tracing 报警 |
| null fallback 可能导致输出 `unset` | 仅在 CSS 自定义属性场景启用 unset，普通声明仍保留 null 过滤 |
| 算子重写期间可能临时降低覆盖率 | 分 L1 → L2 → L3 渐进提交，每层独立全量测试通过 |

## Migration Plan

**渐进式提交，每层独立可编译通过测试**:

1. L1.1 (订阅消灭) → `cargo test` 通过
2. L1.2 (测试迁移) → `cargo test` 通过（无测试缺失）
3. L1.3 (clone 治理) → `cargo test` 通过
4. L2.1 (flat_map+scan 管线) → `cargo test` 通过
5. L2.2 (parse collect) → `cargo test` 通过
6. L3.1-L3.4 (Bootstrap 修复) → 覆盖率渐进提升至 ≥ 99%

## Open Questions

1. `expand` 算子是否能保留 current work queue 的 LIFO 语义（深度优先）？
2. `scan` 算子的状态是否可以是非 `Clone` 的 Frame stack？
3. 是否需要引入新的 `Value::VarRef` 枚举变体用于延迟解析？
