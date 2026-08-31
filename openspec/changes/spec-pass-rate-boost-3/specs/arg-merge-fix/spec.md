## ADDED Requirements

### Requirement: 命名参数不计入位置参数计数

内建函数的参数验证 SHALL 区分位置参数（pos_args）和命名参数（kw_args）。当调用 `str-length($string: "hello")` 时，系统 SHALL 仅将 1 个位置参数计入长度检查，命名参数 `$string` 不应导致 "Only 1 argument allowed, but 2 were passed" 错误。

#### Scenario: str-length 命名参数调用
- **WHEN** 调用 `str-length($string: "hello")` 使用命名参数
- **THEN** 系统 SHALL 正确返回字符串长度 5，不报 "Only 1 argument allowed" 错误

#### Scenario: to-upper-case 命名参数调用
- **WHEN** 调用 `to-upper-case($string: "hello")` 使用命名参数
- **THEN** 系统 SHALL 正确返回 `"HELLO"`，不报参数过多错误

#### Scenario: abs 命名参数调用
- **WHEN** 调用 `abs($number: -5)` 使用命名参数
- **THEN** 系统 SHALL 正确返回 5，不报 "Only 1 argument allowed" 错误

#### Scenario: quote 命名参数调用
- **WHEN** 调用 `quote($string: hello)` 使用命名参数
- **THEN** 系统 SHALL 正确返回 `"hello"`，不报参数过多错误

#### Scenario: unquote 命名参数调用
- **WHEN** 调用 `unquote($string: "hello")` 使用命名参数
- **THEN** 系统 SHALL 正确返回 `hello`，不报参数过多错误

### Requirement: merge_args 统一入口修复

`merge_args` 函数 SHALL 在合并位置参数和命名参数后，仅使用位置参数（pos_args）的长度进行参数计数验证。命名参数通过参数名映射填充到对应位置，不计入多余参数计数。

#### Scenario: 单参数函数命名参数
- **WHEN** 任何接受单参数的内建函数以 `func($param: value)` 形式调用
- **THEN** 系统 SHALL 将命名参数映射到参数位置，不报 "Only 1 argument allowed, but 2 were passed" 错误

#### Scenario: 双参数函数命名参数
- **WHEN** 任何接受双参数的内建函数以 `func($param1: v1, $param2: v2)` 形式调用
- **THEN** 系统 SHALL 将命名参数映射到参数位置，不报 "Only 2 arguments allowed, but 3 were passed" 错误

#### Scenario: 混合位置和命名参数
- **WHEN** 调用 `str-index("hello", $substring: "ll")` 混合位置和命名参数
- **THEN** 系统 SHALL 正确处理参数，不报参数过多错误
