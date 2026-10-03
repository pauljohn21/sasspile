## ADDED Requirements

### Requirement: AtRootDirect child 节点字面 `&` 必须展开

当 `exec_mixin` 封装 mixin 输出将内部 `@at-root` 节点转换为 `CssNode::AtRootDirect`，且 AtRootDirect 内部 Rule 的 child selector 含字面 `&` 字符，`push_atroot_direct` 处理该 child 时必须将 `&` 替换为 AtRootDirect 的 `resolved_selector`（父级选择器上下文）。这与 sass-spec 官方 `parent-selector` 章节的 `&` 替换规范一致。

#### Scenario: step `&.is-flex` 字面 `&` 强制展开

- **WHEN** step.scss 有 `.el-step { @include pseudo(last-of-type) { @include e(line) { display: none } &.is-flex { flex-grow: 0 } } }`
- **THEN** 编译输出 CSS 中包含 `.el-step:last-of-type.is-flex { flex-grow: 0; flex-shrink: 0; flex-basis: auto !important }`
- **AND** 输出 CSS 中不含字面 `&` 字符
- **AND** `.el-step__line` 的 `display: none` 必须嵌套在 `.el-step:last-of-type` 下（输出 `.el-step:last-of-type .el-step__line{display:none}`）

#### Scenario: 嵌套 `&` 引用保留完整上下文

- **WHEN** 一个 AtRootDirect `resolved_selector` 已是多重嵌套形式（如 `.el-rate:focus-visible`）且 inner child Rule selector 含字面 `&`
- **THEN** `&` 被完整替换为 `resolved_selector`，输出 child selector = `resolved_selector` + suffix（替换后的结果）

#### Scenario: 多个 childhood Rules 全部正确展开

- **WHEN** 同一个 AtRootDirect 内含多个 child Rule 且每个 selector 均含字面 `&`
- **THEN** 所有 child Rule 的 selector 都正确展开，无字面 `&` 泄漏到最终 CSS

### Requirement: AtRootDirect child selector 不含 `&` 时强制 nest 到 resolved context

当一个 AtRootDirect 内部 child Rule 的 selector 不含字面 `&`，`push_atroot_direct` 的 else 分支必须调用 `combine_selectors(&resolved_selector, clean_sel)` 产生 descendant 关系，**不能** 当作"完整路径"直接使用。

这与 sass-spec `at-root` 规范一致：mixin 内部 `@at-root` 包装的选择器，在执行者（外层 RuleBuilder）的上下文中 **仍需** nest，除非显式指定 otherwise。

#### Scenario: anchor `e(marker)` 在 `&.--vertical` 内正确 nest

- **WHEN** anchor.scss 有 `&.#{$namespace}-anchor--vertical { @include e(marker) { ... } }`
- **THEN** 编译输出 CSS 中包含 `.el-anchor.el-anchor--vertical .el-anchor__marker{...}`
- **AND** 输出 CSS 中不含 `.el-anchor .el-anchor.el-anchor--vertical` 重复前缀
- **AND** 输出 CSS 中不含 `.el-anchor--vertical .el-anchor__marker` 缺少基础层

#### Scenario: popover `e(title)` 在 `&.el-popper` 内正确 nest

- **WHEN** popover.scss 有 `&.#{$namespace}-popper { @include e(title) { color: ...; ... } @include e(reference) { ... } }`
- **THEN** 编译输出包含 `.el-popover.el-popper .el-popover__title{...}` 而非 `.el-popover .el-popover.el-popper` 重复

#### Scenario: table-v2 `e(main)` 在 `&:hover` 内正确 nest

- **WHEN** table-v2.scss 有类似 `&:hover { @include e(main) {...} }` 的结构
- **THEN** 编译输出不含 `.el-table-v2__root .el-table-v2__root` 重复

#### Scenario: 非规则上下文中的 mixin 调用

- **WHEN** mixin 在顶层（非规则）上下文中被调用（resolved_selector 为空）
- **THEN** else 分支直接输出 `clean_sel` 作为顶级规则（保持 EP mixin 在文档根输出的语义）
