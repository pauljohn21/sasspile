## ADDED Requirements

### Requirement: Module variable visibility in nested @include
当 @use 导入的模块变量在嵌套的 @include mixin 中使用，且 mixin 内部执行运算时，系统 SHALL 保持变量对 mixin 可见。

#### Scenario: Carousel/Cascader loop with module variables
- **WHEN** carousel.scss 中 `@each $break in $breakpoints { @include respond-to($break) { ... $var-from-module ... } }`
- **THEN** mixin 内可访问模块导入的 `$var-from-module`

#### Scenario: Input-number scoped variables
- **WHEN** input-number.scss 中 mixin @include 在嵌套规则内并引用模块级变量
- **THEN** 变量解析正确，不报 "Undefined variable" 错误

### Requirement: Mixed module and local variable resolution
当模块变量与局部变量同名且在同一 mixin 内使用时，系统 SHALL 按 SCSS 作用域规则正确解析（局部优先）。

#### Scenario: Variable shadowing in mixin
- **WHEN** 模块导出 `$namespace: 'el'` 且 mixin 内 `$local-namespace` 存在
- **THEN** mixin 内使用 `$namespace` 解析到局部定义而非模块导入

### Requirement: @use variable in @if/@else chain
当 @use 导入变量用于 @if/@else 条件判断时，系统 SHALL 正确读取已赋值的变量状态。

#### Scenario: Conditional variable check in base.scss
- **WHEN** `@if $enable-dark-mode { ... }` 变量来自 @use 模块
- **THEN** 条件分支选择正确，不因变量读取时机出错
