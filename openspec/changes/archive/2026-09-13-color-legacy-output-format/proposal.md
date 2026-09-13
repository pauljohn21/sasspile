## Why

Sass 旧版 color 函数（`invert`/`complement`/`adjust-color`/`change-color`/`scale-color`/`grayscale`）在计算后强制使用 `ColorOutput::Auto`，导致 HSL 空间颜色丢失原始格式——应输出 `hsl(...)` 却输出 `#hex`，应输出 `rgb(R%, G%, B%)` 却输出 `#hex`。这类 bug 影响约 440~630 个 sass-spec case（color 子域 2717 fails 的主要根源之一），修复后可将 color 从 58% 推到约 75-80%。

## What Changes

- **`invert` legacy 路径**：保留输入色彩空间（HSL→HSL，HWB→HWB，RGB→RGB），不再强制 `ColorSpace::Rgb`
- **`complement`**：保留输入色彩空间，RGB 输入 → `RgbPercent` 输出
- **`adjust-color` HSL 通道**：异构 RGB 输入 + HSL 操作 → 输出 `RgbPercent` 格式
- **`change-color` HSL 路径**：保留输入色彩空间，确保 `rgb_in_range` 时仍输出正确格式
- **`scale-color` HSL 路径**：保留输入色彩空间
- **`grayscale`**：保留输入色彩空间

## Capabilities

### New Capabilities
（无新增能力）

### Modified Capabilities
- `color-invert`: invert 函数现在保留输入色彩空间而非强制转为 RGB
- `color-complement`: complement 保留输入色彩空间
- `color-adjust`: adjust-color 对 HSL 通道操作输出 rgb% 格式
- `color-change`: change-color 保留输入色彩空间
- `color-scale`: scale-color 保留输入色彩空间
- `color-grayscale`: grayscale 保留输入色彩空间

## Impact

- **受影响文件**：
  - `src/eval/builtin/color.rs`（invert + grayscale）
  - `src/eval/builtin/color_hwb_hsl.rs`（complement）
  - `src/eval/builtin/color_adjust.rs`（adjust-color 输出格式）
  - `src/eval/builtin/color_change.rs`（change-color 输出格式）
  - `src/eval/builtin/color_scale.rs`（scale-color 输出格式）
- **API 兼容性**：无破坏性变更——所有修改都是让输出符合 Sass 规范
- **测试影响**：约 440~630 个 sass-spec case 从 FAIL→PASS
