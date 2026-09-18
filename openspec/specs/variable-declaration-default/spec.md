# variable-declaration-default Specification

## ADDED Requirements

### Requirement: $var !default semantics

`$var: value !default` SHALL only assign when `$var` is undefined (or null); skip if already defined.

#### Scenario: !default 被覆盖
- **WHEN** `a.scss` 中 `$x: 1 !default;`;调用者已通过 `@use ... with ($x: 2)` 传入 `2`
- **THEN** `$x` 最终为 `2`

#### Scenario: !default 在空值上生效
- **WHEN** `a.scss` 中 `$x: 1 !default;`;调用者未覆盖
- **THEN** `$x` 为 `1`

### Requirement: Normal declaration overrides !default

A normal declaration (`$var: new`) SHALL override a `!default` declaration in file load order.

#### Scenario: 后写覆盖
- **WHEN** `$x: !default 1; $x: 2;`(两种写法顺序作用)
- **THEN** `$x` 为 `2`
