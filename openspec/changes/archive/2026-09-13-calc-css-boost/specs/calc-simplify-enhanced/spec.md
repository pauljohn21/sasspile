## ADDED Requirements

### Requirement: calc 乘法常量折叠
系统 SHALL 对 calc 表达式中的 `Number * Number` 进行常量折叠。

#### Scenario: calc 左乘常量
- **WHEN** SCSS 表达式为 `calc(3px * 2 + 1%)`
- **THEN** 输出 `calc(6px + 1%)`

#### Scenario: calc 右乘常量
- **WHEN** SCSS 表达式为 `calc(1% + 3px * 2)`
- **THEN** 输出 `calc(1% + 6px)`

### Requirement: calc 除法常量折叠
系统 SHALL 对 calc 表达式中的 `Number / Number`（除数非零）进行常量折叠。

#### Scenario: calc 左除常量
- **WHEN** SCSS 表达式为 `calc(3px / 2 + 1%)`
- **THEN** 输出 `calc(1.5px + 1%)`

#### Scenario: calc 右除常量
- **WHEN** SCSS 表达式为 `calc(1% + 3px / 2)`
- **THEN** 输出 `calc(1% + 1.5px)`

### Requirement: calc 符号反转
系统 SHALL 对 calc 表达式中的 `a - -b` 和 `a + -b` 进行符号反转简化。

#### Scenario: 减负数转加
- **WHEN** SCSS 表达式为 `calc(1% - -1px)`
- **THEN** 输出 `calc(1% + 1px)`

#### Scenario: 加负数转减
- **WHEN** SCSS 表达式为 `calc(1% + -1px)`
- **THEN** 输出 `calc(1% - 1px)`

### Requirement: calc 特殊常量吸收
系统 SHALL 对calc 表达式中的 `infinity * n`、`-infinity * n`、`NaN` 进行吸收简化。

#### Scenario: infinity 乘法吸收
- **WHEN** SCSS 表达式为 `calc(infinity * 2)`
- **THEN** 输出 `calc(infinity)`

#### Scenario: -infinity 乘法吸收
- **WHEN** SCSS 表达式为 `calc(-infinity * 2)`
- **THEN** 输出 `calc(-infinity)`

#### Scenario: calc 内除零产生 infinity
- **WHEN** SCSS 表达式为 `calc((1/0) * (1% + 1px))`
- **THEN** 输出 `calc(infinity * (1% + 1px))`

### Requirement: calc 数学常量求值
系统 SHALL 识别 calc 表达式中的 `e` 标识符并映射到数学常量 e 进行计算。

#### Scenario: e 乘法求值
- **WHEN** SCSS 表达式为 `calc(e * 2)`
- **THEN** 输出 `5.4365636569`（2 * 2.7182818285，精度 10 位小数）
