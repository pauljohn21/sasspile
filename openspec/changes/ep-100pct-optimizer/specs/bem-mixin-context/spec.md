## ADDED Requirements

### Requirement: BEM mixin 嵌套上下文正确传播
系统 SHALL 在 `@include m()` 嵌套于 `@include e()` 内部时，正确捕获元素级 `&` 选择器上下文，并在 `@content` 内使用该上下文。

#### Scenario: 单一 element 内的 modifier
- **WHEN** SCSS 使用 `@include e(content) { @include m(primary) { background: red } }`
- **THEN** 输出 `.el-badge__content--primary { background: red }` 而非 `.el-badge__content, --primary { background: red }`

#### Scenario: 多层嵌套 element-modifier
- **WHEN** SCSS 使用 `@include e(item) { @include m(active) { ... } }` 位于 `@include b(list)` 内
- **THEN** 输出 `.el-list__item--active { ... }`

#### Scenario: Each 循环内的 modifier
- **WHEN** SCSS 使用 `@each $type in (...) { @include m($type) { ... } }` 嵌套在 element 上下文内
- **THEN** 每个迭代正确输出 `parent--modifier` 组合选择器

### Requirement: Mixin at-root 内 $selector 字面量插值
系统 SHALL 在 mixin 内部 `@at-root { #{$selector} { ... } }` 中，`$selector` 值为调用点 `&` 的完整解析。

#### Scenario: utils-clearfix 伪元素输出
- **WHEN** `@include utils-clearfix` 在 `@include m(horizontal)` 内调用
- **THEN** 输出 `.el-button-group--horizontal::before, .el-button-group--horizontal::after { ... }`
- **AND** 不出现 `.el-button-group--horizontal, :before, .el-button-group--horizontal, :after { ... }`

### Requirement: 不引入 selector 泄漏
系统 SHALL 在修复 mixin 上下文后，不破坏现有 selector save/restore 修复——孙规则选择器不泄漏到后续兄弟规则。

#### Scenario: 兄弟规则隔离
- **WHEN** SCSS 使用 `@include m(a) { ... } @include m(b) { ... }`
- **THEN** rule `a` 的 selector 不影响 rule `b` 的输出上下文

### Requirement: BEM e() mixin 嵌套时 @at-root 上下文保留（深度 > 0）
系统 SHALL 在 `@include m($size)` 内嵌套 `@include e(header) { @include e(title) { ... } }` 时，内部 `e(title)` 的 `&` 解析为完整前缀链（包含 `__header` 层）。

#### Scenario: e(title) inside e(header) inside m($size)
- **WHEN** SCSS 使用 `@include b(descriptions) { @include m(large) { @include e(header) { @include e(title) { font-size: 16px } } } }`
- **THEN** 输出 `.el-descriptions--large .el-descriptions__header .el-descriptions__title { font-size: 16px }`
- **AND** 不输出 `.el-descriptions--large .el-descriptions__title`（缺少 __header 层）

#### Scenario: hitAllSpecialNestRule 正确命中
- **WHEN** `e(title)` 的 `&` 解析为包含 `--modifier` 修饰符的链（如 `.el-descriptions--large .el-descriptions__header`）
- **THEN** `hitAllSpecialNestRule` 正确识别修饰符前缀并进入特殊分支
- **AND** 输出选择器包含完整的 `__header` 中间层

#### Scenario: starts_with 前缀检测防双层膨胀
- **WHEN** 子 Rule selector 已以父 selector 开头（如 child = `.el-X--large .el-X__header`，parent = `.el-X--large`）
- **THEN** `RuleBuilder::push()` 跳过 combine_selectors 避免双层膨胀
- **AND** output 不包含 `.el-X--large .el-X--large .el-X__header` 重复前缀

#### Scenario: Top-level 非嵌套规则不组合（extend 兼容）
- **WHEN** SCSS 使用 `.child { @extend %base; }`（顶层，无嵌套）
- **THEN** descenter 保持原始行为：selector.clone()，不触发 compose-in-descender
- **AND** extend 语义无回归
