## Context

Sass 旧版 color 函数（invert/complement/adjust/change/scale/grayscale）在计算后丢失了输入颜色的色彩空间信息，强制使用 `ColorOutput::Auto` + `ColorSpace::Rgb` 输出为 hex。

根据 sass-spec 规范，这些函数应当保留输入颜色的格式：
- HSL 输入 → 输出 `hsl(...)`
- HSL 操作 + RGB 输入 → 输出 `rgb(R%, G%, B%)`
- HWB 输入 → 输出 `hwb(...)` 或规范化 HSL
- RGB 输入 → 输出 hex / rgb（保持现状）

当前问题：`Color::with_rgb(r, g, b, a, ColorSpace::Rgb, ColorOutput::Auto)` 强制所有 legacy 操作结果为 hex。

## Goals / Non-Goals

**Goals:**
- 所有 legacy color 函数保持输入色彩空间（space + output）
- HSL 输入经 invert/complement/grayscale 后仍输出 HSL
- HSL 通道操作（adjust/change/scale）对 RGB 输入输出 `rgb(R%, G%, B%)`
- 加权 invert（`invert(color, weight%)`）输出 `rgb(R%, G%, B%)`

**Non-Goals:**
- 不改现代色彩空间（Oklab/Oklch/Lab/Lch/DisplayP3 等）的 to-space 转换逻辑
- 不改颜色空间转换矩阵或通道计算精度
- 不改 `change-color`/`adjust-color`/`scale-color` 的数值计算逻辑（只改输出格式）

## Decisions

### Decision 1: invert legacy 路径——保留输入 space

**当前代码**（color.rs:28-37）：
```rust
Color::with_rgb(r, g, b, c.a, ColorSpace::Rgb, ColorOutput::Auto)
```

**改为**：根据 `c.space` 选择输出方式：
- `ColorSpace::Hsl` → `Color::with_hsl(h, s, l, a, ColorOutput::Auto, rgb)`
- `ColorSpace::Hwb` → `Color::with_space(Hwb, [h, w, bk], a, Auto, rgb)`
- `ColorSpace::Rgb` → 保持现有 `with_rgb(..., Auto)`

**Rationale**：`ColorOutput::Auto` + HSL space 的序列化已经正确输出 `hsl(...)`（见 `display_color_spaces.rs:103-147`），只需确保 space 正确。

### Decision 2: complement 路径——使用 `RgbPercent`

**当前代码**（color_hwb_hsl.rs:132-139）：
```rust
Color::with_rgb(r, g, b, a, ColorSpace::Rgb, ColorOutput::Auto)
```

**改为**：
```rust
Color::with_space(c.space, [h, s, l], a, ColorOutput::RgbPercent, rgb)
```

**Rationale**：complement 测试期望 `rgb(87.84%, 25.10%, 31.37%, 0.7)` 格式。使用 RgbPercent 而非 Auto 以输出百分比。

### Decision 3: adjust-color HSL 通道——异构输入使用 RgbPercent

**当前代码**（color_adjust.rs:319）通过 `build_legacy_color`：
- 异构 RGB 输入 + HSL 修改 → 经 `hsl_to_rgb_channels` → `build_legacy_color(Rgb, ...)`
- `build_legacy_color` 的 `Rgb` 分支 → 始终 `ColorOutput::Auto` → hex

**改为**：在 `adjust_legacy` 的 HSL 分支中，异构输入路径使用 `build_channel_modified_color` 或直接构造 `RgbPercent` 输出。

**关键逻辑**：当 `modified == Hsl && c.space != Hsl` 时，如果 `hsl_to_rgb_channels` 结果为分数（非整数），应使用 `RgbPercent`。已有逻辑（color_adjust.rs:251）处理此情况，但 `build_legacy_color` 的 `Rgb` 分支绕过了它。

### Decision 4: change-color / scale-color HSL 路径——保持已有逻辑

`change_color.rs:164` 已有 `RgbPercent` 分支。问题可能在 `rgb_in_range` 判断过于宽松——HSL 操作的 rgb 结果为整数时走 Auto 分支。

**不改数值计算**，只确保 HSL-space 输入的 change/scale 总是走 HSL 格式输出。

### Decision 5: grayscale——保留输入 space

**当前代码**（color.rs:87-94）强制 `ColorSpace::Rgb` + `Auto`。

**改为**：与 invert 类似，保留 `c.space`。

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| 某些 RGB 颜色的 HSL 操作结果本来正确输出 hex，改后变成 rgb% | 严格按 sass-spec 验证——只有 HSL 操作且有分数值时用 RgbPercent |
| HWB 序列化在 Auto 模式下规范化为 HSL | 这是正确行为（display_color_spaces.rs:175-201），不需改动 |
| `build_legacy_color` 的修改影响 change-color RGB 路径 | 只改 HSL 分支，RGB 分支保持不变 |
