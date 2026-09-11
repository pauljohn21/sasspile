## ADDED Requirements

### Requirement: @extend SHALL work across module boundaries with proper scope rules
When modules `@use` a common module, `@extend` targets SHALL be resolved across module boundaries following Sass scope rules.

#### Scenario: Diamond dependency extend merging
- **WHEN** both `left` and `right` modules `@use "other"` and both `@extend %in-other`
- **THEN** the output SHALL merge the extender selectors into a single rule with the target's declarations

#### Scenario: @extend through :is() pseudo-selector
- **WHEN** `midstream` defines `:is(in-midstream) { @extend in-upstream }` and `input` defines `in-input { @extend in-midstream }`
- **THEN** the output SHALL contain `in-upstream, :is(in-midstream, in-input) { a: b; }`

#### Scenario: @extend through :matches() pseudo-selector
- **WHEN** `midstream` defines `:matches(in-midstream) { @extend in-upstream }` and `input` defines `in-input { @extend in-midstream }`
- **THEN** the output SHALL contain `in-upstream, :matches(in-midstream, in-input) { a: b; }`

#### Scenario: Transitive extend (A extends B, B extends C)
- **WHEN** `in-input` extends `in-midstream` which extends `in-upstream` (in separate modules)
- **THEN** the output SHALL contain `in-upstream, in-midstream, in-input` with the declaration

#### Scenario: Sibling modules cannot extend each other
- **WHEN** `left` and `right` are sibling modules (no shared ancestor module relationship)
- **THEN** `@extend` from one sibling to another SHALL NOT cross module boundaries (scope isolation)

#### Scenario: Private selector extend (optional)
- **WHEN** `input` tries to extend `%-in-other` (private selector)
- **THEN** with `!optional`, the extend SHALL silently fail (no error)
