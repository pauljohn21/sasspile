# NotPseudoIsWrapper Specification

## Purpose

Defines that CSS `:not()` pseudo-class with multiple selector arguments MUST be serialized using the `:is()` wrapper to match dart-sass output and the CSS Selectors Level 4 spec.

## Requirements

### Requirement: :not() with multiple arguments uses :is() wrapper

When the serializer encounters `:not()` with a selector list of 2 or more arguments, it SHALL wrap the arguments in `:is()`.

#### Scenario: :not with two class arguments
- **WHEN** the SCSS source contains `:not(.is-disabled, .is-focused)`
- **THEN** the output SHALL be `:not(:is(.is-disabled, .is-focused))`

#### Scenario: :not with single argument
- **WHEN** the SCSS source contains `:not(.active)`
- **THEN** the output SHALL be `:not(.active)` (unchanged, no wrapper)

#### Scenario: :not with three arguments
- **WHEN** the SCSS source contains `:not(.a, .b, .c)`
- **THEN** the output SHALL be `:not(:is(.a, .b, .c))`
