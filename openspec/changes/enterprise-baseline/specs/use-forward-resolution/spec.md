# use-forward-resolution Specification

## ADDED Requirements

### Requirement: @use "./foo" as * injects all public symbols

`@use './base.scss' as *` SHALL expose all public `$variable`, `@mixin`, `@function` to the current file scope.下游文件中 `$var` 解析指向被引入者。

#### Scenario: element-plus index 链式 @use
- **WHEN** `index.scss` 有 `@use './base.scss'` 且 `base.scss` 定义 `$color-primary: #409eff`
- **THEN** 后续文件 `@use './button.scss'` 中使用 `$color-primary` 解析为 `#409eff`(相等字符串比较)

#### Scenario: @use with 参数覆盖
- **WHEN** `./a.scss` 定义 `$x: 1 !default;`
- **AND** 调用者 `@use './a' with ($x: 2)`
- **THEN** `$x` 在调用者作用域解析为 `2`

### Requirement: @forward "./bar" forwards upstream symbols

`@forward './colors'` SHALL allow downstream `@use 'this-file'` to access public symbols from `colors.scss`,但本文件作用域内不引入。

#### Scenario: transitive @forward
- **WHEN** `a.scss` 有 `@forward './b.scss'`;`b.scss` 定义 `$var: 1`
- **AND** 顶层 `@use './a'` 访问 `$var`
- **THEN** `$var` 解析为 `1`
