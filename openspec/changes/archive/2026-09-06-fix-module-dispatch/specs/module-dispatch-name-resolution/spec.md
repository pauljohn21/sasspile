## ADDED Requirements

### Requirement: Module-qualified names SHALL resolve to canonical built-in function names
When a module-qualified name (e.g., `string.unquote`, `map.get`, `list.append`, `math.div`, `selector.parse`) is passed to `dispatch_builtin_module`, the dispatch function SHALL convert it to the canonical global name (e.g., `unquote`, `map-get`, `append`, `div`, `selector-parse`) before calling the underlying `call_xxx_builtin` function.

#### Scenario: string.unquote resolves correctly
- **WHEN** SCSS code calls `string.unquote("foo")` after `@use "sass:string"`
- **THEN** the call SHALL be dispatched to `call_string_builtin` with name `"unquote"` and return an unquoted string

#### Scenario: map.get resolves correctly
- **WHEN** SCSS code calls `map.get((a: 1), a)` after `@use "sass:map"`
- **THEN** the call SHALL be dispatched to `call_map_builtin` with name `"map-get"`

#### Scenario: list.append resolves correctly
- **WHEN** SCSS code calls `list.append(1px 2px, 3px)` after `@use "sass:list"`
- **THEN** the call SHALL be dispatched to `list::call` with name `"append"`

#### Scenario: math.div resolves correctly
- **WHEN** SCSS code calls `math.div(10, 2)` after `@use "sass:math"`
- **THEN** the call SHALL be dispatched to `math::call` with name `"div"`

#### Scenario: selector.parse resolves correctly
- **WHEN** SCSS code calls `selector.parse(".foo")` after `@use "sass:selector"`
- **THEN** the call SHALL be dispatched to `selector::call` with name `"selector-parse"`

#### Scenario: Global names remain unaffected
- **WHEN** code calls a built-in with its global name (e.g., `unquote("foo")`, `map-get((a:1), a)`)
- **THEN** the dispatch SHALL work identically — `unwrap_or(name)` fallback passes the global name through unchanged

#### Scenario: Already-correct color dispatch unchanged
- **WHEN** SCSS code calls `color.adjust($c, $red: 10)`
- **THEN** the fix SHALL NOT change any existing color dispatch behavior
