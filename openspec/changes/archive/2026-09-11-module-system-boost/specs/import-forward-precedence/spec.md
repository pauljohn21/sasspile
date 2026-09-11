## ADDED Requirements

### Requirement: Forwarded definitions SHALL take precedence over local definitions through imports
When a file is `@import`'d and it contains `@forward`, the forwarded definitions SHALL take precedence over local definitions in the file doing the importing.

#### Scenario: Variable from @forward overrides local variable
- **WHEN** `input.scss` defines `$a: in-input` then `@import "midstream"` where midstream has `@forward "upstream"` with `$a: in-upstream`
- **THEN** after the import, `$a` SHALL resolve to `in-upstream`

#### Scenario: Nested scope variable assignment
- **WHEN** `b { $a: in-input; @import "midstream"; c: $a; }` where midstream forwards `$a: in-upstream`
- **THEN** `c` SHALL be `in-upstream` (forwarded takes precedence over nested local)

#### Scenario: Multiple @import with function override
- **WHEN** two consecutive `@import` statements import files with different forwarded functions of the same name
- **THEN** the second import's forwarded function SHALL override the first

#### Scenario: Multiple @import with mixin override
- **WHEN** two consecutive `@import` statements import files with different forwarded mixins of the same name
- **THEN** the second import's forwarded mixin SHALL override the first

#### Scenario: @forward with `with()` non-overridable
- **WHEN** `@import "midstream"` where midstream has `@forward "upstream" with ($a: midstream)` and input defines `$a: input`
- **THEN** the forwarded `$a` value SHALL be `midstream` (from the `with()` configuration)
