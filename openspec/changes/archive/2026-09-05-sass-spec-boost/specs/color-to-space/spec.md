## ADDED Requirements

### Requirement: color() 函数解析
系统 SHALL 支持 CSS Color 4 `color()` 函数解析，包括 `color(srgb r g b)`、`color(display-p3 r g b)`、`color(xyz x y z)`、`color(xyz-d65 x y z)`、`color(lab L a b)`、`color(lch L C H)`、`color(oklab L a b)`、`color(oklch L C H)` 等现代色彩空间语法。

#### Scenario: 解析 color(srgb) 函数
- **WHEN** 输入包含 `color(srgb 1 0 0)`
- **THEN** 系统解析为 Srgb 格式的颜色对象

#### Scenario: 解析 color(display-p3) 函数
- **WHEN** 输入包含 `color(display-p3 0.8 0.2 0.1)`
- **THEN** 系统解析为 DisplayP3 格式的颜色对象

#### Scenario: 解析 color(lab) 函数
- **WHEN** 输入包含 `color(lab 50 40 59.5)`
- **THEN** 系统解析为 Lab 格式的颜色对象

### Requirement: to-space 色域转换
系统 SHALL 支持将颜色从源色彩空间转换到目标色彩空间（如 srgb、display-p3、lab、oklab、lch、oklch、xyz），输出对应格式的字符串。

#### Scenario: sRGB 转 display-p3
- **WHEN** 调用 `color.to-space(red, display-p3)`
- **THEN** 系统输出 `color(display-p3 ...)` 格式字符串

#### Scenario: sRGB 转 oklch
- **WHEN** 调用 `color.to-space(red, oklch)`
- **THEN** 系统输出 `color(oklch ...)` 格式字符串

#### Scenario: lab 颜色转 srgb
- **WHEN** 调用 `color.to-space($lab-color, srgb)`
- **THEN** 系统输出 `color(srgb ...)` 或等效 rgb 格式

### Requirement: color() 输出序列化
系统 SHALL 将颜色对象序列化为对应的 `color()` 函数格式字符串，而非降级为 hex 或命名颜色。

#### Scenario: Srgb 颜色序列化
- **WHEN** 颜色格式为 Srgb(1, 0, 0)
- **THEN** 输出 `color(srgb 1 0 0)` 而非 `red` 或 `#ff0000`

#### Scenario: DisplayP3 颜色序列化
- **WHEN** 颜色格式为 DisplayP3(0.8, 0.2, 0.1)
- **THEN** 输出 `color(display-p3 0.8 0.2 0.1)`
