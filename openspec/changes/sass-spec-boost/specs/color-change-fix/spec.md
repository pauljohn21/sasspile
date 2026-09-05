## ADDED Requirements

### Requirement: color-change 参数校验
系统 SHALL 校验 `change-color()` 的通道参数为数值类型，否则抛出类型错误：`<channel> requires a number`。

#### Scenario: lab 通道为 none
- **WHEN** 调用 `change-color($color, $lightness: none)`
- **THEN** 系统抛出错误 "lightness requires a number"

#### Scenario: oklch 通道为字符串
- **WHEN** 调用 `change-color($color, $hue: "red")`
- **THEN** 系统抛出错误 "hue requires a number"

### Requirement: color-change 现代色彩空间
系统 SHALL 支持 `change-color()` 在 oklab/oklch/lab/lch 色彩空间上设置通道值。

#### Scenario: 设置 oklch 色相
- **WHEN** 调用 `change-color($color, $hue: 180)`
- **THEN** 创建指定色相的 Oklch 颜色对象

#### Scenario: 设置 lab 亮度
- **WHEN** 调用 `change-color($color, $lightness: 80)`
- **THEN** 创建指定亮度的 Lab 颜色对象

### Requirement: color-change alpha 通道
系统 SHALL alpha 参数范围校验（0-1），超出时 clamp 到边界值。

#### Scenario: alpha 超出 1
- **WHEN** 调用 `change-color($color, $alpha: 1.5)`
- **THEN** alpha 被 clamp 为 1

#### Scenario: alpha 低于 0
- **WHEN** 调用 `change-color($color, $alpha: -0.5)`
- **THEN** alpha 被 clamp 为 0
