## ADDED Requirements

### Requirement: 命名参数不计为位置参数

内建函数的参数验证 SHALL 基于合并后的位置参数数量，不 SHALL 将命名参数重复计为多余的位置参数。

#### Scenario: math.abs 接受命名参数

- **WHEN** 调用 `math.abs($number: 1)`
- **THEN** 返回 `1`，不报 "Only 1 argument allowed, but 2 were passed"

#### Scenario: str-length 接受命名参数

- **WHEN** 调用 `str-length($string: "hello")`
- **THEN** 返回 `5`，不报参数数量错误

#### Scenario: abs 接受 1 位置参数 + 0 命名参数

- **WHEN** 调用 `abs(1)`
- **THEN** 返回 `1`

### Requirement: if 函数接受正好 3 参数

`if($condition, $if-true, $if-false)` SHALL 接受 3 个参数（位置或命名），不 SHALL 报 "requires 3 arguments" 当 3 个参数通过命名方式传入时。

#### Scenario: if 三个位置参数

- **WHEN** 调用 `if(true, 1, 2)`
- **THEN** 返回 `1`

#### Scenario: if 混合参数

- **WHEN** 调用 `if(true, $if-true: 1, $if-false: 2)`
- **THEN** 返回 `1`

### Requirement: rgba 接受 3-4 number 参数

`rgba()` SHALL 接受 3 个 number 参数（RGB）或 4 个 number 参数（RGBA），不 SHALL 误报参数数量。

#### Scenario: rgba 3 参数

- **WHEN** 调用 `rgba(255, 0, 0)`
- **THEN** 返回 `#ff0000`

#### Scenario: rgba 4 参数

- **WHEN** 调用 `rgba(255, 0, 0, 0.5)`
- **THEN** 返回 `rgba(255, 0, 0, 0.5)`

### Requirement: 字符串到数字的隐式转换

当字符串参数可以解析为数字时，内建函数 SHALL 接受字符串并自动转换为数字。

#### Scenario: math.abs 接受数字字符串

- **WHEN** 调用 `math.abs("0")`
- **THEN** 返回 `0`

#### Scenario: math.abs 接受负数字符串

- **WHEN** 调用 `math.abs("-5")`
- **THEN** 返回 `5`
