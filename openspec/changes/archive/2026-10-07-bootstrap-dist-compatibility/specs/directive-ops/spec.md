# Spec Delta

## ADDED Requirements

### Requirement: @content 占位符替换
系统 SHALL 在 `@include mixin { content }` 调用时，将 mixin body 中的 `@content` 占位符替换为调用方传入的 content block。

#### Scenario: 基本 content 替换
- **WHEN** `@mixin test { div { @content; } }` 后跟 `@include test { color: red; }`
- **THEN** 产物 SHALL 包含 `div { color: red; }`

#### Scenario: media-breakpoint mixin
- **WHEN** `@include media-breakpoint-up(md) { .custom { display: block; } }`
- **THEN** 产物 SHALL 包含 `@media (min-width: 768px) { .custom { display: block; } }`

#### Scenario: @content 多次出现
- **WHEN** mixin body 中 `@content` 出现两次
- **THEN** 两处 SHALL 都被替换为相同 content block

### Requirement: @mixin keyword arguments 和默认值表达式
系统 SHALL 支持 `@mixin name($arg: default)` 的语法，其中默认值可以是表达式（包括函数调用、变量引用）。调用时支持 `@include name($arg: value)` keyword 形式。

#### Scenario: 默认表达式
- **WHEN** `@mixin test($bp: map-get($grid-breakpoints, "md")) { ... }`
- **THEN** `$bp` 默认值 SHALL 正确求值为 `768px`

#### Scenario: keyword argument
- **WHEN** `@include test($bp: 992px)`
- **THEN** `$bp` 绑定为 `992px`，覆盖默认值

### Requirement: @for 支持 range 生成
系统 SHALL 支持 `@for $i from $start through $end` 迭代整数范围。

#### Scenario: basic @for
- **WHEN** 编译 `@for $i from 1 through 3 { .col-#{$i} { width: percentage($i/12); } }`
- **THEN** 产物 SHALL 包含 `.col-1`, `.col-2`, `.col-3`

### Requirement: @while 循环
系统 SHALL 支持 `@while $condition { body }` 循环，在条件为真时重复执行 body。

#### Scenario: basic @while
- **WHEN** 编译 `$i: 1; @while $i <= 3 { .item-#{$i} { content: $i; } $i: $i + 1; }`
- **THEN** 产物 SHALL 包含 `.item-1`, `.item-2`, `.item-3`
