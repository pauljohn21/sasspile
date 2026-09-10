## ADDED Requirements

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
