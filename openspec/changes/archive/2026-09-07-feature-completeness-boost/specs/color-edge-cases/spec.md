## ADDED Requirements

### Requirement: HSL 序列化精度
系统 SHALL 对 HSL 颜色序列化遵循 CSS Color Level 4 规范，hue 和 saturation/lightness 保留合理精度。

#### Scenario: HSL 整数 hue
- **WHEN** 序列化 `hsl(120, 50%, 50%)`
- **THEN** 输出 `hsl(120, 50%, 50%)`

#### Scenario: HSL 浮点 hue
- **WHEN** 序列化 `hsl(120.5, 50%, 50%)`
- **THEN** 输出保留小数精度

#### Scenario: HSL NaN hue
- **WHEN** 序列化饱和度和亮度均为 0% 的 HSL 颜色
- **THEN** hue 应输出为 `none`

### Requirement: HWB 序列化行为
系统 SHALL 对 HWB 颜色遵循 CSS Color 4 规范——当 whiteness + blackness = 100% 时，规范化为 HSL 中性灰。

#### Scenario: HWB 标准输出
- **WHEN** 序列化 `hwb(0 30% 40%)`
- **THEN** 输出 `hwb(0 30% 40%)`

#### Scenario: HWB 全白全黑混合
- **WHEN** whiteness + blackness ≥ 100%
- **THEN** 输出规范化 HSL

### Requirement: Lab/Lch 边界值处理
系统 SHALL 正确处理 Lab/Lch 的 chroma=0 和 hue=NaN 边界。

#### Scenario: Lch chroma 为 0
- **WHEN** 序列化 `lch(50% 0 270deg)`
- **THEN** hue 输出为 `none`（chroma=0 时 hue 无意义）

#### Scenario: Lab NaN 通道
- **WHEN** 序列化含有 NaN 通道的 Lab 颜色
- **THEN** NaN 通道输出为 `none`

### Requirement: Oklab/Oklch 边界值处理
系统 SHALL 正确处理 Oklab/Oklch 的 chroma=0 和 NaN 边界，与 Lab/Lch 行为一致。

#### Scenario: Oklch chroma 为 0
- **WHEN** 序列化 `oklch(70% 0 180deg)`
- **THEN** hue 输出为 `none`

#### Scenario: Oklab L 百分比
- **WHEN** 序列化 `oklab(0.59 0.1 0.1)`
- **THEN** L 输出为 0%~100% 百分比格式

### Requirement: color-mix 算法
系统 SHALL 实现 CSS Color 4 `color-mix()` 函数，支持 in <space> 指定插值空间。

#### Scenario: sRGB 空间混合
- **WHEN** 调用 `color-mix(in srgb, red 50%, blue)`
- **THEN** 返回 sRGB 空间的中混合色（紫色）

#### Scenario: Oklch 空间混合
- **WHEN** 调用 `color-mix(in oklch, red 50%, blue)`
- **THEN** 返回 Oklch 空间插值结果

### Requirement: color() 现代空间解析
系统 SHALL 正确解析 `color(srgb 1 0 0)`、`color(display-p3 1 0 0)`、`color(xyz 0.5 0.5 0.5)` 等格式。

#### Scenario: sRGB 显式
- **WHEN** 解析 `color(srgb 1 0.5 0)`
- **THEN** 创建对应 ColorSpace::Srgb 的颜色值

#### Scenario: Display P3
- **WHEN** 解析 `color(display-p3 1 0 0)`
- **THEN** 创建 ColorSpace::DisplayP3 颜色值

### Requirement: Gamut Mapping
系统 SHALL 实现 gamut mapping，支持 `in-p3`、`in-srgb` 等色域映射策略。

#### Scenario: 色域映射
- **WHEN** 调用 `gamut-map($color, $method: local-minde)`
- **THEN** 将颜色映射到目标色域内

### Requirement: invert 函数精度
系统 SHALL 实现 `invert($color, $weight, $space)` 支持按色彩空间求反色。

#### Scenario: sRGB 反相
- **WHEN** 调用 `invert(white)` (srgb)
- **THEN** 返回 `black`

#### Scenario: oklch 反相
- **WHEN** 调用 `invert(red, $space: "oklch")`
- **THEN** 返回 oklch 空间反相后的颜色
