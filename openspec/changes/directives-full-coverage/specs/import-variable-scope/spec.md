## ADDED Requirements

### Requirement: @import SHALL share variable scope with importing context
When a file is @imported, variables defined in the importing context before the @import statement SHALL be visible to the imported file and SHALL override `!default` variables.

#### Scenario: Variable defined before @import overrides !default
- **WHEN** `_other.scss` defines `$a: original !default; b { c: $a }` and `input.scss` defines `$a: configured; @import "other";`
- **THEN** the output SHALL be `b { c: configured; }`

#### Scenario: Variable defined between two @imports
- **WHEN** input.scss does `@import "other"; $a: changed; @import "other"; d { e: $a; }`
- **THEN** the first @import outputs `b { c: original; }` (no override yet), the second also `b { c: original; }` (the `$a: changed` is after first import but the second import still sees original scope state for !default)

#### Scenario: @import twice produces duplicate output
- **WHEN** input.scss does `$a: configured; @import "other"; @import "other";`
- **THEN** the output SHALL contain `b { c: configured; }` twice (once per import)

#### Scenario: Nested @import with variable scope
- **WHEN** `a.scss` defines `$a: configured; @import "midstream";` and `midstream.scss` does `@import "upstream";` and `upstream.scss` defines `$a: original !default; b { c: $a }`
- **THEN** the output SHALL be `b { c: configured; }`

#### Scenario: Separate file config through forward
- **WHEN** `config.scss` defines `$a: configured;`, `config_wrapper.scss` does `@forward "config";`, `midstream.scss` does `@forward "upstream";`, and input does `@import "config_wrapper"; @import "midstream";`
- **THEN** the output SHALL be `b { c: configured; }`

#### Scenario: Unrelated variable does not affect !default
- **WHEN** input.scss defines `$a: configured; $d: other; @import "midstream";` where midstream only uses `$a`
- **THEN** the output SHALL be `b { c: configured; }` (unrelated `$d` has no effect)
