# Spec Delta

## Purpose

Sass value system supporting all SCSS runtime types needed for Bootstrap 5.3.x and Element-Plus compilation. The Value enum is extended from the basic rx-scss 7-variant starting point with essential Sass types only — no speculative expansion. Each addition is driven by actual compilation requirements from Bootstrap SCSS source.

## ADDED Requirements

### Requirement: Value enum — essential variants only
The Value enum SHALL contain variants for Sass runtime types that actively appear in Bootstrap/Element-Plus compilation: Number (with optional unit), String, Color (with rgb channels + alpha), Bool, Null, List (with separator), Map, and Calc (opaque representation). Advanced color spaces (oklab/lch/etc.) and meta types are deferred to Phase 2.

#### Scenario: Number value creation
- **WHEN** creating `Value::Number(1.0, Some("px".into()))`
- **THEN** the value represents 1px with unit preserved for arithmetic operations

#### Scenario: Color in RGB space
- **WHEN** creating a color with channels `[255.0, 0.0, 0.0]` and alpha 1.0
- **THEN** the value represents opaque red, serializable as `#ff0000`

#### Scenario: Calc expression
- **WHEN** creating `Value::Calc("100% - 20px")`
- **THEN** the value wraps an opaque calc expression that serializes as `calc(100% - 20px)`

### Requirement: Value Display trait
The Display implementation for Value SHALL produce CSS-correct output for each variant. Number SHALL trim trailing zeros (1.0 → "1"), Color SHALL use hex/named-color formatting, List SHALL join with separator, Map SHALL wrap key-value pairs.

#### Scenario: Number formatting
- **WHEN** displaying `Value::Number(1.0, None)` 
- **THEN** output is "1" (no decimal point for integer values)

#### Scenario: List formatting
- **WHEN** displaying `Value::List(vec![Number(1.0,None), Number(2.0,None)], Separator::Space)`
- **THEN** output is "1 2" with space separator

### Requirement: Value equality
Two Value instances SHALL be equal when they represent the same Sass semantic value. Numbers with same value but different display precision SHALL be equal.

#### Scenario: Number equality
- **WHEN** comparing `Number(1.0, None)` with `Number(1.00, None)`
- **THEN** they are equal

#### Scenario: Cross-type inequality
- **WHEN** comparing `Value::Number(1.0, None)` with `Value::String("1".into())`
- **THEN** they are not equal

### Requirement: Value type introspection
The system SHALL support `type-of()`, `unit()`, `is-unitless()` runtime introspection. `type-of(1px)` returns "number", `type-of(red)` returns "color", `type-of((1,2))` returns "list".

#### Scenario: type-of for number
- **WHEN** calling type-of on `Value::Number(42.0, None)`
- **THEN** returns `Value::String("number".into())`

#### Scenario: unit for number with unit
- **WHEN** calling unit on `Value::Number(10.0, Some("em".into()))`
- **THEN** returns `Value::String("em".into())`

### Requirement: Number arithmetic with units
The system SHALL support arithmetic operations on numbers with compatible units. Same-unit operations preserve the unit. CSS unit compatibility rules SHALL be followed (e.g., `10px * 2` → `20px`, `10px + 5px` → `15px`).

#### Scenario: Same-unit addition
- **WHEN** adding 10px + 5px
- **THEN** produces Value::Number(15.0, Some("px"))

#### Scenario: Unit multiplication
- **WHEN** multiplying 10px * 2
- **THEN** produces Value::Number(20.0, Some("px"))

#### Scenario: Incompatible unit error
- **WHEN** adding 10px + 5em
- **THEN** produces a Unit error

### Requirement: Color operations
The system SHALL support Sass color functions: rgb, rgba, hsl, hsl, red, green, blue, alpha, mix, lighten, darken, saturate, desaturate, adjust-hue, grayscale, complement, invert. Colors are stored as RGBA with alpha channel.

#### Scenario: darken operation
- **WHEN** calling `darken(#ff0000, 20%)`
- **THEN** returns a darker red color value

#### Scenario: mix operation
- **WHEN** calling `mix(red, blue, 50%)`
- **THEN** returns a purple color

### Requirement: List and Map operations
Lists SHALL support: length, nth, set-nth, join, append, index, separator. Maps SHALL support: get, merge, keys, values, has-key, remove.

#### Scenario: list.join
- **WHEN** calling `list.join((1, 2), (3, 4))`
- **THEN** returns Value::List([1, 2, 3, 4], Comma)

#### Scenario: map.get
- **WHEN** calling `map.get((primary: blue), primary)`
- **THEN** returns the color blue
