## ADDED Requirements

### Requirement: @forward as prefix-* SHALL insert hyphen separator
When `@forward "url" is used with `as prefix-*` syntax, the forwarded member names SHALL be accessible with a hyphen between the prefix and the original name (e.g., `d-c` for prefix `d` and name `c`).

#### Scenario: Forwarded variable access with prefix
- **WHEN** a module uses `@forward "upstream" as d-*` and upstream defines `$c: e`
- **THEN** the forwarding module SHALL allow access via `namespace.d-c` returning value `e`

#### Scenario: Forwarded function call with prefix
- **WHEN** a module uses `@forward "upstream" as d-*` and upstream defines `@function c() { @return e }`
- **THEN** the forwarding module SHALL allow calling `namespace.d-c()` returning `e`

#### Scenario: Forwarded variable assignment with prefix
- **WHEN** a module uses `@forward "upstream" as d-*` and upstream defines `$a: old`
- **THEN** assigning `namespace.d-a: new` SHALL update upstream's `$a` to `new`

#### Scenario: Namespace access without prefix (no `as`)
- **WHEN** a module uses `@forward "upstream"` without `as`
- **THEN** forwarded members SHALL be accessible with original names (no separator added)
