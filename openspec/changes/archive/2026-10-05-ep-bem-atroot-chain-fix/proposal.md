# Proposal

## Why

EP (Element Plus) 一致性 37/121 sasspile-bug 中约 25 个源于 BEM 嵌套展开时 `@at-root` 链式上下文处理错误。根本原因有两个：(1) `outer_only` 优化在嵌套 Rule 场景错误 unwrap 导致前缀丢失；(2) `eval_at-root` 在 `selector=None` 时不重置 `selector_chain`，导致 `&` 展开使用错误的后裔链。这导致 `when()+e()` 组合修饰符展开规则系统性失败。

## What Changes

- **修复 1**: `eval_at-root` 中 `outer_only` 分支 — 当 `first_body` 包含子 Rule 时不 unwrap，fall-through 保留完整的嵌套结构
- **修复 2**: `eval_at-root` 中 chain 重置逻辑 — 无论 `@at-root` 是否有显式 selector 参数，都无条件将 `env.selector_chain` 重置为 `parent_sel`，确保内部 `&` 展开基于 immediate parent 而非外层后裔链

## Capabilities

### Modified Capabilities

- `scss-eval/at-root-nesting`: 修正 `@at-root` 嵌套规则展开语义 — `outer_only` unwrap 保护和 chain 重置

## Impact

- `src/eval/mixin.rs` — `eval_at_root` 函数逻辑修改
- 影响所有使用 `when()/e()/m()` BEM mixin 组合的 SCSS 文件
- 预期改善 EP 一致性（`when/e/m` 相关 DIFF）
