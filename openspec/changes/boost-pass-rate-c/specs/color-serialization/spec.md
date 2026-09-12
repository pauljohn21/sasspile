## ADDED Requirements

### Requirement: CSS 颜色函数输出必须包含闭合括号
`fmt_color_fn` 函数 SHALL 在序列化 Modern RGB 色彩空间颜色值时输出格式正确的 `color(...)` 表达式，包含闭合括号 `)`。

#### Scenario: 不透明颜色输出
- **WHEN** 序列化一个 alpha=1.0 的 DisplayP3 颜色
- **THEN** 输出格式为 `color(display-p3 R G B)`（包含闭合括号）

#### Scenario: 半透明颜色输出
- **WHEN** 序列化一个 alpha<1.0 的 DisplayP3 颜色
- **THEN** 输出格式为 `color(display-p3 R G B / A)`（包含闭合括号和 /alpha 语法）

#### Scenario: 所有 Modern RGB 空间
- **WHEN** 序列化 Srgb, SrgbLinear, DisplayP3, A98Rgb, ProphotoRgb, Rec2020, XyzD65, XyzD50 空间的颜色
- **THEN** 输出格式均包含闭合括号 `)`
