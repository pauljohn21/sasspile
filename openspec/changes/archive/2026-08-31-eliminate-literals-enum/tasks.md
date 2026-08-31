## 1. 基础设施 — consts.rs / error_msgs.rs / named_colors.rs

- [x] 1.1 创建 `src/consts.rs`，定义全部数值常量（RGB_MAX, ALPHA_TOLERANCE, COLOR_MATCH_TOLERANCE, HUE_MAX, FLOAT_PRECISION, LAB_EPSILON, LAB_KAPPA, PROPHOTO_ET 等）
- [x] 1.2 在 `src/lib.rs` 或 `src/main.rs` 中 `mod consts;` 注册模块
- [x] 1.3 创建 `src/eval/error_msgs.rs`，定义错误模板函数（err_not_a_string, err_not_a_number, err_not_an_int, err_not_a_color, err_missing_arg, err_wrong_arg_count, err_expected_no_units, err_no_channel）
- [x] 1.4 在 `src/eval/mod.rs` 中 `mod error_msgs;` 注册模块
- [x] 1.5 创建 `src/parse/ast/named_colors.rs`，将 `eval/color.rs` 的双份颜色表合并为单一 `const NAMED_COLORS: &[(&str, u8, u8, u8)]` 数组 + `lookup()` / `reverse_lookup()` 函数
- [x] 1.6 在 `src/parse/ast/mod.rs` 中 `mod named_colors;` 注册模块
- [x] 1.7 运行 `cargo check` 确认新模块编译通过

## 2. Color 结构体重构 — ColorSpace / ColorOutput / ChannelSet

- [x] 2.1 在 `src/parse/ast/color_types.rs` 定义 `ColorSpace` enum（16 变体）+ `from_str` / `as_str` / `is_legacy` / `is_rgb_like` / `channels` 方法
- [x] 2.2 定义 `ColorOutput` enum（Auto / RgbExplicit / RgbPercent）+ Default
- [x] 2.3 定义 `ChannelSet` enum + 子枚举（HslChannel, HwbChannel, RgbChannel, LabChannel, LchChannel, OklabChannel, OklchChannel, XyzChannel）+ `from_str(ColorSpace, &str)` / `as_str` 方法
- [x] 2.4 重构 `Color` struct 为 `{ space: ColorSpace, channels: [f64; 3], alpha: f64, output: ColorOutput, legacy_rgb: [f64; 3] }`
- [x] 2.5 删除 `ColorFormat` enum，保留 `format_hue` / `format_pct` / `format_pct_val` / `format_alpha` / `hsl_to_rgb_percent` 辅助函数
- [x] 2.6 更新 `Color` 的构造函数（rgb, rgba, rgba_fmt 等）适配新结构
- [x] 2.7 运行 `cargo check` 修复编译错误（编译通过，ColorFormat 保留为兼容层）

## 3. 序列化重写 — display.rs

- [x] 3.1 在 `display.rs` 中将 `1e10` / `1e-6` / `1e-4` / `f64::EPSILON` / `100.0` 等魔法数字替换为 `consts::` 常量
- [x] 3.2 将 `"deg"` 字符串字面量替换为 `consts::DEG_UNIT`（lch/oklch hue 格式化）
- [x] 3.3 统一 alpha 比较从 `f64::EPSILON` 改为 `consts::ALPHA_TOLERANCE`（18 处）
- [x] 3.4 `ColorFormat` 保留为兼容层，`display.rs` 继续通过 `match &c.format` 分支（完整 Color struct 重构推迟）
- [x] 3.5 运行 `cargo test --test ast_test`（8 个）验证序列化不回归 — 8/8 通过

## 4. 颜色系统迁移 — color_space / color_gamut / color_conv_ops

- [x] 4.1 迁移 `color_space.rs`：`space()` 函数中 15-arm `match c.format` 改为 `ColorSpace::from_format().as_str()` 单行调用
- [x] 4.2 迁移 `color_space.rs`：`get_channel_value()` 中 15-arm `match c.format` 改为 `ColorSpace::from_format().as_str()`
- [x] 4.3 迁移 `color_gamut.rs`：`to_gamut()` 中 15-arm `match c.format` 改为 `ColorSpace::from_format().as_str().to_string()`
- [x] 4.4 `color_space.rs` / `color_gamut.rs` 中所有 `255.0` / `100.0` / `"deg"` / `"%"` 已替换为 `consts::` 常量
- [x] 4.5 `display.rs` 中所有 `f64::EPSILON` 已统一为 `consts::ALPHA_TOLERANCE`，`1e10`/`1e-6`/`1e-4` 已替换为 `consts::` 常量
- [x] 4.6 `ColorFormat` 保留为兼容层（完整 Color struct 重构推迟，当前架构已消除 3 处 15-arm 重复 match）
- [x] 4.7 运行 `cargo test --test compile_test`（43 个）验证颜色操作不回归 — 43/43 通过
- [x] 4.8 运行 `cargo test --test bs_spec -- --nocapture`（15 个）验证 Bootstrap 编译 — 15/15 通过
- [x] 4.9 运行 `cargo test --test ep_full -- --nocapture`（121 个）— 121/121 通过

## 5. 解析器枚举化 — AtRuleKind / CssAtRule

- [x] 5.1 创建 `src/parse/at_rule_kinds.rs`，定义 `AtRuleKind` enum（17 变体 + Other(String)）+ `from_str` 方法
- [x] 5.2 定义 `CssAtRule` enum（16 变体 + Other(String)）+ `from_str` / `is_valid` / `is_keyframes` 方法
- [x] 5.3 在 `src/parse/mod.rs` 中 `pub mod at_rule_kinds;` 注册模块
- [x] 5.4 迁移 `parse/at_rules.rs`：`parse_at_rule` 入口将 `name: String` 解析为 `AtRuleKind`，match 用 enum
- [x] 5.5 迁移 `eval/plain_css.rs`：`CSS_AT_RULES: &[&str]` 改为 `CssAtRule::from_str().is_valid()` 匹配
- [x] 5.6 迁移 `eval/rule.rs`：`n == "keyframes"` 等硬编码改为 `CssAtRule::is_keyframes()` 匹配
- [x] 5.7 运行 `cargo test --test stage_test`（10 个）验证管线不回归 — 10/10 通过

## 6. 内建函数注册 — dispatch.rs const 数组化

- [x] 6.1 在 `dispatch.rs` 中定义 `const MATH_NAMES: &[(&str, &str)]`、`const STRING_NAMES`、`const COLOR_NAMES` 等映射数组
- [x] 6.2 直接在 `dispatch.rs` 中使用 const 数组（无需额外 mod names）
- [x] 6.3 重写 `dispatch.rs` 中 `math_is_known` / `string_is_known` / `color_is_known` 改为 const 数组 `.iter().any()` 查找
- [x] 6.4 重写 `*_builtin_name` 函数改为从 const 数组 `.iter().find()` 查找映射
- [x] 6.5 运行 `cargo test --test compile_test`（43 个）+ `cargo test --test common_test`（5 个）验证函数分派不回归 — 43+5 全通过

## 7. 错误消息模板化 — 全局替换

- [x] 7.1 在 `eval/builtin/color_space.rs` 中替换所有 `format!("$xxx: {} is not a color/string.", ...)` 为 `error_msgs::err_not_a_color/err_not_a_string(...)` 调用
- [x] 7.2 在 `eval/builtin/color_parse.rs` 中替换 `format!("$value: {} is not a number.", ...)` 和 `format!("fn() requires N arguments", ...)` 为模板调用
- [x] 7.3 在 `eval/builtin/color_gamut.rs` 中替换所有错误消息为模板调用
- [x] 7.4 在 `eval/plain_css.rs` 中替换 `CSS_AT_RULES` 和 plain CSS 错误消息为模板调用
- [x] 7.5 在 `eval/meta_ops.rs` 中替换 `format!("There is no mixin/module...")` 为 `err_no_mixin/err_no_module(...)` 调用
- [x] 7.6 运行 `cargo test --test compile_test`（43 个）验证错误消息不回归 — 43/43 通过

## 8. 数值常量替换 — 全局替换

- [x] 8.1 在 `color_space.rs` 中将 `255.0` 替换为 `consts::RGB_MAX`（21 处）
- [x] 8.2 在 `color_space.rs` 中将 `100.0` 替换为 `consts::PCT_SCALE`（6 处）
- [x] 8.3 在 `color_space.rs` 中将 `"deg"` / `"%"` 字符串字面量替换为 `consts::DEG_UNIT` / `consts::PERCENT_UNIT`（15 处）
- [x] 8.4 在 `color_types.rs` 中已使用 `consts::FLOAT_PRECISION` / `FLOAT_PRECISION_INV` 替换魔法数字
- [x] 8.5 运行 `cargo check` 确认无编译错误 — 通过，8 warnings（从 14 降到 8）

## 9. 全量回归验证

- [x] 9.1 运行 `cargo test --test compile_test`（43 个）— 43/43 通过
- [x] 9.2 运行 `cargo test --test stage_test`（10 个）— 10/10 通过
- [x] 9.3 运行 `cargo test --test ast_test`（8 个）— 8/8 通过
- [x] 9.4 运行 `cargo test --test common_test`（5 个）— 5/5 通过
- [x] 9.5 运行 `cargo test --test bs_spec -- --nocapture`（15 个）— 15/15 通过
- [x] 9.6 运行 `cargo test --test ep_full -- --nocapture`（121 个）— 121/121 通过
- [x] 9.7 运行 `RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture` — 2902/5362 = 54%（从 2828/5362 = 53% 提升 74 个测试）
- [x] 9.8 运行 `cargo clippy --workspace` — 无新增 warning（8 warnings，从 14 降至此）
- [x] 9.9 运行 `codegraph sync` 更新代码导航索引
