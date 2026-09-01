## ADDED Requirements

### Requirement: @use module loop 检测

当模块 A 通过 `@use` 引用模块 B，而 B 已在加载中（在 `loaded_modules` 但不在 `module_cache` 中）时，SHALL 报 "Module loop" 错误。

#### Scenario: 直接循环引用

- **WHEN** 模块 A `@use "B"`，模块 B `@use "A"`
- **THEN** 报 "Module loop: this module is already being loaded."

#### Scenario: 间接循环引用

- **WHEN** 模块 A `@use "B"`，B `@use "C"`，C `@use "A"`
- **THEN** 报 "Module loop: this module is already being loaded."

### Requirement: @use with 配置验证

`@use "module" with ($var: value)` SHALL 验证配置变量在目标模块中以 `!default` 声明。

#### Scenario: 配置变量未声明 !default

- **WHEN** `@use "m" with ($x: 1)` 但模块 m 中 `$x` 未声明 `!default`
- **THEN** 报 "This variable was not declared with !default in the @used module."

#### Scenario: 重复配置

- **WHEN** 同一模块被两次 `@use with` 配置
- **THEN** 报 "This module was already loaded, so it can't be configured using \"with\"."

### Requirement: @import 文件冲突检测

`@import` SHALL 正确检测文件路径歧义（partial vs non-partial、扩展名冲突、index 冲突、import-only 冲突）。

#### Scenario: partial vs non-partial

- **WHEN** 同时存在 `_file.scss` 和 `file.scss`
- **THEN** 报文件歧义错误

#### Scenario: 扩展名冲突

- **WHEN** 同时存在 `file.scss` 和 `file.sass`
- **THEN** 报文件歧义错误
