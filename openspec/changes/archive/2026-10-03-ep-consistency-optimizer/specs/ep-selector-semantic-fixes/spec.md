## ADDED Requirements

### Requirement: CSS 自定义属性 var() null fallback 序列化规范
当 `var()` 函数的 fallback 值为 `null` 时，序列化 SHALL 输出空字符串（`, `逗号后无值），输出格式为 `var(--name, )`。

#### Scenario: getCssVar() with null fallback
- **WHEN** SCSS 调用 `getCssVarWithDefault('message-close-size', null)` 展开为 `var(--el-message-close-size, null)`
- **THEN** 序列化输出 SHALL 为 `var(--el-message-close-size, )`（null → 空字符串）

#### Scenario: 非 null fallback 保持不变
- **WHEN** SCSS `var(--el-message-close-size, 16px)`
- **THEN** 输出 SHALL 为 `var(--el-message-close-size, 16px)`（不受影响）

### Requirement: @at-root body 内声明块正确嵌套在父选择器下
`@at-root { #{$selector} { #{$currentSelector} { @content } } }` 模式中，`@content` 内的声明 SHALL 出现在 `#{$currentSelector}` 选择器下，而非 `#{$selector}` 下。

#### Scenario: @mixin e() 的 @content 声明位置
- **WHEN** SCSS `.el-anchor.el-anchor--horizontal { @include e(list) { @include e(item) { padding-left: 16px; } } }`
- **THEN** `padding-left: 16px`  SHALL 出现在 `.el-anchor.el-anchor--horizontal .el-anchor__list .el-anchor__item { }` 中（声明在最内层选择器）

### Requirement: margin-bottom 类声明不跨选择器错误移动
嵌套规则中的声明 SHALL 保持在其所属选择器下，不应泄漏到外层或兄弟选择器。

#### Scenario: form-item margin-bottom 位置正确
- **WHEN** SCSS `.el-form-item { @include e('content') { margin-bottom: 0; } .el-form-item {}}`
- **THEN** `margin-bottom: 0`  SHALL 出现在 `.el-form-item__content { }` 而非 `.el-form-item { }` 中

### Requirement: appearance 属性序列化保留标准值
`appearance: none` 的值 SHALL 保持不变，不添加额外前缀（`-webkit-appearance`/`-moz-appearance` 由 autoprefixer 负责，非 sasspile 职责）。

#### Scenario: appearance: none
- **WHEN** SCSS `appearance: none`
- **THEN** 输出 SHALL 为 `appearance: none`（不加前缀）
