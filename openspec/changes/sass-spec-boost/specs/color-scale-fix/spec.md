## ADDED Requirements

### Requirement: color-scale 通道边界计算
系统 SHALL 基于当前通道值与极值之间的有符号距离计算 scale 结果。正值表示向极值移动，负值表示远离极值移动。公式：`new = current + (max - current) * percent/100`（正向），`new = current - (current - min) * percent/100`（负向）。

#### Scenario: RGB 正向 scale
- **WHEN** 调用 `color.scale(red, $red: 50%)`
- **THEN** 红色通道值增加（向 255 移动一半距离）

#### Scenario: RGB 负向 scale
- **WHEN** 调用 `color.scale(red, $red: -50%)`
- **THEN** 红色通道值减少（向 0 移动一半距离）

#### Scenario: HSL 亮度 scale
- **WHEN** 调用 `color.scale($hsl-color, $lightness: 20%)`
- **THEN** HSL 亮度值按 scale 规则调整

#### Scenario: 现代色彩空间 scale
- **WHEN** 调用 `color.scale($oklch-color, $chroma: 30%)`
- **THEN** Oklch 色相通道按 scale 规则调整

### Requirement: color-scale 零值和满值处理
系统 SHALL 当通道已在极值时，对同方向 scale 操作返回原值不变。

#### Scenario: 已满红色正向 scale
- **WHEN** 调用 `color.scale(red, $red: 100%)`
- **THEN** 返回红色（值不变）

#### Scenario: 已零红色负向 scale
- **WHEN** 调用 `color.scale(green, $red: -100%)`
- **THEN** 红色通道变为 0
