# import-variable-visibility

## Requirements

### Requirement: Importing context variables visible in imported file

WHEN an `@import "file"` is evaluated, the imported file SHALL be able to read variables defined in the importing context.

#### Scenario: Basic variable visibility

- GIVEN importing context has `$color: blue`
- GIVEN imported file `_other.scss` uses `$color`
- WHEN `@import "other"` is evaluated
- THEN the output uses `blue` for `$color`

#### Scenario: Variable defined after @import doesn't affect imported file

- GIVEN `_other.scss` uses `$color`
- WHEN importing file has `$color: red` AFTER `@import "other"`
- THEN imported file uses the value of `$color` at the point of `@import`
