## MODIFIED Requirements

### Requirement: math 函数边界值与精度
所有 math 模块内建函数 MUST 符合 CSS Math 规范和 sass-spec 精度要求。**扩展：** 特殊浮点常量 (`infinity`, `-infinity`, `NaN`)  SHALL 被视为合法数值输入。

#### Scenario: percentage 函数的整数输入
- **WHEN** 调用 `math.percentage(0.5)`
- **THEN** 返回 `50%`

#### Scenario: ceil/floor/round 的负数处理
- **WHEN** 调用 `math.ceil(-1.5)`, `math.floor(-2.5)`, `math.round(-1.5)`
- **THEN** 分别返回 `-1`, `-3`, `-2`(符合 Sass Math 规范而非 Rust f64)

#### Scenario: pow 的整数幂
- **WHEN** 调用 `math.pow(2, 3)`
- **THEN** 返回 `8`

#### Scenario: sqrt 的负数处理
- **WHEN** 调用 `math.sqrt(-1)`
- **THEN** 返回 `NaN` 或 NaN 的单位化形式

#### Scenario: 三角函数 deg 单位
- **WHEN** 调用 `math.sin(90deg)`
- **THEN** 返回 `1`

#### Scenario: math.div 的单位列表
- **WHEN** 调用 `math.div(10px, 2px)`
- **THEN** 返回 `5`(无单位)

#### Scenario: clamp 的边界
- **WHEN** 调用 `math.clamp(10px, 20px, 30px)`, `math.clamp(40px, 20px, 30px)`
- **THEN** 分别返回 `20px`, `30px`

#### Scenario: abs 的百分比保持
- **WHEN** 调用 `math.abs(-5%)`
- **THEN** 返回 `5%`

#### Scenario: comparable 的单位兼容性
- **WHEN** 调用 `math.comparable(1px, 1cm)`
- **THEN** 返回 `true`(px 和 cm 均为长度单位,可换算比较)

#### Scenario: comparable 的不兼容单位
- **WHEN** 调用 `math.comparable(1px, 1s)`
- **THEN** 返回 `false`

#### Scenario: pow 接受 infinity 参数
- **WHEN** 调用 `math.pow(2, infinity)` 或全局 `pow(2, infinity)`
- **THEN** 返回 `infinity`

#### Scenario: sqrt 接受 infinity
- **WHEN** 调用 `sqrt(infinity)`
- **THEN** 返回 `infinity`

#### Scenario: asin/acos 超出定义域返回 NaN
- **WHEN** 调用 `asin(infinity)` 或 `asin(2)`
- **THEN** 返回 `NaN`
