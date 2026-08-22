## MODIFIED Requirements

### Requirement: plain CSS 限制
系统 SHALL 在 plain CSS 模式中检测不允许的操作并报错。系统 SHALL 在以下场景中检测并报错：sass() 函数调用、@use/@forward/@include/@function/@mixin at-rule、选择器和属性名中的插值 `#{...}`、声明值中的运算符（`+`/`-`/`*`/`/`）、Sass 变量引用 `$var`、父选择器后缀 `&-suffix`、占位选择器 `%placeholder`、顶级前导组合器（`> ` 或 `+ ` 开头）。错误信息格式为 "<X> isn't allowed in plain CSS"。

#### Scenario: sass() 在 plain CSS
- **WHEN** 在 plain CSS 模式中调用 `sass()`
- **THEN** 系统 报 "sass() conditions aren't allowed in plain CSS" 错误

#### Scenario: @use 在 plain CSS
- **WHEN** 在 `.css` 文件中使用 `@use "sass:math"`
- **THEN** 系统 报 "This at-rule isn't allowed in plain CSS." 错误

#### Scenario: @include 在 plain CSS
- **WHEN** 在 `.css` 文件中使用 `@include mixin-name`
- **THEN** 系统 报 "This at-rule isn't allowed in plain CSS." 错误

#### Scenario: 插值在 plain CSS
- **WHEN** 在 `.css` 文件中的选择器位置使用 `#{...}` 插值
- **THEN** 系统 报 "Interpolation isn't allowed in plain CSS." 错误

#### Scenario: 运算符在声明值中
- **WHEN** 在 `.css` 文件的声明值中使用 `+` 运算符
- **THEN** 系统 报 "Operators aren't allowed in plain CSS." 错误

#### Scenario: Sass 变量引用
- **WHEN** 在 `.css` 文件的声明值中引用 `$variable`
- **THEN** 系统 报 "Sass variables aren't allowed in plain CSS." 错误

#### Scenario: 父选择器后缀
- **WHEN** 在 `.css` 文件中使用 `&-suffix` 父选择器后缀
- **THEN** 系统 报 "Parent selectors can't have suffixes in plain CSS." 错误

#### Scenario: 占位选择器
- **WHEN** 在 `.css` 文件中使用 `%placeholder` 选择器
- **THEN** 系统 报 "Placeholder selectors aren't allowed in plain CSS." 错误

#### Scenario: 顶级前导组合器
- **WHEN** 在 `.css` 文件中使用 `> .child` 顶级前导组合器
- **THEN** 系统 报 "Top-level leading combinators aren't allowed in plain CSS." 错误
