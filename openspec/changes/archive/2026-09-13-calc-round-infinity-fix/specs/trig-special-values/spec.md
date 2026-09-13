## ADDED Requirements

### Requirement: trig functions accept infinity/NaN as input
The trigonometric functions (sin, cos, tan, asin, acos, atan) SHALL accept `infinity`, `-infinity`, and `NaN` as valid numeric arguments.

#### Scenario: sin(infinity)
- **WHEN** `sin(infinity)` is evaluated
- **THEN** the result is `NaN` (CSS spec: sin of infinity is undefined)

#### Scenario: sin(-infinity)
- **WHEN** `sin(-infinity)` is evaluated
- **THEN** the result is `NaN`

#### Scenario: sin(NaN)
- **WHEN** `sin(NaN)` is evaluated
- **THEN** the result is `NaN`

#### Scenario: cos(infinity)
- **WHEN** `cos(infinity)` is evaluated
- **THEN** the result is `NaN`

#### Scenario: cos(-infinity)
- **WHEN** `cos(-infinity)` is evaluated
- **THEN** the result is `NaN`

#### Scenario: asin(infinity)
- **WHEN** `asin(infinity)` is evaluated
- **THEN** the result is an error or NaN (domain error)

#### Scenario: acos(infinity)
- **WHEN** `acos(infinity)` is evaluated
- **THEN** the result is an error or NaN (domain error)

#### Scenario: tan(infinity)
- **WHEN** `tan(infinity)` is evaluated
- **THEN** the result is `NaN`

### Requirement: validate_single_number accepts string special values
The `validate_single_number` helper SHALL recognize string values `"infinity"`, `"-infinity"`, and `"nan"` as valid numeric arguments.

#### Scenario: validate passes for "infinity" string
- **WHEN** `validate_single_number(&[Value::String("infinity".into(), false)])` is called
- **THEN** the result is `Ok(())`

#### Scenario: validate passes for "-infinity" string
- **WHEN** `validate_single_number(&[Value::String("-infinity".into(), false)])` is called
- **THEN** the result is `Ok(())`

#### Scenario: validate passes for "NaN" string
- **WHEN** `validate_single_number(&[Value::String("NaN".into(), false)])` is called
- **THEN** the result is `Ok(())`

#### Scenario: validate still rejects non-numeric strings
- **WHEN** `validate_single_number(&[Value::String("hello".into(), false)])` is called
- **THEN** the result is `Err` with "is not a number" error
