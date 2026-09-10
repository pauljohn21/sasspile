## ADDED Requirements

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

### Requirement: atan/asin/acos 遇变量时保留
`atan()`、`asin()`、`acos()` 等反三角函数的参数包含 Sass 变量时，SHALL 保留函数形式不编译时求值。

#### Scenario: atan 含变量保留
- **WHEN** 输入为 `atan($var)`
- **THEN** 输出为 `atan($var)` 保留形式

#### Scenario: atan 含 calc+变量
- **WHEN** 输入为 `atan(3px - 1px + var(--c))`
- **THEN** 输出为 `atan(2px + var(--c))`（简化已知量但保留整体函数）
