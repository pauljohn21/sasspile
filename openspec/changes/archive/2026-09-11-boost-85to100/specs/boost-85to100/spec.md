# Capability: boost-85to100

## ADDED Requirements

### Requirement: Comment parsing in @use and @forward declarations

The parser SHALL handle comments in all positions within `@use` and `@forward` declarations,
consistent with how `parse_variable` handles comments.

#### Scenario: Comment after @use keyword and before URL
- **WHEN** encountering `@use /* comment */ "module"`
- **THEN** the comment is skipped and the URL is parsed correctly

#### Scenario: Comment after URL and before `as`
- **WHEN** encountering `@use "module" /* comment */ as m`
- **THEN** the comment is skipped

#### Scenario: Comment before `with` and around parentheses
- **WHEN** encountering `@use "module" with /* comment */ ($key: value)`
- **THEN** the comment is skipped

#### Scenario: Comment before/after `in` keyword in @forward
- **WHEN** encountering `@forward "module" /* comment */ hide $x`
- **THEN** the comment is skipped

#### Scenario: Loud comment (`/* */`) in all critical positions
- **WHEN** a loud comment appears between any two tokens in @use/@forward
- **THEN** the comment is correctly consumed as whitespace

---

### Requirement: CSS property serialization suppresses empty string values

The CSS serializer SHALL omit entire property declarations when the value is an empty string.

#### Scenario: Property with empty string value
- **WHEN** serializing a property with value `""`
- **THEN** the property declaration is omitted from CSS output

#### Scenario: Property with non-empty string value
- **WHEN** serializing a property with value `"foo"`
- **THEN** the property is emitted normally as `key: foo;`

---

### Requirement: List equality distinguishes bracket and paren containers

The list equality comparison SHALL return false when comparing a bracket list with a paren list,
even if their elements are identical.

#### Scenario: Empty bracket list vs empty paren list
- **WHEN** evaluating `[] == ()`
- **THEN** the result is `false`

#### Scenario: Non-empty bracket list vs paren list with same elements
- **WHEN** evaluating `[a, b] == (a, b)`
- **THEN** the result is `false`

#### Scenario: Two bracket lists with same elements
- **WHEN** evaluating `[a, b] == [a, b]`
- **THEN** the result is `true`

---

### Requirement: @import inside rules is hoisted to root level

When `@import` appears inside a rule, it SHALL be hoisted to the root level rather than
producing an error.

#### Scenario: @import inside a style rule
- **WHEN** encountering `a { @import "other"; }`
- **THEN** the import is processed as if it were at the root level

---

### Requirement: @if expressions handle and/or/not error cases correctly

The error reporting for invalid `and`/`or`/`not` usage in `@if` SHALL match sass-spec expected
error messages.

#### Scenario: Invalid `not` combination
- **WHEN** parsing `@if not and` or `@if not or`
- **THEN** the error message matches the spec expectation

#### Scenario: Invalid `and`/`or` raw operators
- **WHEN** parsing `@if raw and or` or similar invalid combinations
- **THEN** appropriate parse error is raised

---

### Requirement: Math functions handle special floating point values

Math functions SHALL correctly handle negative zero, infinity, and NaN per IEEE 754 and
CSS Values Level 4 specification.

#### Scenario: sin(-0.0)
- **WHEN** computing `math.sin(-0.0deg)`
- **THEN** the result is `-0.0` (preserves sign)

#### Scenario: atan2 with signed zeros
- **WHEN** computing `math.atan2(-0, -1)`
- **THEN** the result correctly reflects the quadrant

#### Scenario: pow with negative base and fractional exponent
- **WHEN** computing `math.pow(-2, 0.5)`
- **THEN** the result is NaN
