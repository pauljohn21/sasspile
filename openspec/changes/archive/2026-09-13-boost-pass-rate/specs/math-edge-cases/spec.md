## ADDED Requirements

### Requirement: math.pow edge cases
`math.pow()` SHALL handle negative base with fractional exponent by returning NaN.

#### Scenario: Negative base with half exponent
- **WHEN** `math.pow(-2, 0.5)` is evaluated
- **THEN** result is NaN (or "nan" string)

### Requirement: math.pow large exponent
`math.pow()` SHALL handle overflow by returning Infinity.

#### Scenario: Overflow exponent
- **WHEN** `math.pow(10, 309)` is evaluated
- **THEN** result is "infinity"

### Requirement: math.atan2 zero handling
`math.atan2()` SHALL follow signed zero rules: `atan2(0, -1)` returns π, `atan2(-0, -1)` returns -π.

#### Scenario: Positive zero y
- **WHEN** `math.atan2(0, -1)` is evaluated
- **THEN** result is 3.1415926536 (π radians)

#### Scenario: Negative zero y
- **WHEN** `math.atan2(-0.0, -1)` is evaluated
- **THEN** result is -3.1415926536 (-π radians)

### Requirement: math.clamp unit preservation
`math.clamp()` SHALL preserve the unit of the input value.

#### Scenario: Clamp with px unit
- **WHEN** `math.clamp(10px, 0px, 5px)` is evaluated
- **THEN** result is `5px` (unit preserved from input)

### Requirement: math.unit extraction
`math.unit()` SHALL return unit string for compound units.

#### Scenario: Compound unit
- **WHEN** `math.unit(1px * 1s)` is evaluated
- **THEN** result is `"px*s"`
