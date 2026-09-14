# import-nested-scope

## Requirements

### Requirement: Nested import chain has correct scope

WHEN file A imports file B which imports file C, file C SHALL be able to read variables from both A and B.

#### Scenario: Three-level import chain

- GIVEN `a.scss` defines `$x: 1` and imports `b.scss`
- GIVEN `b.scss` defines `$y: 2` and imports `c.scss`
- GIVEN `c.scss` uses both `$x` and `$y`
- WHEN all imports are resolved
- THEN `$x` resolves to `1` and `$y` resolves to `2`

#### Scenario: Write isolation

- GIVEN importing context has `$a: original`
- GIVEN imported file modifies `$a: changed`
- WHEN `@import "other"` completes
- THEN importing context's `$a` is still `original`
