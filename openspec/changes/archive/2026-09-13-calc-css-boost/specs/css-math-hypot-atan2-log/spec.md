## ADDED Requirements

### Requirement: hypot() 作为 CSS 全局函数可求值
系统 SHALL 将 `hypot(x, y, ...)` 识别为内建数学函数并返回 sqrt(sum(xi^2)) 的数值结果。

#### Scenario: hypot 兼容单位
- **WHEN** SCSS 表达式为 `hypot(3px, 4px)`
- **THEN** 输出 `5px`

#### Scenario: hypot 未知单位报错
- **WHEN** SCSS 表达式为 `hypot(1xx, 2px)`
- **THEN** 编译报错（未知单位 `xx` 不可转换）

#### Scenario: hypot sass_script 参数报错
- **WHEN** SCSS 表达式为 `hypot(7 % 3, 4px)`
- **THEN** 编译报错

### Requirement: atan2() 作为 CSS 全局函数可求值
系统 SHALL 将 `atan2(y, x)` 识别为内建数学函数并返回角度值。

#### Scenario: atan2 兼容单位
- **WHEN** SCSS 表达式为 `atan2(1px, 1px)`
- **THEN** 输出 `45deg`（atan2(1,1) = π/4 = 45°）

#### Scenario: atan2 未知单位报错
- **WHEN** SCSS 表达式为 `atan2(1xx, 2px)`
- **THEN** 编译报错

### Requirement: log() 作为 CSS 全局函数可求值
系统 SHALL 将 `log(x)` 识别为内建数学函数并返回自然对数值（以 e 为底）。

#### Scenario: log 正数
- **WHEN** SCSS 表达式为 `log(100)`
- **THEN** 输出 `4.6051701859`（ln 100，截断到 10 位小数）

#### Scenario: log sass_script 参数报错
- **WHEN** SCSS 表达式为 `log(7 % 3)`
- **THEN** 编译报错
