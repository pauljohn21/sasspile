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

### Requirement: Interpolation With Parent Selector `&`

`eval_rule` SHALL expand `&` inside `#{...}` interpolation to the parent selector value, while preserving literal `&` for `combine_selectors` handling.

#### Scenario: Simple interpolation concat

- **WHEN** `.el-overlay { #{& + '-root'} { height: 0 } }` is evaluated
- **THEN** output selector SHALL be `.el-overlay-root` (parent expanded)

#### Scenario: Interpolation with multiple `&`

- **WHEN** `.x { #{& + '__a'}, #{& + '__b'} { ... } }` is evaluated
- **THEN** output SHALL be `.x__a, .x__b`

#### Scenario: Nested interpolation

- **WHEN** `.x { #{& + #{'-suffix'}} { ... } }` is evaluated
- **THEN** output SHALL be `.x-suffix`

#### Scenario: Literal `&` preserved for combine_selectors

- **WHEN** `.x { & { color: red } }` is evaluated (no `#{`)
- **THEN** output SHALL be `.x` (combine_selectors handles descendant)
- **AND** `&` SHALL NOT be expanded during `eval_selector_str`

#### Scenario: Mixed literal and interpolation `&`

- **WHEN** `.x { & #{& + '-y'} { ... } }` is evaluated
- **THEN** literal `&` preserved, interpolated `&` expanded
- **AND** `combine_selectors` produces correct combined selector

### Requirement: At-Root Hoisting Order

`RuleBuilder::build` SHALL insert `@at-root` hoisted nodes after parent declarations but before nested child rules.

#### Scenario: Parent has declarations and at-root children

- **WHEN** `.x { color: red; @at-root { .y { ... } } .z { ... } }` is evaluated
- **THEN** output order SHALL be: `.x { color: red }`, `.y { ... }`, `.x .z { ... }`

#### Scenario: Parent has no declarations (pure mixin)

- **WHEN** `.x { @at-root { .y { ... } } .z { ... } }` is evaluated
- **THEN** output order SHALL be: `.y { ... }`, `.x .z { ... }`
