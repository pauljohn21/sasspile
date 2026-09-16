# color-functions Specification

## Purpose
TBD - created by archiving change boost-spec-pass-rate. Update Purpose after archive.
## Requirements
### Requirement: darken 函数降低颜色亮度
`darken($color, $amount)` MUST 将颜色的 HSL 亮度通道降低 $amount 百分比，返回 hex 或 rgba 格式。

#### Scenario: darken 基本
- **WHEN** SCSS: `a { color: darken(red, 20%); }`
- **THEN** MUST 输出变暗的 hex 颜色（有效颜色）

### Requirement: lighten 函数提升颜色亮度
`lighten($color, $amount)` MUST 将颜色的 HSL 亮度通道提升 $amount 百分比。

#### Scenario: lighten 基本
- **WHEN** SCSS: `a { color: lighten(#333, 50%); }`
- **THEN** MUST 输出亮于输入的 hex 颜色

### Requirement: mix 函数混合两种颜色
`mix($color1, $color2, $weight)` MUST 按权重线性混合两种 RGB 颜色。

#### Scenario: mix 50%
- **WHEN** SCSS: `a { color: mix(#ff0000, #0000ff, 50%); }`
- **THEN** MUST 输出 `#800080`（紫色）或等价 rgba/hsl 值

### Requirement: rgba 4 参数函数
`rgba($red, $green, $blue, $alpha)` MUST 输出 CSS rgba 字符串。

#### Scenario: rgba 4 参数
- **WHEN** SCSS: `a { color: rgba(255, 0, 0, 0.5); }`
- **THEN** MUST 输出 `rgba(255, 0, 0, 0.5)`

### Requirement: grayscale 转换
`grayscale($color)` MUST 将颜色转换为灰度（加权平均）。

#### Scenario: grayscale
- **WHEN** SCSS: `a { color: grayscale(red); }`
- **THEN** MUST 输出灰度 hex 颜色

### Requirement: invert 颜色反转
`invert($color)` MUST 返回 RGB 通道的补色（255 - channel）。

#### Scenario: invert white
- **WHEN** SCSS: `a { color: invert(white); }`
- **THEN** MUST 输出 `#000000`

