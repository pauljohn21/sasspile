# color-adjust-change-scale Specification

## ADDED Requirements

### Requirement: color.adjust 实现

`color.adiz` SHALL 接受一个 color 与可选的 `$red, $green, $blue, $hue, $saturation, $lightness, $alpha` keyword args（-255..255 for RGB, -1..1 for alpha），返回调整后颜色。

#### Scenario: adjust alpha
- **WHEN** 输入为 `a {b: color.adjust(red, $alpha: -0.5)}`
- **THEN** 输出 `rgba(255, 0, 0, 0.5)`

### Requirement: color.scale 实现

`color.scale` SHALL 接受 color + % 形式的 `$red, $green, $blue, $saturation, $lightness, $alpha`，按比例缩放。

#### Scenario: scale saturation
- **WHEN** 输入为 `a {b: color.scale(red, $saturation: 50%)}`
- **THEN** 输出具有更高饱和度的颜色

### Requirement: color.opacify / color.fade-in 实现

 SHALL 接受 color + 0..1 的 amount，增加不透明度。

#### Scenario: opacify half-transparent red
- **WHEN** 输入为 `a {b: color.opacify(rgba(255,0,0,0.3), 0.2)}`
- **THEN** 输出 `rgba(255, 0, 0, 0.5)`

### Requirement: color.transparentize / color.fade-out 实现

 SHALL 降低不透明度。

#### Scenario: transparentize
- **WHEN** 输入为 `a {b: color.transparentize(red, 0.3)}`
- **THEN** 输出 `rgba(255, 0, 0, 0.7)`
