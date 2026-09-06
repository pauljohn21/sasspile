## Why

sasspile 颜色系统当前仅支持 RGB/HSL/HWB 和传统 rgba()/hsl() 构造函数，缺少 CSS Color Level 4 的 lab()/lch()/oklab()/oklch() 等现代色彩空间构造函数。这导致 sass-spec 颜色域 49% 的测试失败，是现代颜色功能的主要障碍。

## What Changes

- **新增 4 个颜色构造函数**: `lab()`, `lch()`, `oklab()`, `oklch()`，支持 CSS Color Level 4 色彩空间
- **扩展序列化逻辑**: 新建 `ColorFormat::Lab/Lch/Oklab/Oklch` 枚举变体，序列化时保留原始色彩空间格式（而非转 hex）
- **color.channel 通道名补全**: 扩展通道名映射表，支持 `lightness`、`a`、`b`、`chroma`、`hue`、`whiteness`、`blackness`、`saturation` 等现代通道名
- **color.to-space 色彩空间转换**: 实现 Lab/Lch/Oklab/Oklch 与其他色彩空间之间的转换
- **精度提升**: 序列化输出保留 ~10 位小数精度，符合 CSS Color 4 规范要求

## Capabilities

### New Capabilities
- `color-lab-constructor`: lab(L a b / alpha) 构造函数 + Lab 序列化
- `color-lch-constructor`: lch(L C H / alpha) 构造函数 + Lch 序列化
- `color-oklab-constructor`: oklab(L a b / alpha) 构造函数 + Oklab 序列化
- `color-oklch-constructor`: oklch(L C H / alpha) 构造函数 + Oklch 序列化
- `color-modern-channels`: color.channel() 现代色彩空间通道名支持
- `color-to-space-modern**: color.to-space() 现代色彩空间转换
- `color-serialization-precision**: 颜色序列化输出高精度小数

### Modified Capabilities
- `color-rgb-serializer`: rgb/rgba 序列化精度提升至 ~10 位
- `color-hsl-serializer`: hsl/hsla 序列化精度配合到 ~10 位

## Impact

- **影响文件**: `src/eval/color.rs`、`src/eval/builtin/color.rs`、`src/eval/builtin/color_conv.rs`、`src/eval/builtin/color_space.rs`、`src/css/serializer.rs`（或等价文件）
- **影响 API**: 公开 builtin 函数注册表扩展
- **sass-spec 预期**: 颜色域通过率 51% → 75%+（新增 ~100+ 测试通过）
