## ADDED Requirements

### Requirement: @forward SHALL propagate !default variable overrides
When a file defines a variable with `!default` and is imported via `@forward` through a chain, the importing context's variable value SHALL override the `!default` value.

#### Scenario: Direct forward with !default override
- **WHEN** `_upstream.scss` defines `$a: original !default; b { c: $a }`, `_midstream.scss` does `@forward "upstream"`, and `input.scss` defines `$a: configured; @import "midstream";`
- **THEN** the output SHALL be `b { c: configured; }`

#### Scenario: Forward chain without override keeps !default
- **WHEN** `_upstream.scss` defines `$a: original !default; b { c: $a }`, `_midstream.scss` does `@forward "upstream"`, and `input.scss` does `@import "midstream";` (no `$a` defined)
- **THEN** the output SHALL be `b { c: original; }`

#### Scenario: Midstream defines variable, upstream has !default
- **WHEN** `_upstream.scss` defines `$a: original !default; b { c: $a }`, `_midstream.scss` defines `$a: midstream; @forward "upstream"`, and `input.scss` does `@import "midstream";`
- **THEN** the output SHALL be `b { c: midstream; }`

#### Scenario: Input overrides through forward chain
- **WHEN** `_upstream.scss` defines `$a: original !default; b { c: $a }`, `_midstream.scss` defines `$a: midstream; @forward "upstream"`, and `input.scss` defines `$a: configured; @import "midstream";`
- **THEN** the output SHALL be `b { c: configured; }`

#### Scenario: Prefixed variable name with !default through forward
- **WHEN** `_upstream.scss` defines `$d-a: original !default; b { c: $d-a }`, and `input.scss` defines `$d-a: configured; @import "midstream";`
- **THEN** the output SHALL be `b { c: configured; }`
