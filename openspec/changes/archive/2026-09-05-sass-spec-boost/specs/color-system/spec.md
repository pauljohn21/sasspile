## MODIFIED Requirements

### Requirement: 颜色通道操作支持现代色彩空间
颜色通道操作函数（scale/change/adjust） SHALL 支持 oklab、oklch、lab、lch、xyz 色彩空间的通道调整。

#### Scenario: scale 操作 oklab 通道
- **WHEN** 调用 `color.scale($oklab-color, $a: 50%)`
- **THEN** oklab 的 a 通道按 scale 规则调整

#### Scenario: change 操作 lch 通道
- **WHEN** 调用 `color.change($lch-color, $chroma: 50)`
- **THEN** lch 的色相通道被设置为 50

### Requirement: 颜色输出格式保留
通过 HSL/HWB/Lab/Lch/Oklab/Oklch 等函数创建的颜色 SHALL 保持其原始格式输出，仅在 `to-space` 操作或混合后才可能转变格式。

#### Scenario: Oklch 颜色不转 hex
- **WHEN** 表达式返回 `oklch(70% 0.1 180)`
- **THEN** 输出保持 `oklch(...)` 格式或等效 `color(oklch ...)`

#### Scenario: Lab 颜色不转 hex
- **WHEN** 表达式返回 `lab(50% 40 59.5)`
- **THEN** 输出保持 `lab(...)` 格式或等效 `color(lab ...)`
