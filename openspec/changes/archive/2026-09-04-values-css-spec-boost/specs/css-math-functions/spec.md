## ADDED Requirements

### Requirement: CSS min() 函数简化
系统 SHALL 在 `min()` 函数的所有参数为同单位纯数值时，计算并返回最小值。

#### Scenario: min 同单位
- **WHEN** 输入 `min(1px, 2px)`
- **THEN** 系统输出 `1px`

#### Scenario: min 无单位
- **WHEN** 输入 `min(3, 1, 2)`
- **THEN** 系统输出 `1`

#### Scenario: min 含 var 不简化
- **WHEN** 输入 `min(1px, var(--c))`
- **THEN** 系统保留 `min(1px, var(--c))`

#### Scenario: min 不兼容单位报错
- **WHEN** 输入 `min(1s, 2px)`
- **THEN** 系统报错 "1s and 2px are incompatible."

### Requirement: CSS max() 函数简化
系统 SHALL 在 `max()` 函数的所有参数为同单位纯数值时，计算并返回最大值。

#### Scenario: max 同单位
- **WHEN** 输入 `max(1px, 2px)`
- **THEN** 系统输出 `2px`

#### Scenario: max 含 var 不简化
- **WHEN** 输入 `max(1px, var(--c))`
- **THEN** 系统保留 `max(1px, var(--c))`

### Requirement: CSS clamp() 函数简化
系统 SHALL 在 `clamp()` 的三个参数为同单位纯数值时，计算并返回中间值被 clamp 的结果。

#### Scenario: clamp 全数值
- **WHEN** 输入 `clamp(1px, 5px, 10px)`
- **THEN** 系统输出 `5px`

#### Scenario: clamp 含 var 不简化
- **WHEN** 输入 `clamp(1px, var(--c), 10px)`
- **THEN** 系统保留 `clamp(1px, var(--c), 10px)`

### Requirement: CSS round() 函数简化
系统 SHALL 在 `round()` 的参数为同单位纯数值时计算结果。`round(value, multiple)` 返回最接近 `multiple` 倍数的值。

#### Scenario: round 两参数同单位
- **WHEN** 输入 `round(10px, 3px)`
- **THEN** 系统输出 `9px`

#### Scenario: round 单参数
- **WHEN** 输入 `round(4.5px)`
- **THEN** 系统输出 `5px`

#### Scenario: round 含 var 不简化
- **WHEN** 输入 `round(var(--c), 3px)`
- **THEN** 系统保留 `round(var(--c), 3px)`

### Requirement: CSS mod() 函数简化
系统 SHALL 在 `mod()` 的参数为同单位纯数值时计算模运算结果（floored division）。

#### Scenario: mod 同单位
- **WHEN** 输入 `mod(10px, 3px)`
- **THEN** 系统输出 `1px`

#### Scenario: mod 含 var 不简化
- **WHEN** 输入 `mod(var(--c), 3px)`
- **THEN** 系统保留 `mod(var(--c), 3px)`

### Requirement: CSS rem() 函数简化
系统 SHALL 在 `rem()` 的参数为同单位纯数值时计算余数（truncated division）。

#### Scenario: rem 同单位
- **WHEN** 输入 `rem(10px, 3px)`
- **THEN** 系统输出 `1px`

#### Scenario: rem 含 var 不简化
- **WHEN** 输入 `rem(var(--c), 3px)`
- **THEN** 系统保留 `rem(var(--c), 3px)`
