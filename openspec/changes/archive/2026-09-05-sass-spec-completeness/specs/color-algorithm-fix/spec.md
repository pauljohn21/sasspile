## ADDED Requirements

### Requirement: color.scale() 精度修复
MUST 实现符合规范的 `color.scale($color, ...)` 算法——按百分比缩放各通道值。

#### Scenario: 亮度 scale
- **WHEN** 调用 `color.scale(red, $lightness: 20%)`
- **THEN** 返回正确亮度的红色，符合 sass-spec 期望值

#### Scenario: scale 边界值（100%）
- **WHEN** 调用 `color.scale(red, $lightness: 100%)`
- **THEN** 返回最亮值（但不超出有效范围）

#### Scenario: scale 负值
- **WHEN** 调用 `color.scale(red, $lightness: -50%)`
- **THEN** 返回更暗的红色

### Requirement: color.change() 边界修复
MUST 实现 `color.change($color, $prop: value)` 直接替换通道值，处理边界溢出。

#### Scenario: change alpha > 1
- **WHEN** 调用 `color.change(red, $alpha: 1.5)`
- **THEN** alpha 被 clamp 到 1

#### Scenario: change 超出 RGB 范围
- **WHEN** 调用 `color.change(red, $red: 300)`
- **THEN** red 被 clamp 到 255

### Requirement: color.invert() HSL 空间
MUST 修复 `color.invert()` 在 HSL 空间的计算逻辑（hue 旋转 180° 而非简单 255-减）。

#### Scenario: HSL invert
- **WHEN** 调用 `color.invert(hsl(120, 50%, 50%))`（通过 HSL 创建的颜色）
- **THEN** 返回正确反转色相的结果

### Requirement: color.to-space() 转换
MUST 实现 `color.to-space($color, $space)` 色域空间转换。

#### Scenario: sRGB → display-p3
- **WHEN** 调用 `color.to-space(red, display-p3)`
- **THEN** 返回 display-p3 空间表示
