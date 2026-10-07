# Spec Delta

## Purpose

Comprehensive library of ~50 Sass builtin functions organized into 7 modules (math, string, list, map, color, selector, meta). Provides all functions needed for Bootstrap 5.3.x and Element-Plus full compilation. Implementation ported from sasspile's `eval/builtin/` algorithms to rxrust-reactive style.

## ADDED Requirements

### Requirement: Math functions
The system SHALL provide math module functions: `math.abs`, `math.div`, `math.ceil`, `math.floor`, `math.round`, `math.max`, `math.min`, `math.percentage`, `math.pow`, `math.sqrt`, `math.sin`, `math.cos`, `math.tan`, `math.asin`, `math.acos`, `math.atan`, `math.hypot`, `math.atan2`, `math.log`, `math.random`, `math.clamp`, `math.mod`, `math.rem`, `math.compatible`, `math.comparable`, `math.unit`, `math.is-unitless`. Angle units (deg/rad/grad/turn) SHALL be correctly converted for trig functions.

#### Scenario: math.percentage
- **WHEN** calling `math.percentage(0.5)`
- **THEN** returns Value::Number(50.0, Some("%"))

#### Scenario: math.sin with degrees
- **WHEN** calling `math.sin(90deg)`
- **THEN** returns Value::Number(1.0, None)

#### Scenario: math.div
- **WHEN** calling `math.div(10px, 2px)`
- **THEN** returns Value::Number(5.0, None) (unit cancellation)

### Requirement: String functions
The system SHALL provide string module functions: `string.length`, `string.index` (str-index), `string.slice` (str-slice), `string.insert` (str-insert), `string.split` (str-split), `string.to-upper-case`, `string.to-lower-case`, `string.quote`, `string.unquote`, `string.unique-id`.

#### Scenario: string.quote
- **WHEN** calling `string.quote(hello)`
- **THEN** returns Value::String('"hello"') (quoted)

#### Scenario: string.split
- **WHEN** calling `string.split("a,b,c", ",")`
- **THEN** returns Value::List(["a", "b", "c"], Comma)

### Requirement: List functions
The system SHALL provide list module functions: `list.length`, `list.nth`, `list.set-nth`, `list.join`, `list.append`, `list.zip`, `list.index`, `list.separator`, `list.is-bracketed`, `list.slash` (deprecated).

#### Scenario: list.join
- **WHEN** calling `list.join(1 2 3, 4 5 6)`
- **THEN** returns Value::List([1,2,3,4,5,6], Space)

#### Scenario: list.nth
- **WHEN** calling `list.nth(red green blue, 2)`
- **THEN** returns Value::Color (green)

### Requirement: Map functions
The system SHALL provide map module functions: `map.get` (map-get), `map.merge` (map-merge), `map.keys`, `map.values`, `map.has-key`, `map.remove`, `map.deep-merge`, `map.deep-get`.

#### Scenario: map.get
- **WHEN** calling `map.get((primary: blue), primary)`
- **THEN** returns Value::Color (blue)

### Requirement: Color functions
The system SHALL provide color module functions: `rgb`, `rgba`, `hsl`, `hsla`, `red`, `green`, `blue`, `hue`, `saturation`, `lightness`, `alpha`, `adjust-hue`, `lighten`, `darken`, `saturate`, `desaturate`, `grayscale`, `complement`, `invert`, `mix`, `opacify` (fade-in), `transparentize` (fade-out), `adjust-color`, `scale-color`, `change-color`, `ie-hex-str`, plus modern space functions `color`, `color-mix`, `color-contrast`, `hwb`, `lab`, `lch`, `oklab`, `oklch`, `to-space`, `in-space`, `is-legacy`, `is-in-gamut`, `gamut-map`, `channel`, `same`, `hue`, `blackness`, `whiteness`.

#### Scenario: darken
- **WHEN** calling `darken(red, 20%)`
- **THEN** returns a darker red color

#### Scenario: mix
- **WHEN** calling `mix(red, blue, 50%)`
- **THEN** returns a purple color (midpoint mix)

### Requirement: Selector functions
The system SHALL provide selector module functions: `selector.append`, `selector.extend`, `selector.nest`, `selector.parse`, `selector.replace`, `selector.simple-selectors`, `selector.is-superselector`, `selector.unify`, `selector.is-parent-selector`.

#### Scenario: selector.nest
- **WHEN** calling `selector.nest(".a", ".b")`
- **THEN** returns Value::String(".a .b")

### Requirement: Meta functions
The system SHALL provide meta module functions: `meta.global-variable-exists`, `meta.variable-exists`, `meta.mixin-exists`, `meta.function-exists`, `meta.call`, `meta.get-function`, `meta.get-mixin`, `meta.content-exists`, `meta.module-functions`, `meta.module-mixins`, `meta.module-variables`, `meta.type-of`, `meta.inspect`, `meta.calc-args`, `meta.calc-name`, `meta.keywords`.

#### Scenario: meta.type-of
- **WHEN** calling `meta.type-of(42px)`
- **THEN** returns Value::String("number")

#### Scenario: meta.module-functions
- **WHEN** calling `meta.module-functions("sass:math")`
- **THEN** returns a list of math module function names

### Requirement: `if()` function
The system SHALL provide the special `if(condition, if-true, if-false)` function that returns one of two values based on condition truthiness. This is NOT the same as `@if` directive.

#### Scenario: if true
- **WHEN** calling `if(true, yes, no)`
- **THEN** returns Value::String("yes")

### Requirement: `inspect()` function
The system SHALL provide `inspect(value)` that returns a human-readable string representation of any Sass value (used for debugging and error messages).

#### Scenario: inspect number
- **WHEN** calling `inspect(42px)`
- **THEN** returns Value::String("42px")
