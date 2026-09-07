# Spec: Color 操作链式 API

## ADDED Requirements

### Requirement: Color scale/adjust/change 作为 trait method
Color 操作 SHALL 作为 `Color` 方法使用, 通过 `self -> Self` 链式调用。

#### Scenario: scale 返回新 Color
- **WHEN** 调用 `Color::new(...).scale(&kw)`
- **THEN** 返回新 `Color`, 旧颜色不变

#### Scenario: to_space 接受目标空间
- **WHEN** 调用 `color.to_space(ColorSpace::Lab)`
- **THEN** 返回转换后的颜色, 原空间不变
- **AND** 内部通过 XYZ hub 路由 (任意空间 ↔ XYZ ↔ 任意空间)

### Requirement: to_gamut 链式调用
GAMUT 映射 SHALL 作为 Color 方法, 接受方法参数。

#### Scenario: 链式调用 to_gamut
- **WHEN** 调用 `color.to_space(Lab).to_gamut(GamutMethod::Clamp)`
- **THEN** 返回 gamut-mapped 颜色
- **AND** 中间步骤可通过 OTel span 追踪

### Requirement: 现有 color_xxx builtin 函数保持兼容
REACTOR 重构 SHALL 保持现有 `color.adjust / color.change / color.scale` builtin 函数签名和 sass-spec 通过率。

#### Scenario: 现有测试不回归
- **WHEN** 运行 `cargo test --test compile_test`
- **THEN** 继续保持 57/57 通过

### Requirement: Color 变换追踪
每次 Color 操作 SHALL 创建 OTel span, 记录输入空间、参数、输出空间。

#### Scenario: color.scale 产生可追踪的 span
- **WHEN** RUST_LOG=trace 运行 color.scale 测试
- **THEN** OTel 输出包含 span:
  - `stage = "eval"`
  - `module = "builtin"`
  - `fn = "color.scale"`
  - `space = "rgb"`
  - `channels = [{red: +10%}, {alpha: -5%}]`
  - `result = "#ff3366cc"`
