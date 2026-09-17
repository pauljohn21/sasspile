# math-advanced Specification

## ADDED Requirements

### Requirement: math.pow / math.sqrt / math.log

SHALL 支持指数、对数、平方根。当输入非法返回 NaN 或报错（对齐 dart-sass）。

#### Scenario: sqrt(4)
- **WHEN** 输入为 `a {b: math.sqrt(4)}`
- **THEN** 输出 `2`

#### Scenario: pow(2, 3)
- **WHEN** 输入为 `a {b: math.pow(2, 3)}`
- **THEN** 输出 `8`

### Requirement: math.sin / math.cos / math.tan / math.asin / math.acos / math.atan / math.atan2

SHALL 接受带单位的角度参数（deg/rad/grad/turn），按 dart-sass 规则处理。

#### Scenario: sin(30deg)
- **WHEN** 输入为 `a {b: math.sin(30deg)}`
- **THEN** 输出 `0.5`

### Requirement: math.random

SHALL 接受可选 `$limit`，返回 1..limit（整数）或 0..1（浮点）。

#### Scenario: random range
- **WHEN** 调用 `math.random(10)`
- **THEN** 输出介于 1 与 10 之间（测试用范围断言）

### Requirement: math.hypot

SHALL 计算 `sqrt(a^2 + b^2 + ...)`，接受 2+ 参数。

#### Scenario: hypot(3px, 4px)
- **WHEN** 输入为 `a {b: math.hypot(3px, 4px)}`
- **THEN** 输出 `5px`

### Requirement: math.clamp

SHALL 将值限制在 min..max 之间。

#### Scenario: clamp between 0 and 1
- **WHEN** 输入为 `a {b: math.clamp(0, 0.5, 1)}`
- **THEN** 输出 `0.5`

#### Scenario: clamp above max
- **WHEN** 输入为 `a {b: math.clamp(0, 5, 1)}`
- **THEN** 输出 `1`
