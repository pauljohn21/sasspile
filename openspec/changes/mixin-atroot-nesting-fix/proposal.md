## Why

`exec_mixin` 中 `AtRoot` → `AtRootDirect` 的转换**完全扁平化**了嵌套 `@at-root` 结构，导致 `@mixin e()` 在 `@mixin m()` 内部调用时，`e()` 生成的规则丢失了 `m()` 的 `@at-root` 选择器上下文。

具体表现：EP 文件中 `@include m($size) { @include e(header) { ... } }` 模式输出的 `.el-descriptions__header` 缺少前缀 `.el-descriptions--large`，与 dart-sass 输出不一致。

**根因证据**：trace 显示 exec_mixin 输出 `[AtRootDirect(Rule("&!--large,")), AtRootDirect(Rule(".el-descriptions__header,")), ...]` — 6 个独立兄弟节点，父节点 children 为空。

## What Changes

- 重构 `exec_mixin` 的 `AtRoot` 处理逻辑：不再将 `AtRoot` 扁平化为多个 `AtRootDirect` 兄弟节点
- 保留原始嵌套结构：`AtRootDirect` 内部包含完整的子规则树
- 在 `RuleBuilder::push` 的 `AtRootDirect` 分支中，递归组合子节点选择器与组合后的外层选择器
- 保留 EP BEM mixin 链的源码位置排序语义

## Capabilities

### New Capabilities
- `nested-atroot-context`: 嵌套 @at-root mixin 内部子节点继承外层 @at-root 的选择器上下文，正确生成组合选择器

### Modified Capabilities
- 无（不修改现有 capability 的 requirements）

## Impact

- **代码文件**：`src/eval/mixin.rs`（exec_mixin 重构）、`src/eval/rule.rs`（RuleBuilder::push 增强）
- **影响范围**：所有使用 `@at-root` 的 mixin（EP BEM 模式 b/e/m/when 系列）
- **测试**：ep_normalized_test（目标 descriptions.scss, descriptions-item.scss, form-item.scss, button.scss 等 IDENTICAL）
- **风险**：需保留 EP 已通过的 73/121 文件的输出不变
