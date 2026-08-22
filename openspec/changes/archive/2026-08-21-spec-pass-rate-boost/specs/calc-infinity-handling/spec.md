## ADDED Requirements

### Requirement: calc(infinity) 在 pow 函数中的处理
系统 SHALL 在 `pow` 函数中识别 `calc(infinity)` 和 `calc(-infinity)` 作为 base 或 exponent 参数，并返回正确的 Calc 值。

#### Scenario: pow(infinity, positive)
- **WHEN** 调用 `pow(calc(infinity), 2)`
- **THEN** 系统 返回 `calc(infinity)`

#### Scenario: pow(infinity, 0)
- **WHEN** 调用 `pow(calc(infinity), 0)`
- **THEN** 系统 返回 `1`

#### Scenario: pow(0, infinity)
- **WHEN** 调用 `pow(0, calc(infinity))`
- **THEN** 系统 返回 `0`

#### Scenario: pow(infinity, infinity)
- **WHEN** 调用 `pow(calc(infinity), calc(infinity))`
- **THEN** 系统 返回 `calc(infinity)`

#### Scenario: pow(-infinity, even)
- **WHEN** 调用 `pow(calc(-infinity), 2)`
- **THEN** 系统 返回 `calc(infinity)`

#### Scenario: pow(-infinity, odd)
- **WHEN** 调用 `pow(calc(-infinity), 3)`
- **THEN** 系统 返回 `calc(-infinity)`

### Requirement: calc(infinity) 在 div 函数中的处理
系统 SHALL 在 `div` 函数中识别 `calc(infinity)` 作为参数并返回正确的 Calc 值。

#### Scenario: div(infinity, number)
- **WHEN** 调用 `math.div(calc(infinity), 2)`
- **THEN** 系统 返回 `calc(infinity)`

#### Scenario: div(number, infinity)
- **WHEN** 调用 `math.div(1, calc(infinity))`
- **THEN** 系统 返回 `0`

### Requirement: calc(infinity) 在 sqrt 函数中的处理
系统 SHALL 在 `sqrt` 函数中识别 `calc(infinity)` 参数。

#### Scenario: sqrt(infinity)
- **WHEN** 调用 `sqrt(calc(infinity))`
- **THEN** 系统 返回 `calc(infinity)`

### Requirement: infinity/nan 序列化
系统 SHALL 将 `infinity`、`-infinity`、`NaN` 特殊数值序列化为 CSS 兼容格式。

#### Scenario: infinity 序列化
- **WHEN** 值为 `infinity`（无单位）
- **THEN** CSS 输出为 `infinity`

#### Scenario: 负 infinity 序列化
- **WHEN** 值为 `-infinity`
- **THEN** CSS 输出为 `-infinity`

#### Scenario: NaN 序列化
- **WHEN** 值为 `NaN`
- **THEN** CSS 输出为 `NaN`

#### Scenario: infinity 带单位序列化
- **WHEN** 值为 `infinity` 带单位 `px`
- **THEN** CSS 输出为 `calc(infinity * 1px)`
