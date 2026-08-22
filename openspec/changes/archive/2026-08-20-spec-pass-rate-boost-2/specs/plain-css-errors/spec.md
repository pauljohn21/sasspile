## ADDED Requirements

### Requirement: plain CSS at-rule 检测
系统 SHALL 在 plain CSS 模式中检测不允许的 at-rule 并报错 "This at-rule isn't allowed in plain CSS."

#### Scenario: @use 在 plain CSS
- **WHEN** 在 `.css` 文件中使用 `@use "sass:math"`
- **THEN** 系统 报 "This at-rule isn't allowed in plain CSS." 错误

#### Scenario: @forward 在 plain CSS
- **WHEN** 在 `.css` 文件中使用 `@forward "module"`
- **THEN** 系统 报 "This at-rule isn't allowed in plain CSS." 错误

#### Scenario: @include 在 plain CSS
- **WHEN** 在 `.css` 文件中使用 `@include mixin-name`
- **THEN** 系统 报 "This at-rule isn't allowed in plain CSS." 错误

#### Scenario: @function 在 plain CSS
- **WHEN** 在 `.css` 文件中使用 `@function foo() { }`
- **THEN** 系统 报 "This at-rule isn't allowed in plain CSS." 错误

#### Scenario: @mixin 在 plain CSS
- **WHEN** 在 `.css` 文件中使用 `@mixin bar() { }`
- **THEN** 系统 报 "This at-rule isn't allowed in plain CSS." 错误

### Requirement: plain CSS interpolation 限制
系统 SHALL 在 plain CSS 模式中检测不允许位置的插值并报错。

#### Scenario: 选择器中的插值
- **WHEN** 在 `.css` 文件中的选择器位置使用 `#{...}` 插值
- **THEN** 系统 报 "Interpolation isn't allowed in plain CSS." 错误

#### Scenario: 属性名中的插值
- **WHEN** 在 `.css` 文件中的属性名位置使用 `#{...}` 插值
- **THEN** 系统 报 "Interpolation isn't allowed in plain CSS." 错误

### Requirement: plain CSS 运算符限制
系统 SHALL 在 plain CSS 模式中检测不允许的运算符并报错。

#### Scenario: 加号运算符
- **WHEN** 在 `.css` 文件的声明值中使用 `+` 运算符
- **THEN** 系统 报 "Operators aren't allowed in plain CSS." 错误

#### Scenario: 减号运算符
- **WHEN** 在 `.css` 文件的声明值中使用 `-` 运算符（非负号）
- **THEN** 系统 报 "Operators aren't allowed in plain CSS." 错误

#### Scenario: 乘除运算符
- **WHEN** 在 `.css` 文件的声明值中使用 `*` 或 `/` 运算符
- **THEN** 系统 报 "Operators aren't allowed in plain CSS." 错误

### Requirement: plain CSS 变量限制
系统 SHALL 在 plain CSS 模式中检测 Sass 变量使用并报错。

#### Scenario: 变量引用
- **WHEN** 在 `.css` 文件的声明值中引用 `$variable`
- **THEN** 系统 报 "Sass variables aren't allowed in plain CSS." 错误

#### Scenario: 父选择器后缀
- **WHEN** 在 `.css` 文件中使用 `&-suffix` 父选择器后缀
- **THEN** 系统 报 "Parent selectors can't have suffixes in plain CSS." 错误

#### Scenario: 占位选择器
- **WHEN** 在 `.css` 文件中使用 `%placeholder` 选择器
- **THEN** 系统 报 "Placeholder selectors aren't allowed in plain CSS." 错误

#### Scenario: 顶级前导组合器
- **WHEN** 在 `.css` 文件中使用 `> .child` 或 `+ .sibling` 顶级前导组合器
- **THEN** 系统 报 "Top-level leading combinators aren't allowed in plain CSS." 错误
