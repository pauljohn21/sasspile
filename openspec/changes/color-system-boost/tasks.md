## 1. 取消颜色测试跳过

- [x] 1.1 从 `tests/spec_manifest.rs` 的 `SKIP_DIRS` 中移除 `core_functions/color` 和 `values/colors`
- [x] 1.2 取消 `tests/sass_spec_full.rs` 中 `test_core_functions_subdirs` 的 `#[ignore]` 标记
- [x] 1.3 运行颜色子目录测试，记录基线通过率（2403/6027 = 39%）

## 2. color() 单参数透传 + none 通道

- [ ] 2.1 在 `color_parse.rs` 的 `parse_color_space()` 中添加单参数 `Value::Color` 透传
- [ ] 2.2 在 `color_parse.rs` 的 `flatten_space_list` 和参数解析中识别 `none` 关键字 → `f64::NAN`
- [ ] 2.3 在 `color.rs` 的 `hsl()`/`hsla()`/`hwb()` 创建函数中解析 `none` 参数
- [ ] 2.4 在 `display.rs` 序列化中检测 `f64::NAN` → 输出 `none`
- [ ] 2.5 在 `color_conv_ops.rs` 的 `space_to_srgb_f64` 转换中将 `NAN` 替换为 0.0

## 3. Legacy 同空间转换规范化

- [ ] 3.1 在 `color_conv_ops.rs` 的 `convert_space()` 中：legacy 空间同空间转换时不直接返回，改为经过 HSL 规范化计算
- [ ] 3.2 HWB→HWB：经过 `hwb_to_srgb_f64` → `rgb_to_hsl` → `hsl_to_srgb_f64` 规范化链路
- [ ] 3.3 RGB→RGB 和 HSL→HSL 同样规范化（不直接返回原值）
- [ ] 3.4 现代空间同空间转换保持直接返回（Lab→Lab、Oklch→Oklch 等）

## 4. color.to-space 精度修复

- [ ] 4.1 验证 `hwb_to_srgb_f64` 精度：对照 spec `hwb(0deg 20% 30%) → hsl(0, 55.5555555556%, 45%)`
- [ ] 4.2 修复 `convert_space` 中 HWB 输出格式：HWB→其他空间时输出 HSL 而非 HWB
- [ ] 4.3 验证 oklch→lab 转换精度对齐 spec 期望值
- [ ] 4.4 修复 out-of-range 颜色转换（如 `hwb(20deg 200% -125%)` 的超范围值处理）

## 5. color.mix 现代空间支持

- [ ] 5.1 在 `builtin.rs` 的 `builtin_mix` 中添加颜色空间分派：根据两个颜色的空间选择混合算法
- [ ] 5.2 实现独立函数 `mix_legacy(c1, c2, weight)` — legacy RGB 混合
- [ ] 5.3 实现独立函数 `mix_oklch(c1, c2, weight)` — oklch 线性插值
- [ ] 5.4 实现独立函数 `mix_oklab(c1, c2, weight)` — oklab 线性插值
- [ ] 5.5 实现独立函数 `mix_lab(c1, c2, weight)` — lab 线性插值
- [ ] 5.6 实现独立函数 `mix_lch(c1, c2, weight)` — lch 线性插值
- [ ] 5.7 实现独立函数 `mix_display_p3(c1, c2, weight)` — display-p3 线性插值
- [ ] 5.8 混合时将第二个颜色转换到第一个颜色的空间

## 6. color.invert/grayscale 现代空间

- [ ] 6.1 在 `color.rs` 的 `invert` 中添加现代空间分派：oklch → hue+180，oklab → 转 oklch 反转
- [ ] 6.2 实现独立函数 `invert_oklch(c)` — hue + 180
- [ ] 6.3 实现独立函数 `invert_oklab(c)` — 转 oklch 反转 hue
- [ ] 6.4 实现独立函数 `invert_lab(c)` — 转 lch 反转 hue
- [ ] 6.5 在 `color.rs` 的 `grayscale` 中添加现代空间分派
- [ ] 6.6 实现独立函数 `grayscale_oklch(c)` — chroma = 0, hue = none
- [ ] 6.7 实现独立函数 `grayscale_oklab(c)` — a=0, b=0
- [ ] 6.8 实现独立函数 `grayscale_lab(c)` — a=0, b=0

## 7. color.adjust/change/scale 独立函数

- [ ] 7.1 验证 `color_adjust.rs` 中每个空间已有独立函数（adjust_oklch/adjust_oklab/adjust_lab 等）
- [ ] 7.2 修复 adjust/change/scale 中 legacy 颜色 + 现代通道参数的处理（先转换空间再操作）
- [ ] 7.3 验证 scale 函数对现代空间的 chroma/lightness 缩放正确性
- [ ] 7.4 修复 adjust/change 中 `$red/$green/$blue` 参数在 RGB-like 空间（display-p3/srgb/a98-rgb 等）的处理

## 8. 序列化精度对齐

- [ ] 8.1 在 `color_fmt.rs` 的 `format_num` 中确保最多 10 位小数，去除尾随零
- [ ] 8.2 验证 HSL 输出百分比精度：`55.5555555556%` 格式（10 位小数）
- [ ] 8.3 验证 oklch hue 输出带 `deg` 后缀
- [ ] 8.4 验证 lch/oklch chroma=0 时 hue 输出为 `none`
- [ ] 8.5 验证 HWB 颜色 Auto 输出格式（应为 HSL 而非 HWB）

## 9. 测试验证

- [ ] 9.1 运行 `cargo test --test sass_spec_full -- --nocapture test_core_functions_subdirs`，确认通过率提升
- [ ] 9.2 运行核心测试：`cargo test --test compile_test` + `cargo test --test ep_full`
- [ ] 9.3 运行全量 sass-spec 统计：`cargo test --test sass_spec_full -- --nocapture test_sass_spec_full_stats`
- [ ] 9.4 `codegraph sync`
- [ ] 9.5 提交（等用户确认后推送）
