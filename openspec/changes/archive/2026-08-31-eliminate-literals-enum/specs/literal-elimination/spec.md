## ADDED Requirements

### Requirement: Numeric constants SHALL be defined in consts.rs
The system SHALL centralize all magic number literals (255.0, 0.0001, 360.0, 1e-10, etc.) into a `consts.rs` module with semantically named constants. Source files SHALL reference these constants instead of inline numeric literals.

#### Scenario: RGB_MAX constant
- **WHEN** a color operation needs the maximum RGB value
- **THEN** it SHALL reference `consts::RGB_MAX` (255.0) instead of the literal `255.0`

#### Scenario: ALPHA_TOLERANCE constant
- **WHEN** comparing alpha values for equality
- **THEN** it SHALL reference `consts::ALPHA_TOLERANCE` (0.0001) instead of the literal `0.0001`

### Requirement: Error message templates SHALL be centralized in error_msgs.rs
The system SHALL define error message builder functions in `error_msgs.rs`. All `SassError::Eval(format!("... is not a string."))` patterns SHALL be replaced with calls like `error_msgs::err_not_a_string("$string", &value)`.

#### Scenario: Not-a-string error
- **WHEN** a function receives a non-string Value for a `$string` parameter
- **THEN** it SHALL call `error_msgs::err_not_a_string("$string", &value)` instead of inline `format!("$string: {} is not a string.", value)`

#### Scenario: Wrong argument count error
- **WHEN** a function receives the wrong number of arguments
- **THEN** it SHALL call `error_msgs::err_wrong_arg_count(expected, actual, is_singular)` instead of inline formatting

### Requirement: Named colors SHALL use a single data source
The system SHALL maintain a single `const NAMED_COLORS` array in `named_colors.rs` for both forward lookup (name→RGB) and reverse lookup (RGB→name). The duplicate `reverse_lookup_named_color` table SHALL be removed.

#### Scenario: Forward lookup
- **WHEN** `named_colors::lookup("red")` is called
- **THEN** it SHALL return `Some((255, 0, 0))`

#### Scenario: Reverse lookup
- **WHEN** `named_colors::reverse_lookup(255.0, 0.0, 0.0)` is called
- **THEN** it SHALL return `Some("red")` using the same data array as forward lookup

### Requirement: AtRuleKind enum SHALL replace string matching in parser
The system SHALL define an `AtRuleKind` enum with variants for all Sass at-rules (If, For, Each, While, Mixin, Include, Content, Function, Return, Use, Forward, Import, Extend, AtRoot, Warn, Debug, Error, Other). The `parse_at_rule` function SHALL parse the string name into `AtRuleKind` once and match on the enum.

#### Scenario: Parse if rule
- **WHEN** the parser encounters `@if`
- **THEN** it SHALL parse the name "if" into `AtRuleKind::If` and dispatch to `parse_if()`

#### Scenario: Unknown at-rule
- **WHEN** the parser encounters `@unknown-rule`
- **THEN** it SHALL parse into `AtRuleKind::Other("unknown-rule")` and dispatch to `parse_generic_at_rule()`

### Requirement: CssAtRule enum SHALL validate plain CSS at-rules
The system SHALL define a `CssAtRule` enum for standard CSS at-rules (Media, Supports, Container, Import, Charset, Page, FontFace, FontFeatureValues, Keyframes, Layer, Scope, StartingStyle, PositionTry, Property, Namespace, Document, Other). The `check_plain_css_node` function SHALL use this enum instead of a `&[&str]` array.

#### Scenario: Validate media rule in plain CSS
- **WHEN** `check_plain_css_node` checks a `@media` rule in plain CSS mode
- **THEN** it SHALL match `CssAtRule::Media` and return `Ok(())`

#### Scenario: Reject sass rule in plain CSS
- **WHEN** `check_plain_css_node` checks an `@if` rule in plain CSS mode
- **THEN** it SHALL not match any `CssAtRule` variant and return `Err`

### Requirement: Builtin function names SHALL use const arrays
The system SHALL define builtin function name lists as `const` arrays in `names.rs` instead of inline `matches!` macro calls with repeated string literals. The `*_is_known` and `*_dispatch` functions SHALL reference these const arrays.

#### Scenario: Math function name lookup
- **WHEN** `math_is_known("abs")` is called
- **THEN** it SHALL check a `const MATH_NAMES: &[&str]` array instead of an inline `matches!` with 26+ string literals

### Requirement: No inline string literals in match arms
After refactoring, source files SHALL NOT contain inline string literals in `match` arms for known category comparisons (color spaces, channels, at-rules, builtin names). All such comparisons SHALL use enum matching or const array lookups.

#### Scenario: Color space match arm
- **WHEN** inspecting `color_space.rs` match arms
- **THEN** they SHALL match on `ColorSpace::Hsl` etc., not `"hsl"`
