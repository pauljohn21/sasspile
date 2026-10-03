## ADDED Requirements

### Requirement: exec_mixin 出口 AtRootDirect 包装需保留 child selector nest 上下文

`exec_mixin` 出口处将 `CssNode::AtRoot(inner, _)` 的每个非 Declaration 内部节点包装为 `CssNode::AtRootDirect(Box::new(other))` 时，必须确保后续 `RuleBuilder::push` 的 `CssNode::AtRootDirect` 分支调用 `push_atroot_direct` 对 AtRootDirect 内部 child （尤其是含 `&` 的 kid selector）执行完整的 `combine_selectors` nest，与调用者 selector context 正确组合。

#### Scenario: e() mixin 内嵌套字面 `&` 的 selector 正确展开

- **WHEN** body 含 `e(item)` mixin 调用，而 `e(item)` mixin body 内部 @content 包含含 `&` 的选择器（如 pseudo mixin）
- **THEN** 输出 CSS 中 `&` 被完整展开为选择器引用

#### Scenario: nested AtRootDirect 递归处理

- **WHEN** 嵌套 mixin 中有多层 `AtRootDirect` 包装（如 rate.scss 的 `&:focus-visible { @include e(item) { & .rate__icon { @include when(focus-visible) } } }`）
- **THEN** 每一层 AtRootDirect 的 child selector 都结合其 resolved_selector 正确展开

### Requirement: push_atroot_direct 的 resolved_selector 合成后再 bench 育儿

AtRootDirect 包装一个 Rule 时，`resolved_selector` 是 `push_atroot_direct` 的第一个 primary key：combine_selectors(self.selector, clean_parent_sel)。当 clean_parent_sel 不含 `&`、不以 `:` / `[` 开头时，当前代码走 else 分支"视为完整路径"。此分支 **必须 nest** combine(self.selector, clean_parent_sel)，产生 `.parent .child`  descendant 选择器，除非 self.selector 为空（即嵌套顶层 mixin 调用）。

#### Scenario: anchor `e(marker)` resolved_selector 正确合成

- **WHEN** anchor.scss 的 `&.#{$namespace}-anchor--vertical { @include e(marker) { ... } }` 求值时：
  1. b(anchor) → outer rule selector = `.el-anchor`
  2. `&.--vertical` → 解析为 `.el-anchor.el-anchor--vertical`
  3. `e(marker)` 的 `@at-root { .el-anchor__marker, { @content } }` 被 exec_mixin 包装为 `AtRootDirect(Rule(.el-anchor__marker, ...))`
  4. `push_atroot_direct` 在该 AtRootDirect 进入时 self.selector = `.el-anchor.el-anchor--vertical`
  5. clean_parent_sel = `.el-anchor__marker`
  6. else 分支 combine → `.el-anchor.el-anchor--vertical .el-anchor__marker`
- **THEN** 输出包含 `.el-anchor.el-anchor--vertical .el-anchor__marker{...}`

#### Scenario: rate 三层嵌套内层合成

- **WHEN** rate.scss `&:focus-visible { @include e(item) { ... } }` 的 body 执行时：
  1. outer selector = `.el-rate:focus-visible`
  2. `e(item)` mixin body @at-root `{ .el-rate__item, { @content & .rate__icon {...} } }`
  3. clean_parent_sel = `.el-rate__item`, combine with `.el-rate:focus-visible` → `.el-rate:focus-visible .el-rate__item`
  4. 内部 @content `& .rate__icon` 中 `&` = `.el-rate__item` → `.el-rate:focus-visible .el-rate__item .el-rate__icon`
  5. `when(focus-visible)` mixin 添加 `.is-focus-visible` class
- **THEN** 输出包含 `.el-rate:focus-visible .el-rate__item .el-rate__icon.is-focus-visible{...}`
- **AND** 输出不含 `.el-rate .el-rate:focus-visible` 重复前缀
