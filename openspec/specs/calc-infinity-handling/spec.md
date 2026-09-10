## Purpose

defines sasspile compiler specs.

## Requirements



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

### Requirement: calc() 输出特殊常量大小写规范化
`calc()` 表达式中的 `infinity`、`-infinity`、`NaN` 关键字 SHALL 在输出时统一为规范大小写。

#### Scenario: infinity 大小写规范化
- **WHEN** 输入为 `calc(InFiNiTy)` 或 `calc(INFINITY)`
- **THEN** 输出为 `calc(infinity)`（统一小写）

#### Scenario: minus_infinity 规范化
- **WHEN** 输入为 `calc(-InFiNiTy)`
- **THEN** 输出为 `calc(-infinity)`

#### Scenario: NaN 大小写规范化
- **WHEN** 输入为 `calc(nan)` 或 `calc(NAN)`
- **THEN** 输出为 `calc(NaN)`（N 大写）

### Requirement: calc() 除法特殊值简化
`calc()` 中 `1/0`、`-1/0`、`0/0` 等除法特殊表达式 SHALL 在输出中简化为对应常量。

#### Scenario: 1/0 简化为 infinity
- **WHEN** 输入为 `calc((1/0) * (1% + 1px))`
- **THEN** 输出为 `calc(infinity * (1% + 1px))`

#### Scenario: -1/0 简化为 -infinity
- **WHEN** 输入为 `calc((-1/0) * 2)`
- **THEN** 输出为 `calc(-infinity * 2)`

#### Scenario: 0/0 简化为 NaN
- **WHEN** 输入为 `calc((0/0) + 1px)`
- **THEN** 输出为 `calc(NaN + 1px)`

### Requirement: calc() 特殊常量简化规则
`calc()` 表达式中的特殊常量参与运算时 SHALL 按 IEEE 754 规则简化。

#### Scenario: infinity 乘有限数
- **WHEN** 输入为 `calc(infinity * 2)`
- **THEN** 输出为 `calc(infinity)`

#### Scenario: infinity 乘零（保留不简化）
- **WHEN** 输入为 `calc(infinity * 0)`
- **THEN** 输出为 `calc(NaN)` 或保留原表达式

#### Scenario: NaN 传播
- **WHEN** 输入为 `calc(NaN * 2)`
- **THEN** 输出为 `calc(NaN)`

### Requirement: type-of(calc(特殊常量)) 类型
`calc()` 表达式包含特殊常量时，`meta.type-of()` SHALL 返回 `number`。

#### Scenario: type-of(calc(infinity))
- **WHEN** 调用 `meta.type-of(calc(infinity))`
- **THEN** 返回 `number`

#### Scenario: type-of(calc(NaN))
- **WHEN** 调用 `meta.type-of(calc(NaN))`
- **THEN** 返回 `number`
