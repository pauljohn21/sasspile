## Why

颜色系统目前 sass-spec 通过率仅 2403/6027 = 39%。颜色测试此前被整体跳过以防止无限修复循环，现已取消跳过纳入全量统计。最大的失败集中在 `to_space`（1763 fail）、`change`（239 fail）、`scale`（252 fail）、`mix`（86 fail）、`invert`（56 fail）、`hwb`（131 fail）、`lab/lch/oklab/oklch`（均 ~60 fail）和 `color()`（182 fail）目录。根因是：legacy 颜色同空间转换未规范化、`color()` 单参数透传缺失、`none`/missing 通道处理缺失、序列化精度对齐偏差、以及 `color.mix`/`invert`/`grayscale` 缺乏现代颜色空间支持。

## What Changes

- 取消 `SKIP_DIRS` 中 `core_functions/color` 和 `values/colors` 的跳过，颜色测试纳入全量统计
- 修复 `color()` 函数单参数透传：`color(color(srgb ...))` 应直接返回内部颜色
- 修复 legacy 颜色同空间转换规范化：HWB→HWB 应输出为 HSL 格式（SCSS 规范行为）
- 实现 `none`/missing 通道值处理：`hwb(none 20% 30%)` 等 CSS Color 4 语法
- 修复 `color.to-space()` 精度问题：f64 转换链路精度对齐 spec 期望输出
- 修复 `color.mix()` 支持现代颜色空间（lab/oklab/oklch 等）及 `weight` 参数
- 修复 `color.invert()` 支持现代颜色空间
- 修复序列化精度：小数位数、百分比格式对齐 sass-spec 期望输出
- 修复 `color.adjust/change/scale` 对现代颜色空间的通道处理

## Capabilities

### New Capabilities
- `color-none-channel`: CSS Color 4 `none` 关键字通道处理——解析、序列化、转换中的 missing 通道语义
- `color-mix-modern`: `color.mix()` 在现代颜色空间（lab/oklab/oklch/display-p3 等）中的混合算法

### Modified Capabilities
- `color-serialization`: 颜色序列化精度对齐 spec——小数位数、百分比格式、hue 格式
- `color-space-conversion`: legacy 同空间转换规范化 + `to-space()` 精度修复
- `color-functions`: `color()` 单参数透传 + `adjust/change/scale/invert/grayscale` 现代空间支持

## Impact

- `src/parse/ast/display.rs` — 颜色序列化格式修正
- `src/eval/builtin/color_parse.rs` — `color()` 单参数透传
- `src/eval/builtin/color_conv_ops.rs` — legacy 同空间转换规范化
- `src/eval/builtin/color_adjust.rs` — 现代空间通道修正
- `src/eval/builtin/color_space.rs` — `to-space` 精度 + `none` 通道
- `src/eval/builtin/color_gamut.rs` — 色域检查修正
- `src/eval/builtin/color.rs` — `invert`/`grayscale`/`mix` 现代空间
- `src/eval/builtin/color_inspect.rs` — `is-powerless`/`is-missing` 修正
- `tests/spec_manifest.rs` — 取消颜色目录跳过
- `tests/sass_spec_full.rs` — 取消 `#[ignore]` 标记
