## Purpose

defines sasspile compiler specs for special floating-point constants (infinity, -infinity, NaN) handling in math functions.

## Requirements

### Requirement: extract_unitless 接受特殊浮点常量
`extract_unitless()` 函数 SHALL 接受 `infinity`、`-infinity`、`NaN` 作为合法数值输入。

#### Scenario: infinity 作为 base
- **WHEN** 调用 `pow(2, infinity)`
- **THEN** 返回 `infinity`（即 f64::INFINITY 的格式化输出）

#### Scenario: -infinity 参数
- **WHEN** 调用 `atan(-infinity)`
- **THEN** 返回 `-90deg`

#### Scenario: NaN 参数
- **WHEN** 调用 `asin(NaN)`
- **THEN** 返回 `NaN`

### Requirement: 各 math 函数特殊值传播
所有调用 `extract_unitless` 的 math 函数 SHALL 正确传播特殊浮点值结果。

#### Scenario: sqrt(infinity)
- **WHEN** 调用 `sqrt(infinity)`
- **THEN** 返回 `infinity`

#### Scenario: log(infinity)
- **WHEN** 调用 `log(infinity)`
- **THEN** 返回 `infinity`

#### Scenario: asin(infinity)（超出定义域）
- **WHEN** 调用 `asin(infinity)`
- **THEN** 返回 `NaN`

#### Scenario: round 的 infinity 处理
- **WHEN** 调用 `round(infinity, 1)`
- **THEN** 返回 `infinity`

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
- **WHEN** `validate_single_number` receives `Value::String("infinity", false)`
- **THEN** the result is `Ok(())`

#### Scenario: validate passes for "-infinity" string
- **WHEN** `validate_single_number` receives `Value::String("-infinity", false)`
- **THEN** the result is `Ok(())`

#### Scenario: validate passes for "NaN" string
- **WHEN** `validate_single_number` receives `Value::String("NaN", false)`
- **THEN** the result is `Ok(())`

#### Scenario: validate still rejects non-numeric strings
- **WHEN** `validate_single_number` receives `Value::String("hello", false)`
- **THEN** the result is `Err` with "is not a number" error

### Requirement: atan/asin/acos 遇变量时保留
`atan()`、`asin()`、`acos()` 等反三角函数的参数包含 Sass 变量时，SHALL 保留函数形式不编译时求值。

#### Scenario: atan 含变量保留
- **WHEN** 输入为 `atan($var)`
- **THEN** 输出为 `atan($var)` 保留形式

#### Scenario: atan 含 calc+变量
- **WHEN** 输入为 `atan(3px - 1px + var(--c))`
- **THEN** 输出为 `atan(2px + var(--c))`（简化已知量但保留整体函数）
