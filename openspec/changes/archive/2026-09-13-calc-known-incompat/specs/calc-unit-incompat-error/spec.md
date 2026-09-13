## ADDED Requirements

### Requirement: 顶层二元不兼容单位 Add/Sub 报错
当 `calc()` 表达式的顶层运算为 `+` 或 `-`，且左右两边都是纯数字+单位字面量，且两个单位属于不同的物理量类别（length、angle、time、frequency、resolution）时，系统 SHALL 产生编译错误。

#### Scenario: 长度 + 角度不兼容
- **WHEN** 输入 `a {b: calc(1px + 1deg);}`
- **THEN** 编译失败，报告单位不兼容错误

#### Scenario: 时间 + 分辨率不兼容
- **WHEN** 输入 `a {b: calc(1s + 1dpi);}`
- **THEN** 编译失败，报告单位不兼容错误

#### Scenario: 频率 + 角度不兼容
- **WHEN** 输入 `a {b: calc(1hz + 1turn);}`
- **THEN** 编译失败，报告单位不兼容错误

#### Scenario: 长度 + 时间（减法）
- **WHEN** 输入 `a {b: calc(1cm - 1ms);}`
- **THEN** 编译失败，报告单位不兼容错误

#### Scenario: 兼容单位不报错
- **WHEN** 输入 `a {b: calc(1px + 1in);}`
- **THEN** 编译成功，结果为 `calc(97px)`（长度单位转换后简化）

#### Scenario: 同一单位不报错
- **WHEN** 输入 `a {b: calc(1px + 2px);}`
- **THEN** 编译成功，结果为 `3px`

#### Scenario: 百分比不被误判
- **WHEN** 输入 `a {b: calc(1px + 1%);}`
- **THEN** 编译成功，保留 `calc(1px + 1%)`（% 有特殊语义，不属于任何物理量类别）

#### Scenario: 无单位不影响
- **WHEN** 输入 `a {b: calc(1px + 2);}`
- **THEN** 编译成功，结果为 `3px`
