# math-param-validation Specification

## Purpose
TBD - created by archiving change spec-pass-rate-boost-2. Update Purpose after archive.
## Requirements
### Requirement: clamp 参数验证
系统 SHALL 验证 `clamp($min, $number, $max)` 的参数类型和数量。

#### Scenario: clamp 参数不足
- **WHEN** 调用 `clamp(1px, 2px)` 缺少第三个参数
- **THEN** 系统 报 "Missing argument $max." 错误

#### Scenario: clamp 参数过多
- **WHEN** 调用 `clamp(1px, 2px, 3px, 4px)` 多于三个参数
- **THEN** 系统 报 "Only 3 arguments allowed, but 4 were passed." 错误

#### Scenario: clamp 非数字参数
- **WHEN** 调用 `clamp("0", 1px, 2px)` 第一个参数不是数字
- **THEN** 系统 报 "$min: \"0\" is not a number." 错误

### Requirement: min/max 参数验证
系统 SHALL 验证 `min(...)` 和 `max(...)` 的参数类型和数量。

#### Scenario: min 无参数
- **WHEN** 调用 `min()` 无参数
- **THEN** 系统 报 "min requires at least 1 argument" 错误

#### Scenario: min 非数字参数
- **WHEN** 调用 `min(1px, "foo")` 第二个参数不是数字
- **THEN** 系统 报 "min requires number arguments" 错误

#### Scenario: max 无参数
- **WHEN** 调用 `max()` 无参数
- **THEN** 系统 报 "max requires at least 1 argument" 错误

### Requirement: pow 参数验证
系统 SHALL 验证 `pow($base, $exponent)` 的参数类型和单位。

#### Scenario: pow 非数字参数
- **WHEN** 调用 `pow("0", 2)` 第一个参数不是数字
- **THEN** 系统 报 "$base: \"0\" is not a number." 错误

#### Scenario: pow 带单位参数
- **WHEN** 调用 `pow(1px, 2px)` 参数带单位
- **THEN** 系统 报 "$base: Expected 1px to have no units." 错误

#### Scenario: pow 参数不足
- **WHEN** 调用 `pow(2)` 缺少第二个参数
- **THEN** 系统 报 "Missing argument $exponent." 错误

### Requirement: hypot 参数验证
系统 SHALL 验证 `hypot(...)` 的参数类型。

#### Scenario: hypot 非数字参数
- **WHEN** 调用 `hypot("0", 1)` 第一个参数不是数字
- **THEN** 系统 报 "$number: \"0\" is not a number." 错误

### Requirement: log 参数验证
系统 SHALL 验证 `log($number, $base: null)` 的参数类型。

#### Scenario: log 非数字参数
- **WHEN** 调用 `log("0")` 参数不是数字
- **THEN** 系统 报 "$number: \"0\" is not a number." 错误

### Requirement: abs/ceil/floor/round 参数验证
系统 SHALL 验证单参数 math 函数的参数数量和类型。

#### Scenario: abs 参数过多
- **WHEN** 调用 `abs(1, 2)`
- **THEN** 系统 报 "Only 1 argument allowed, but 2 were passed." 错误

#### Scenario: abs 参数不足
- **WHEN** 调用 `abs()` 无参数
- **THEN** 系统 报 "Missing argument $number." 错误

#### Scenario: abs 非数字参数
- **WHEN** 调用 `abs("0")` 参数不是数字
- **THEN** 系统 报 "$number: \"0\" is not a number." 错误

