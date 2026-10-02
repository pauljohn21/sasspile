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
