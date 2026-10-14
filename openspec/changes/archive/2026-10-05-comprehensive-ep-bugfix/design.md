# Design

## Context

27 EP cases fail with ctx=sasspile-bug. Root causes fall into 6 categories (see proposal.md). Each category maps to a spec capability in this change. Fix requires changes across:

- `src/eval/mixin.rs` -- eval_at_root nesting (A)
- `src/css/serializer.rs` -- var() null handling (B), -moz- prefix (C), :not() wrapping (D)
- `src/eval/rule_builder.rs` -- property ordering (F)
- `src/eval/mod.rs` -- variable resolution (E)

## Goals / Non-Goals

**Goals:**
- 修复全部 27 个 sasspile-bug（A 类 4 + B 类 3 + C 类 3~4 + D 类 1 + E 类 1 + F 类 ~15）
- 建立可扩展的输出规范化层（CSS 序列化 hook 点）
- 不破坏 sass-spec 通过率（当前基线保持）

**Non-Goals:**
- 完全重写序列化器 — 局部修补可满足需求
- 引入 autoprefixer 类库 — 只需一次性手动补丁

## Decisions

### Decision 1: A 类 — BEM 中间层级保留

**Approach:** 扩展 has_rule_children 检测 — 当 outer_only wrapper 含多层嵌套 Rule 时，不仅 fall-through 到 eval_nodes，还需确保 eval_content 传播的 chain 是 immediate selector 而非完整路径（延续 Decision 3 of atroot-fix）。

**Rationale:** 4 个 A 类 bug 均为嵌套 `e()` 调用链中间层丢失。模式与 atroot-fix 的 collapse 场景同源——eval_content 用 chain 替代 current_selector。

### Decision 2: B 类 — var() null fallback

**Approach:** Sequence 阶段检测 `Value::String(s)` 中的 `var(--x,null)` 字面量——当 null 出现在 var() 参数中时，将 null 替换为对应 CSS variable（如果环境有定义）或空字符串 `""`。在 serializer 输出加 post-pass。

```rust
// 匹配模式: "var(--name,null)" 或 "var(--sx, null)"
// 替换为:    "var(--name, )" 或 "var(--sx, )"
```

**插入点:** 在 `Serializer::serialize_decl()` 输出前，对 value 字符串做 regex/模式替换。

### Decision 3: C 类 — appearance vendor prefix

**Approach:** Serializer 的属性顺序校验——当遇到 `appearance: none` 时，自动在前插入 `-moz-appearance: none`。

```rust
if prop == "appearance" && value == "none" {
    emit("-moz-appearance: none");
}
emit("appearance: none");
```

**插入点:** `Serializer::serialize_decl()` 的属性名匹配分支。

### Decision 4: D 类 — :not() wrapper

**Approach:** Selector 序列化时检测 `:not(` 内的多参数——统计逗号分隔的选择器列表数量，>=2 时自动包装为 `:is(...)`。

**插入点:** `Selector` 类型的 Debug/Display impl，在输出 `:not()` 时检查 inner selector。

### Decision 5: E 类 — button 变量值逻辑

**Approach:** 读取 button.scss 的 diff，定位 SCSS 中错误的 `$state` 或条件分支。通过 trace 追踪 LESS→SCSS 转换中变量引用的来源，找到条件判断错误的 eval_if 路径。

### Decision 6: F 类 — CSS 属性差异

**Approach:** 逐文件 diagnose——对 15 个 F 类文件分别建立 trace，定位：
- Property 缺失：查找被吞没的 Decl 节点（RuleBuilder::push 是否正确累积）
- Value 错误：错误的 mixin include 路径或变量查找
- Order 差异：serializer 中属性遍历 HashMap 的迭代顺序

**批处理:** F 类不是统一根因，应拆为多个 mini-fix，每个文件独立诊断修复。

## Risks

| Risk | Mitigation |
|------|-----------|
| var() regex 误匹配正常 content | 精确格式 `var(--x,null)` 不含空格变体 |
| -moz- 前缀重复注入（源头已有） | 检测前置属性是否已存在 |
| :not() wrapper 破坏单参数 case | 仅在多逗号参数时触发 |
| 属性顺序修复引入新 hashmap 顺序问题 | 使用 BTreeMap 或 IndexMap 替代 HashMap |

## Implementation Phases

1. **Phase 1 (快速 win):** B + C + D（序列化层小补丁，预计修复 7~8 个 bug）
2. **Phase 2 (BEM 扩展):** A 类 4 个文件（复用 atroot-fix 模式）
3. **Phase 3 (逐一消灭):** F 类 15 个文件（逐文件 diagnose + fix）
4. **Phase 4 (变量逻辑):** E 类 1 个文件（button）
