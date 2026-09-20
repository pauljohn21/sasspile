# css-round-strategy Specification

## Purpose
定义 CSS `round()` 函数的四种取整策略（nearest/up/down/to-zero）行为，以及可选的 step 参数。Sass 内置 `math.round()` 与此函数行为不同——CSS round 接受策略名作为第一参数。

## ADDED Requirements

### Requirement: round() 策略取整
系统 SHALL 支持 CSS `round(strategy, number, step?)` 函数，其中 strategy 为 `"nearest"`/`"up"`/`"down"`/`"to-zero"`。

#### Scenario: nearest 策略
- **WHEN** 计算 `round("nearest", 5.5)`
- **THEN** 结果为 `6`

#### Scenario: nearest 策略 - 正中间
- **WHEN** 计算 `round("nearest", 0.5)`
- **THEN** 结果为 `1`（round half up）

#### Scenario: up 策略正数
- **WHEN** 计算 `round("up", 5.1)`
- **THEN** 结果为 `6`（向正无穷）

#### Scenario: up 策略负数
- **WHEN** 计算 `round("up", -5.1)`
- **THEN** 结果为 `-5`（向正无穷 = 更大的值）

#### Scenario: down 策略正数
- **WHEN** 计算 `round("down", 5.9)`
- **THEN** 结果为 `5`（向负无穷）

#### Scenario: down 策略负数
- **WHEN** 计算 `round("down", -5.1)`
- **THEN** 结果为 `-6`（向负无穷 = 更小的值）

#### Scenario: to-zero 策略正数
- **WHEN** 计算 `round("to-zero", 5.9)`
- **THEN** 结果为 `5`（向零截断）

#### Scenario: to-zero 策略负数
- **WHEN** 计算 `round("to-zero", -5.9)`
- **THEN** 结果为 `-5`（向零截断）

### Requirement: round() step 参数
CSS `round()` SHALL 支持可选的 step 参数，将值取整到最接近的 step 倍数。

#### Scenario: 2-arg round (number, step)
- **WHEN** 计算 `round(117, 25)`
- **THEN** 结果为 `125`（117/25 = 4.68 → 5 * 25 = 125）

#### Scenario: round down with step
- **WHEN** 计算 `round("down", 13, 10)`
- **THEN** 结果为 `10`（floor(13/10) * 10 = 10）

#### Scenario: round negative step
- **WHEN** 计算 `round(13, -10)`
- **THEN** 结果为 `10`（step 取绝对值）

#### Scenario: round NaN 输入
- **WHEN** 计算 `round(NaN, NaN)`
- **THEN** 输出为 `calc(NaN)`（不传播为数字）

### Requirement: round() 2-arg 简写
Shasspile SHALL 支持 CSS round() 的 2 参数形式 `round(number, step)`（不带策略名，默认 nearest）。

#### Scenario: round(10px, 10px)
- **WHEN** 计算 `round(10px, 10px)`
- **THEN** 结果为 `10px`（10 恰好是 10 的倍数）

#### Scenario: round(13px, 10px)
- **WHEN** 计算 `round(13px, 10px)`
- **THEN** 结果为 `10px`

#### Scenario: round 带单位的负 step
- **WHEN** 计算 `round(-18px, 10px)`
- **THEN** 结果为 `-20px`（-18/10 = -1.8 → nearest = -2 → -20）
