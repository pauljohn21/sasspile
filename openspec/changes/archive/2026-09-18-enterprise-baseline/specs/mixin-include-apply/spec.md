# mixin-include-apply Specification

## ADDED Requirements

### Requirement: @mixin definition + @include call expansion

`@mixin name($a, $b: default) {...}` SHALL register; `@include name($x)` SHALL substitute formal params with actual params inside the reducer then emit the body.

#### Scenario: @include 替换两个参数
- **WHEN** `@mixin btn($size, $color) { .btn { width: $size; color: $color; } }`
- **`@include btn(16px, blue)` 被编译
- **THEN** 输出 `.btn{width:16px;color:blue}`

#### Scenario: @include 使用默认参数
- **WHEN** `@mixin pad($x: 8px) { padding: $x; }` + `@include pad()`
- **THEN** 输出 `padding:8px`
