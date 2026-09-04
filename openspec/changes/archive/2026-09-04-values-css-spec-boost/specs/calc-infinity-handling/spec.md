## MODIFIED Requirements

### Requirement: infinity/nan 序列化
系统 SHALL 将 `infinity`、`-infinity`、`NaN` 特殊数值序列化为 CSS 兼容格式。当 infinity 携带多个单位时，必须保留所有单位。

#### Scenario: infinity 序列化
- **WHEN** 值为 `infinity`（无单位）
- **THEN** CSS 输出为 `calc(infinity)`

#### Scenario: 负 infinity 序列化
- **WHEN** 值为 `-infinity`
- **THEN** CSS 输出为 `calc(-infinity)`

#### Scenario: NaN 序列化
- **WHEN** 值为 `NaN`
- **THEN** CSS 输出为 `calc(NaN)`

#### Scenario: infinity 带单个单位序列化
- **WHEN** 值为 `infinity` 带单位 `px`
- **THEN** CSS 输出为 `calc(infinity * 1px)`

#### Scenario: infinity 带多个分子单位序列化
- **WHEN** 输入 `math.div(1px * 1em, 0)`
- **THEN** CSS 输出为 `calc(infinity * 1px * 1em)`

#### Scenario: infinity 带分母单位序列化
- **WHEN** 输入 `math.div(1, 0px)`
- **THEN** CSS 输出为 `calc(infinity / 1px)`

#### Scenario: infinity 带分子和分母单位序列化
- **WHEN** 输入 `math.div(1px, 0em)`
- **THEN** CSS 输出为 `calc(infinity * 1px / 1em)`

#### Scenario: 负 infinity 带单位序列化
- **WHEN** 输入 `math.div(-1px * 1em, 0)`
- **THEN** CSS 输出为 `calc(-infinity * 1px * 1em)`
