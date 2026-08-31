## MODIFIED Requirements

### Requirement: abs/ceil/floor/round 参数验证
系统 SHALL 验证单参数 math 函数的参数数量和类型。系统 SHALL 接受 `infinity` 和 `-infinity` 作为合法的数字参数。

#### Scenario: abs 参数过多
- **WHEN** 调用 `abs(1, 2)` 传递 2 个位置参数
- **THEN** 系统 报 "Only 1 argument allowed, but 2 were passed." 错误

#### Scenario: abs 参数不足
- **WHEN** 调用 `abs()` 无参数
- **THEN** 系统 报 "Missing argument $number." 错误

#### Scenario: abs 非数字参数
- **WHEN** 调用 `abs("0")` 参数不是数字
- **THEN** 系统 报 "$number: \"0\" is not a number." 错误

#### Scenario: abs infinity 参数
- **WHEN** 调用 `abs(infinity)` 参数为 infinity
- **THEN** 系统 SHALL 返回 infinity，不报 "$number: infinity is not a number." 错误

#### Scenario: sqrt infinity 参数
- **WHEN** 调用 `sqrt(infinity)` 参数为 infinity
- **THEN** 系统 SHALL 返回 infinity，不报 "$number: infinity is not a number." 错误

#### Scenario: abs -infinity 参数
- **WHEN** 调用 `abs(-infinity)` 参数为 -infinity
- **THEN** 系统 SHALL 返回 infinity，不报错误

### Requirement: is-unitless / is_unitless 名称映射

系统 SHALL 同时接受 `is-unitless`（kebab-case）和 `is_unitless`（snake_case）作为函数名，因为 sass-spec 两种形式都使用。

#### Scenario: is-unitless kebab-case 调用
- **WHEN** 调用 `math.is-unitless(5)` 使用 kebab-case 名称
- **THEN** 系统 SHALL 返回 `true`，不报 "Undefined function" 错误

#### Scenario: is_unitless snake_case 调用
- **WHEN** 调用 `is_unitless(5)` 使用 snake_case 名称
- **THEN** 系统 SHALL 返回 `true`，不报 "Undefined function" 错误

#### Scenario: is-unitless 带单位参数
- **WHEN** 调用 `math.is-unitless(5px)` 参数带单位
- **THEN** 系统 SHALL 返回 `false`
