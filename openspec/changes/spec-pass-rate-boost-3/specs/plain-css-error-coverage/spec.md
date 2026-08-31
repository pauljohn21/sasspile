## ADDED Requirements

### Requirement: plain CSS 模式 sass 特有语法检测

系统 SHALL 在 plain CSS 模式（`.css` 文件）中检测 sass 特有语法并报错。包括 sass() 条件、Interpolation（`#{}`）、Operators（`+`/`-`/`*`/`/`）在 CSS 声明值中的使用。

#### Scenario: sass() conditions 在 plain CSS 中报错
- **WHEN** plain CSS 文件中包含 `sass()` 条件
- **THEN** 系统 SHALL 报 "sass() conditions aren't allowed in plain CSS" 错误

#### Scenario: Interpolation 在 plain CSS 中报错
- **WHEN** plain CSS 文件中包含 `#{...}` 插值
- **THEN** 系统 SHALL 报 "Interpolation isn't allowed in plain CSS." 错误

#### Scenario: Operators 在 plain CSS 中报错
- **WHEN** plain CSS 文件中声明值包含 `+`/`-`/`*`/`/` 运算符
- **THEN** 系统 SHALL 报 "Operators aren't allowed in plain CSS." 错误

#### Scenario: Sass variables 在 plain CSS 中报错
- **WHEN** plain CSS 文件中包含 `$variable` 引用
- **THEN** 系统 SHALL 报 "Sass variables aren't allowed in plain CSS." 错误

#### Scenario: Top-level leading combinators 在 plain CSS 中报错
- **WHEN** plain CSS 文件中包含顶层 `> + ~` 组合器
- **THEN** 系统 SHALL 报 "Top-level leading combinators aren't allowed in plain CSS." 错误

#### Scenario: Parent selectors 带后缀在 plain CSS 中报错
- **WHEN** plain CSS 文件中包含 `&suffix` 父选择器后缀
- **THEN** 系统 SHALL 报 "Parent selectors can't have suffixes in plain CSS." 错误

### Requirement: plain CSS at-rule 限制

系统 SHALL 在 plain CSS 模式中检测 sass 特有 at-rules（@if/@for/@each/@while/@mixin/@include/@function/@use/@forward/@extend/@at-root）并报错，但允许标准 CSS at-rules（@media/@supports/@import/@keyframes 等）。

#### Scenario: sass at-rule 在 plain CSS 中报错
- **WHEN** plain CSS 文件中包含 `@if`/`@for`/`@mixin` 等 sass at-rule
- **THEN** 系统 SHALL 报 "This at-rule isn't allowed in plain CSS." 错误

#### Scenario: CSS at-rule 在 plain CSS 中允许
- **WHEN** plain CSS 文件中包含 `@media`/`@supports`/`@keyframes` 等 CSS at-rule
- **THEN** 系统 SHALL 正常处理，不报错

### Requirement: plain CSS 声明上下文检测

系统 SHALL 在 plain CSS 模式中检测声明是否在 style rule 上下文内。顶层声明 SHALL 报错。

#### Scenario: 顶层声明在 plain CSS 中报错
- **WHEN** plain CSS 文件中声明出现在顶层（非规则体内）
- **THEN** 系统 SHALL 报 "Declarations may only be used within style rules." 错误
