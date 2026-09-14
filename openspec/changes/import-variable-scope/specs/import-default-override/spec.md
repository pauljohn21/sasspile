# import-default-override

## Requirements

### Requirement: Importing context overrides !default variables

WHEN importing context has a variable defined AND imported file declares the same variable with `!default`, the importing context's value SHALL be used.

#### Scenario: !default overridden by importing context

- GIVEN importing context has `$a: configured`
- GIVEN imported file has `$a: default !default`
- WHEN `@import "other"` is evaluated
- THEN `$a` resolves to `configured`

#### Scenario: !default used when importing context has no value

- GIVEN importing context has no `$a`
- GIVEN imported file has `$a: default !default`
- WHEN `@import "other"` is evaluated
- THEN `$a` resolves to `default`
