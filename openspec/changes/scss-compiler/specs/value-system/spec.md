## ADDED Requirements

### Requirement: Value System SHALL represent all Sass value types
The system SHALL model: `Number(value, unit)`, `String(quoted)`, `Boolean`, `Null`, `Color(r,g,b,a)`, `List(sep)`, `Map`, `ArgList`, `Function(ref)`, `Calculation`, `Error`.

#### Scenario: Number value
- **WHEN** constructing `Number(16, Some(Px))`
- **THEN** value carries numeric 16 and unit Px with full equality semantics

#### Scenario: Color value
- **WHEN** constructing `Color(255, 0, 0, 1.0)` (red)
- **THEN** value is equal to named color `red` after canonicalization

#### Scenario: List with separator
- **WHEN** constructing `List(vec![1.px(), 2.px()], Comma)`
- **THEN** list preserves comma vs space separator for CSS output

### Requirement: Value System SHALL implement correct equality
Two values SHALL be equal based on type-specific rules (number unitless-vs-unit numeric equality is NOT allowed; string quoted vs unquoted comparison rules).

#### Scenario: Number equality
- **WHEN** comparing `Number(1, None)` with `Number(1, None)`
- **THEN** result is `true`

#### Scenario: String semantic equality
- **WHEN** comparing `String("foo", Quoted)` with `String("foo", Unquoted)`
- **THEN** result is `true` (semantically equivalent when stringified)

### Requirement: Value System SHALL support type conversion
Values SHALL implement `to_bool()`, `to_string()`, `to_number()`, `to_list()` following Sass coercion rules.

#### Scenario: Truthiness
- **WHEN** converting `Boolean(false)`, `Null`, `Number(0, None)`, `String("", Quoted)` to bool
- **THEN** all yield `false` except empty string which yields `true` in Sass semantics

#### Scenario: Single value to list
- **WHEN** converting `Number(42, None)` to list
- **THEN** result is single-element list (not already a list)

### Requirement: Value System SHALL support arithmetic coercion
Numbers SHALL support unit conversion (e.g. `1in = 96px`) and complex unit tracking per CSS Values 4.

#### Scenario: Compatible unit addition
- **WHEN** adding `Number(1, Some(In))` with `Number(96, Some(Px))`
- **THEN** result is `Number(2, Some(In))` after conversion

#### Scenario: Incompatible unit error
- **WHEN** adding `Number(1, Some(Px))` with `Number(1, Some(Em))`
- **THEN** operation produces error "Incompatible units"

### Requirement: Value System SHALL serialize to CSS
Each value SHALL implement correct CSS serialization for stylesheets.

#### Scenario: Number serialization
- **WHEN** serializing `Number(0.5, None)`
- **THEN** output is `"0.5"` and `Number(0, None)` renders as `"0"`

#### Scenario: Color serialization
- **WHEN** serializing `Color(255, 0, 0, 1.0)`
- **THEN** output is `"#ff0000"` when shorter than named form

#### Scenario: List serialization with separator
- **WHEN** serializing comma-separated list `1px 2px, 3px 4px`
- **THEN** output preserves comma: `"1px 2px, 3px 4px"`
