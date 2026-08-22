## MODIFIED Requirements

### Requirement: math 函数命名参数支持
系统 SHALL 在所有 math 内建函数中支持命名参数传递，合并位置参数和命名参数后进行验证。系统 SHALL 对所有 math 函数的参数数量、类型和单位进行严格验证，对不足参数报 "Missing argument $<name>." 错误，对过多参数报 "Only N argument(s) allowed, but M were passed." 错误，对非数字参数报 "$<name>: <value> is not a number." 错误，对带单位参数报 "$<name>: Expected <value> to have no units." 错误。

#### Scenario: atan2 命名参数
- **WHEN** 调用 `math.atan2($y: 1, $x: 2)`
- **THEN** 系统 正确计算 atan2(1, 2) 的值

#### Scenario: sin 命名参数
- **WHEN** 调用 `math.sin($number: 0)`
- **THEN** 系统 返回 0

#### Scenario: pow 命名参数
- **WHEN** 调用 `math.pow($base: 2, $exponent: 3)`
- **THEN** 系统 返回 8

#### Scenario: abs 参数不足
- **WHEN** 调用 `abs()` 无参数
- **THEN** 系统 报 "Missing argument $number." 错误

#### Scenario: abs 参数过多
- **WHEN** 调用 `abs(1, 2)`
- **THEN** 系统 报 "Only 1 argument allowed, but 2 were passed." 错误

#### Scenario: abs 非数字
- **WHEN** 调用 `abs("0")` 参数不是数字
- **THEN** 系统 报 "$number: \"0\" is not a number." 错误
