## ADDED Requirements

### Requirement: exp() 作为 CSS 全局函数可求值
系统 SHALL 将 `exp(x)` 识别为内建数学函数并在求值时计算 e^x 的数值结果。

#### Scenario: exp 正数参数
- **WHEN** SCSS 表达式为 `exp(5)`
- **THEN** 输出 `148.4131591026`（e^5 截断到 10 位小数）

#### Scenario: exp 零参数
- **WHEN** SCSS 表达式为 `exp(0)`
- **THEN** 输出 `1`

#### Scenario: exp 负数参数
- **WHEN** SCSS 表达式为 `exp(-10.5)`
- **THEN** 输出 `0.0000275364`

#### Scenario: exp 溢出为 infinity
- **WHEN** SCSS 表达式为 `exp(1000.65)`
- **THEN** 输出 `calc(infinity)`

#### Scenario: exp 大小写不敏感
- **WHEN** SCSS 表达式为 `ExP(5)`
- **THEN** 输出 `148.4131591026`（CSS 函数名大小写不敏感）

#### Scenario: exp sass_script 参数报错
- **WHEN** SCSS 表达式为 `exp(7 % 3)`（SassScript 含运算符）
- **THEN** 编译报错（非纯数值不可直接求值）

### Requirement: sign() 作为 CSS 全局函数可求值
系统 SHALL 将 `sign(x)` 识别为内建数学函数并在求值时返回数值的符号（-1 / 0 / +1 / NaN）。

#### Scenario: sign 正数
- **WHEN** SCSS 表达式为 `sign(3)`
- **THEN** 输出 `1`

#### Scenario: sign 负数
- **WHEN** SCSS 表达式为 `sign(-3)`
- **THEN** 输出 `-1`

#### Scenario: sign 零
- **WHEN** SCSS 表达式为 `sign(0)`
- **THEN** 输出 `0`

#### Scenario: sign NaN
- **WHEN** SCSS 表达式为 `sign(NaN)`
- **THEN** 输出 `calc(NaN)`

#### Scenario: sign 保留单位
- **WHEN** SCSS 表达式为 `sign(3px)`
- **THEN** 输出 `1px`

#### Scenario: sign 大小写不敏感
- **WHEN** SCSS 表达式为 `sIgN(3)`
- **THEN** 输出 `1`
