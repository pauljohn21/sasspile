# scope-global-semantics Specification

## ADDED Requirements

### Requirement: $var !global penetrates to outermost scope

Writing `$var !global: value` inside an `@include` block SHALL modify the outer `$var` directly,下游所有引用看到新值。

#### Scenario: !global 值传播
- **WHEN** mixin body 内 `$bold: true !global`
- **THEN** 调用 `@include define-bold()` 后外部 `$bold` 输出 `true`

#### Scenario: Shadowing without !global 不污染外层
- **WHEN** mixin body 内 `$local: 1`(无 !global)
- **THEN** 调用 `@include` 后外部 `$local` 未定义或保持原值
