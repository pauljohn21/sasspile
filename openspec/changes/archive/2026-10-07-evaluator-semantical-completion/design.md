# Design

## Context

当前 `src/eval/mod.rs` 的 `@each` 实现（第 236-267 行）**已支持 Map 迭代**：
```rust
Value::Map(entries) => entries.into_iter()
    .map(|(k, v)| Value::List(vec![Value::String(k), v]))
    .collect(),
```
双变量模式（`@each $key, $value in $map`）已实现，map→pairs 转换正确。

**实际待诊断**：Bootstrap `$utilities` 三层嵌套 map 迭代失败，根因未明。可能原因：
1. 外层 `@each` 变量（`$key`, `$value`）在内层作用域不可见（`child_scope` 的 parent chain 可能未正确传递）
2. 中间层 map 结构需要 `map-get($value, "values")` 提取后才能迭代
3. 编译路径中某处 `unwrap()` / 类型不匹配导致迭代提前终止

**诊断先行**：实施前需运行 `RUST_LOG=trace cargo test --test bootstrap_test -- --nocapture` 或 `bootstrap_dist_check()` 定位真实失败点，再针对性修复。

同时 `@include media-breakpoint-up(md) { ... }` 调用链中，mixin body 的 CSS 事件被直接追加到 `Vec<EvalEvent>`，缺少 @media 包裹标记的插入逻辑（覆盖率损失 ~15%）。

## Goals / Non-Goals

**Goals:**
- 通过诊断定位嵌套 map 迭代真实根因并修复，生成 spacing/display/flex/sizing utility 类
- 修复响应式 breakpoint mixin 展开为 @media 包裹
- Bootstrap dist 覆盖率从 22% 提升到 60%+
- 新代码**强制函数式风格**：`into_iter().map().collect()` 替代 `for + push`，`flat_map` 替代手动 `Vec::insert`

**Non-Goals:**
- 全面重构为真实 rxrust 算子替换 expand/scan 模拟（后续变更）
- 修复 libsass 不支持的 CSS 特性（color space、CSS Color Level 4 等）
- 100% Bootstrap 覆盖率（当前阶段目标 60%+）

## RxRust 算子使用原则

本变更在现有 expand/scan 模拟框架内工作，新增逻辑遵循：
- 集合变换用 `into_iter().map().collect()`，不用 `for + Vec::push`
- 事件流包装用 `flat_map`，不用手动 `Vec::insert`
- 累积状态用 `try_fold`，不用 `&mut` 参数
- 消费语义优先：`(input) -> (output)` move 模式

## Decisions

### Decision 1: 嵌套 map 迭代修复策略

**方案**: 先诊断后修复。若确认是作用域链问题，在 `child_scope()` 实现中确保 parent chain 正确传递（当前 `ExpandContext` 应通过 `Rc<Scope>` 链查找变量，而非复制）。

**理由**:
- 现有 Map→Pairs 实现已正确，不需要重写
- 作用域链断裂是 Sass 嵌套结构中最常见的 trap
- 诊断先行避免错误假设

**替代方案**:
- 重写整个 `@each` 为递归展开 — 不必要，现有模式工作正常
- 预展平 `$utilities` map — 不可行，map 值是动态的

### Decision 2: @media 包裹生成策略

**方案**: 在 `@include` 展开 mixin body 时，检测 mixin 名称前缀 `media-breakpoint-`。匹配时，消费 body 事件流，通过 `flat_map` 或 `into_iter().flat_map` 包装：

```rust
// 函数式风格：消费旧流，生成新流
let wrapped: Vec<EvalEvent> = body_events.into_iter()
    .flat_map(|e| vec![EvalEvent::EnterMedia(query.clone()), e, EvalEvent::LeaveMedia])
    .collect();
```

**理由**:
- 复用现有 `EnterMedia` / `LeaveMedia` 事件机制
- `flat_map` 消费旧流生成新流 — 符合所有权语义
- 最小侵入：只需在 mixin 展开点检测名称前缀
- 不破坏现有 `@content` 替换逻辑

**替代方案**:
- 在 mixin 定义处标记"需要 @media 包裹" — 不够灵活
- 完全重构为 true @media scope — 推迟

### Decision 3: CSS 属性名插值支持

**方案**: 消费 property list（`Value::List` 或 `Value::String`），通过迭代器链 flat_map 展开为多条声明：

```rust
// 单 property: "margin-top" → 声明
// 多 property: ["margin-left", "margin-right"] → flat_map → 两条声明
let decls: Vec<EvalEvent> = properties.into_iter()
    .flat_map(|prop| {
        values.into_iter().map(move |(cls_suffix, val)| {
            EvalEvent::Terminal(CssStmt::Decl { property: prop.clone(), value: val })
        })
    })
    .collect();
```

**理由**:
- Bootstrap utility map 使用完整 CSS 属性名
- `flat_map` + `collect()` 替代命令式嵌套 for + push
- 消费语义：每次迭代消费 properties/values，避免 `&mut Vec` 累积

## Risks / Trade-offs

| 风险 | 影响 | 缓解 |
|------|------|------|
| 作用域链诊断耗时 | 中 | 用 trace span 插桩 + 4 步调试协议 |
| flat_map 所有权转移导致 double-iterate 编译错误 | 中 | collect 到 Vec 后再消费，或 clone 关键共享值 |
| 新代码风格与现有模拟框架不协调 | 低 | 局部函数式 + 整体命令式共存，渐进迁移 |
| 性能退化（大量 utility 类生成） | 低 | 当前管线性能可接受 |

## Implementation Notes

- **诊断阶段**：运行 `cargo test --test bootstrap_test` + trace 日志，定位三层嵌套 `@each` 真实失败点
- **修复阶段 1**：根据诊断结果修复作用域链或 map-get 提取
- **修复阶段 2**：@media 包裹使用 `flat_map` + `collect()` 消费生成新事件流
- **验证阶段**：覆盖率门控从 22% → 60%+
- **强制约束**：所有新增集合操作使用迭代器链，禁止 `for + push`
