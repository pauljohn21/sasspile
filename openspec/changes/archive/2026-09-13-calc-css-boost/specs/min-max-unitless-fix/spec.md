## ADDED Requirements

### Requirement: min/max 正确处理 unitless 最值
系统 SHALL 在 min/max 返回 unitless 值时不附加任何单位。

#### Scenario: unitless 为最大值
- **WHEN** SCSS 表达式为 `max(1px, 2.5, 0.9px)`
- **THEN** 输出 `2.5`（无 `px` 单位，2.5 是 unitless）

#### Scenario: unitless 为最小值
- **WHEN** SCSS 表达式为 `min(1px, 0.5, 0.9px)`
- **THEN** 输出 `0.5`

### Requirement: min/max 兼容单位选取
系统 SHALL 当参数为兼容长度单位时，以最大数值参数的单位输出结果。

#### Scenario: 最大值单位选取
- **WHEN** SCSS 表达式为 `max(1px, 1in, 1cm)`
- **THEN** 输出 `1in`（1in = 96px，是三者最大值，保持原始单位）

### Requirement: min/max 未知单位编译期求值
系统 SHALL 当参数全为相同 unknown unit 时按数值比较大小并返回最大者。

#### Scenario: 同 unknown unit 比较
- **WHEN** SCSS 表达式为 `max(1d, 2, 3e)`
- **THEN** 输出 `3e`（3e 是最大数值，同单位组返回原始字符串）

**注意**：在此 case 中，`d` 和 `e` 为非法 CSS 单位标识符。根据 Sass 规范，无法在编译期转换，但该组内数值比较有效。
