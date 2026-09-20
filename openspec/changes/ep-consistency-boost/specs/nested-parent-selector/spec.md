# Spec: Nested Parent Selector (`&`) Expansion

## ADDED Requirements

### Requirement: At-Root Rules With Parent Reference Expand `&`

`RuleBuilder::push` SHALL expand `&` in selectors of `@at-root` child rules when the selector contains a parent reference, even when no `query` is present.

#### Scenario: BEM modifier mixin outputs corrected selector

- **WHEN** `m()` mixin sets `$selector: &` inside `.el-avatar { }` and emits `@at-root { &--circle { ... } }`
- **THEN** the output selector SHALL be `.el-avatar--circle` (not literal `&--circle`)
- **AND** the rule SHALL be at root level (not nested)

#### Scenario: BEM modifier with `compound.is-class` pattern

- **WHEN** `.el-badge__content { &.is-fixed { ... } }` is evaluated
- **THEN** output SHALL be `.el-badge__content.is-fixed` at root level

#### Scenario: Pseudo-element `&::before` expansion

- **WHEN** `.el-breadcrumb { &::before, &::after { ... } }` is evaluated
- **THEN** output SHALL be `.el-breadcrumb::before, .el-breadcrumb::after`

#### Scenario: At-root without parent reference preserved as-is

- **WHEN** `@at-root { .unrelated { ... } }` is emitted (no `&` in selector)
- **THEN** output SHALL be `.unrelated` at root level
- **AND** `root_nodes` direct append SHALL be used

#### Scenario: Deeply nested mixin with multiple `&`

- **WHEN** `m()` mixin inside `m()` emits `#{$selector}--x` where `$selector` = `.el-avatar--circle`
- **THEN** output SHALL be `.el-avatar--circle--x` (or per CSS spec behavior)
