# module-advanced Specification

## Purpose
TBD - created by archiving change feature-completeness-boost. Update Purpose after archive.
## Requirements
### Requirement: @use 命名空间冲突检测
系统 SHALL 在同一文件内对同一模块多次 `@use` 使用不同命名空间时报错。

#### Scenario: 重复 @use 无 as
- **WHEN** 同一文件包含两次 `@use "sass:math"`
- **THEN** 报 `There's already a module with namespace "math"` 错误

#### Scenario: 不同命名空间可共存
- **WHEN** 同一文件包含 `@use "sass:math" as m1` 和 `@use "sass:math" as m2`
- **THEN** 编译成功，两个命名空间独立可用

### Requirement: @forward show/hide 过滤
系统 SHALL 支持 `@forward "url" show $a, $b` 和 `@forward "url" hide $a, $b` 成员过滤。

#### Scenario: show 白名单
- **WHEN** `@forward "lib" show $var, fn`
- **THEN** 仅导出 `$var` 和 `fn`，其他成员不可见

#### Scenario: hide 黑名单
- **WHEN** `@forward "lib" hide $internal`
- **THEN** 导出除 `$internal` 外的所有成员

### Requirement: @forward prefix 拼接
系统 SHALL 支持 `@forward "url" as prefix-*` 语法。

#### Scenario: 带前缀转发
- **WHEN** `@forward "lib" as lib-*`
- **THEN** 所有转发成员名前加 `lib-`

### Requirement: @forward with 配置
系统 SHALL 支持 `@forward "url" with ($var: value)` 对转发模块进行配置。

#### Scenario: 带配置转发
- **WHEN** `@forward "theme" with ($primary: blue)`
- **THEN** 转发模块的默认变量被覆盖

### Requirement: @import 歧义检测
系统 SHALL 在 `@import` 找到多个候选文件时报错并列出候选项。

#### Scenario: 双候选歧义
- **WHEN** `@import "foo"` 同时存在 `_foo.scss` 和 `foo.scss`
- **THEN** 报 `It's not clear which file to import. Found:` 错误并列出候选项

#### Scenario: extension 歧义
- **WHEN** `@import "bar"` 同时存在 `bar.scss` 和 `bar.sass`
- **THEN** 报歧义错误

### Requirement: @use 路径解析
系统 SHALL 正确解析 `@use` 中的相对路径和 load-path 路径。

#### Scenario: 相对路径
- **WHEN** `@use "./utils"` 从 `src/main.scss`
- **THEN** 正确加载 `src/_utils.scss`

#### Scenario: load-path 查找
- **WHEN** `@use "lib/helpers"` 且 `lib/` 在 load-path 中
- **THEN** 从 load-path 目录加载 `_helpers.scss`

