# Design

## Context

`sasspile` 的 `@at-root` 处理在 `src/eval/mixin.rs` 的 `eval_at_root` 函数中。EP 的 BEM mixin（`when()`, `e()`, `m()`）依赖 `@at-root` 语义将元素/修饰符选择器提升到 root 级别而非嵌套声明。当前有两个相关 bug 源于链式上下文传播不正确。

**Bug 1 示例** (`dialog.scss`):
```scss
@include b(dialog) {
  @include when(draggable) {
    @include e(header) { cursor: move; }
  }
}
```
sasspile: `.el-dialog__header { cursor: move; }` (丢失 `.el-dialog.is-draggable`)
expected: `.el-dialog.is-draggable .el-dialog__header { cursor: move; }`

**Bug 2 示例** (`collapse-item.scss`):
```scss
@include e(header) {
  @include e(arrow) {
    @include when(active) { transform: rotate(90deg); }
  }
}
```
sasspile: `.el-collapse-item__header .el-collapse-item__arrow.is-active { ... }` (多余 `.el-collapse-item__header`)
expected: `.el-collapse-item__arrow.is-active { ... }`

## Goals / Non-Goals

**Goals:**
- 修复 `eval_at_root` 中 `outer_only` 对嵌套 Rule 的误 unwrap
- 修复 `eval_at_root` 进入时不重置 `selector_chain` 的 bug
- 修复 `eval_content` 用 `selector_chain` 替代 `current_selector` 导致嵌套 mixin 链中 at-root parent_sel 解析错误
- 消除所有 25+ 个 EP 一致性 bug（`when/e/m` 相关 DIFF）
- 不破坏 sass-spec 通过率（当前 ~8000/12133）

**Non-Goals:**
- 不改变 `@at-root` 在简单场景（纯声明块）的语义
- 不涉及 selector 解析逻辑（`selector_combine.rs`）修改

## Decisions

### Decision 1: outer_only 保护 — 检测到子 Rule 时 fall-through

**问题**: `outer_only=true` 时 unwrap wrapper Rule 并直接处理 `first_body` 子节点。当 `first_body` 包含 Rule nodes 时，unwrap 丢失了 wrapper 作为前缀的语义。

**方案**: 在 `outer_only` 分支前检测 `first_body` 是否包含 `Node::Rule` — 如果是，不进入 unwrap 分支，fall-through 到 `Self::eval_nodes(body, env)?`，让 eval_rule 正确处理嵌套结构。

```rust
let has_rule_children = first_body.iter().any(|n| matches!(n, Node::Rule { .. }));
if at_root_match || (outer_only && !has_rule_children) {
    // 原 unwrap 逻辑
} else {
    Self::eval_nodes(body, env)?
}
```

**Alternatives considered:**
- 在 unwrap 后通过 `nest_rule_in_children(parent_sel, child_css)` 重新加前缀 — 但 `child_css` 已经是最终 CssNode，不能再加前缀，且 `resolve_ampersand_in_nodes` 只处理 `&` 字面量
- 递归检测最多 N 层嵌套 — 过度复杂，fall-through 自然处理任意深度

### Decision 2: at-root 无条件 chain 重置

**问题**: `eval_at_root` 仅在 `selector=Some(_)` 时调用 `reset_selector`。当 `selector=None`（EP mixin 常见情况 — `@at-root { &.is-active { ... } }`），chain 被外层 scope 传递进来，导致 `&` 展开使用错误的完整后裔链。

**方案**: 无条件调用 `env.reset_selector(parent_sel.clone())`，无论是否有显式 selector 参数。

```rust
let env = env.reset_selector(parent_sel.clone());
```

**Alternatives considered:**
- 仅在 `selector=None` 时走不同路径保留 chain — 与 at-root 语义矛盾，at-root 就是脱离外层作用域链
- 选择性保留 chain 部分 — 不确定保留哪部分，过于复杂

**Rationale**: at-root 的语义是"脱离当前嵌套层级"。进入 at-root body 时，当前有效的选择器就是 `parent_sel`（env.get_selector()），chain 应该从它重新开始。

### Decision 3: eval_content 使用 immediate selector 而非 chain（实际实现中新增）

**问题**: `eval_content`（`src/eval/mod.rs`）将 `content_chain`（通过 `env.get_selector_chain().or_else(|| env.get_selector())` 获得的完整后代链）设成 `current_selector`。当 `@content` 在嵌套 Rule 内触发时（如 `e(arrow)` mixin body 里 `@at-root { .__arrow { @content } }`），chain 已被 `with_chain` 更新为 `.el-collapse-item__header .el-collapse-item__arrow`。后续 `@content` 执行 `when(active)` mixin，其 `@at-root` 的 `parent_sel = env.get_selector()` 被错误解析为完整 chain。

**方案**: 在 `eval_content` 中改用 `env.get_selector()`（immediate selector）设置 `current_selector`。`selector_chain`（完整后代链）保留供 `&` 展开使用（`combine_selectors` 从 chain 展开 `&`），但 `current_selector` 只存 immediate 层级。

```rust
.with_selector(
    env.get_selector().unwrap_or_default().to_string(),
)
.with_chain_opt(content_chain.as_deref());
```

**Alternatives considered:**
- 在 `eval_at_root` 入口处额外检查并截断 chain — 治标不治本，根因在 eval_content 错误设置 current_selector
- 修改 `eval_at_root` 用 `min(chain, sel)` 逻辑 — 无明确语义，immediate selector 就是正确答案

## Risks / Trade-offs

- **[Risk]**: 无条件 chain 重置可能影响现有不依赖 at-root 链传播的用例 → **Mitigation**: sass-spec 全量回归测试验证
- **[Risk]**: outer_only fall-through 改变了已有测试的输出格式（嵌套选择器 vs 扁平选择器） → **Mitigation**: sass-spec 6 个 EP 基线测试对比
- **[Risk]**: 修复后可能与 dart-sass 在某些 edge case 仍有差异 → **Mitigation**: EP 分类测试验证

## Open Questions

None — 设计决策已通过 trace 验证，场景清晰。
