## ADDED Requirements

### Requirement: color-adjust 增量精度
系统 SHALL `adjust-color()` 的增量参数基于当前值计算，确保多次调整的累积结果与 sass-spec 期望值一致。

#### Scenario: adjust-color 红色增量
- **WHEN** 调用 `adjust-color(red, $red: 50)`
- **THEN** 红色通道 = min(255, current + 50)

#### Scenario: adjust-color 红色减量
- **WHEN** 调用 `adjust-color(red, $red: -50)`
- **THEN** 红色通道 = max(0, current - 50)

### Requirement: mix 权重边界
系统 SHALL `mix()` 的 weight 参数默认 50%，范围为 0-100%，超出时 clamp 到边界值。

#### Scenario: mix 默认权重
- **WHEN** 调用 `mix(red, blue)`
- **THEN** 返回 50% 红色 + 50% 蓝色的混合结果

#### Scenario: mix 0% 权重
- **WHEN** 调用 `mix(red, blue, 0%)`
- **THEN** 返回纯蓝色

#### Scenario: mix 100% 权重
- **WHEN** 调用 `mix(red, blue, 100%)`
- **THEN** 返回纯红色

### Requirement: HSL 函数输出格式
系统 SHALL `hsl()`/`hsla()` 创建的颜色保留 HSL 序列化格式，不转为 hex。

#### Scenario: hsl 颜色序列化
- **WHEN** 表达式返回 `hsl(120, 50%, 50%)`
- **THEN** 输出保持 `hsl(120, 50%, 50%)` 格式

#### Scenario: hsla 颜色序列化
- **WHEN** 表达式返回 `hsla(120, 50%, 50%, 0.8)`
- **THEN** 输出保持 `hsla(120, 50%, 50%, 0.8)` 格式
