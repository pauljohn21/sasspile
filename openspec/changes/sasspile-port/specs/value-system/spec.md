# Spec Delta

## Purpose

Extended Value type system — the runtime representation of Sass values during evaluation. Starting from the basic 7-variant rx-scss Value enum, extend only with types that are actually needed for Bootstrap compilation. No speculative type expansion.

## ADDED Requirements

### Requirement: Extended Value type
The Value enum SHALL be expanded from the basic rx-scss variants to cover essential Sass runtime types: Number (with optional unit), String, Color (rgb + alpha), Bool, Null, List (with separator), Map, and Calc (opaque). Each extension is motivated by actual Bootstrap SCSS requirements.

#### Scenario: Calc value
- **WHEN** evaluating `calc(100% - 20px)`
- **THEN** produces a Value::Calc wrapping an opaque representation that serializes correctly

#### Scenario: Color with rgb
- **WHEN** evaluating `#ff0000` or `rgb(255, 0, 0)`
- **THEN** produces a Value::Color with channels [255, 0, 0] and alpha 1.0

### Requirement: Value Display correctness
Display implementations SHALL produce CSS-correct formatting including: integer-trimming for numbers (`1.0` → `"1"`), hex color formatting, separator-aware list formatting, and proper parenthesization of maps.

#### Scenario: Number display
- **WHEN** displaying Value::Number(1.0, None)
- **THEN** produces "1" (no trailing .0)

#### Scenario: Number with decimal
- **WHEN** displaying Value::Number(1.5, None)
- **THEN** produces "1.5"

### Requirement: Value arithmetic with units
The system SHALL support arithmetic operations on numbers with compatible units. Same-unit operations preserve the unit. Unit conversion SHALL follow CSS spec (e.g., `10px * 2` → `20px`).

#### Scenario: Same-unit addition
- **WHEN** adding 10px + 5px
- **THEN** produces Value::Number(15.0, Some("px"))

#### Scenario: Unit multiplication
- **WHEN** multiplying 10px * 2
- **THEN** produces Value::Number(20.0, Some("px"))

#### Scenario: Incompatible unit error
- **WHEN** adding 10px + 5em
- **THEN** produces a Unit error

### Requirement: List and Map value semantics
Lists SHALL have a separator (Comma/Space/Slash). Maps SHALL have ordered key-value pairs. Both SHALL support `list.join`, `map.get` and other Sass operations.

#### Scenario: Comma list display
- **WHEN** displaying Value::List([1,2,3], Comma)
- **THEN** produces "1, 2, 3"

#### Scenario: Map access
- **WHEN** calling map.get on a map with key "primary"
- **THEN** returns the associated value
