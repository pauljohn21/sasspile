## Why

sasspile 源码中散布约 1400 行硬编码字面量（字符串、数值），分布在 25+ 文件中。核心问题：

1. **ColorFormat→空间名映射重复 6 处**：同一组 13 个色彩空间名 `&str` 在 `color_space.rs`、`color_gamut.rs`、`color_conv_ops.rs` 中各重复一遍，修改一处忘改另五处。
2. **命名颜色表双份重复**：`lookup_named_color`（~150 条）和 `reverse_lookup_named_color`（~150 条）各自维护独立数据源。
3. **内建函数名三重重复**：`dispatch.rs` 中每个模块的 `builtin_name` / `is_known` / `dispatch` 三个函数各列一遍相同函数名。
4. **错误消息模板散布**：`"is not a string"` / `"is not a number"` 等模板在 15+ 文件中重复手写。
5. **数值魔法常量**：`255.0`、`0.0001`、`360.0`、`1e-10` 等散布在计算逻辑中，无语义命名。

这些问题导致维护成本高、拼写错误风险大、编译器无法保证穷尽性。通过全量枚举化重构，用类型系统替代字符串比较，从根源消除字面量重复。

## What Changes

- **BREAKING** 重构 `Color` 结构体为 `{ space: ColorSpace, channels: [f64; 3], alpha: f64, output: ColorOutput }`，移除现有 `ColorFormat` enum
- 新增 `ColorSpace` enum（16 变体）+ `from_str()` / `as_str()` / `channels()` / `is_legacy()` 方法
- 新增 `ColorOutput` enum（`Auto` / `Hex` / `Rgb` / `RgbPercent`）独立表达输出模式
- 新增 `ChannelSet` enum（按空间分组：`Hsl(HslChannel)` / `Rgb(RgbChannel)` / ...）
- 新增 `named_colors.rs` 单一数据源，合并正反向查找
- 新增 `AtRuleKind` enum 替代 `at_rules.rs` 中的 `&str` match arms
- 新增 `CssAtRule` enum 替代 `plain_css.rs` 中的 `const CSS_AT_RULES: &[&str]`
- 新增 `consts.rs` 集中定义数值常量（`RGB_MAX`、`ALPHA_TOLERANCE`、`HUE_MAX` 等）
- 新增 `error_msgs.rs` 错误消息模板函数，消除散布的 `format!("... is not a string.")` 模式
- 重写 `display.rs` 序列化逻辑以适配新 `Color` 结构
- 重写 `color_space.rs` / `color_gamut.rs` / `color_conv_ops.rs` / `color_adjust.rs` 使用 `ColorSpace` enum 替代 `&str` 比较
- `dispatch.rs` 保留手工版本但用 `const` 数组替代内联字面量列表

## Capabilities

### New Capabilities

- `color-space-enum`: ColorSpace / ChannelSet / ColorOutput 枚举体系，替代 ColorFormat 的空间标识和数据存储双职责
- `literal-elimination`: 全项目字面量消除框架——consts.rs / error_msgs.rs / named_colors.rs / AtRuleKind / CssAtRule

### Modified Capabilities

（无现有 spec 需要修改——这是内部重构，不改变对外行为）

## Impact

- **核心结构变更**：`Color` 和 `ColorFormat` 定义在 `parse/ast/color_types.rs`，被 250+ 处引用
- **序列化重写**：`parse/ast/display.rs` 中 15 个 ColorFormat match arm 全部重写
- **颜色系统重写**：`eval/color.rs`、`eval/builtin/color.rs`、`eval/builtin/color_space.rs`、`eval/builtin/color_gamut.rs`、`eval/builtin/color_conv_ops.rs`、`eval/builtin/color_adjust.rs`、`eval/builtin/color_conv.rs`、`eval/builtin/color_parse.rs` 全部受影响
- **解析器修改**：`parse/at_rules.rs` @规则名改为 enum 匹配
- **内建函数注册**：`eval/builtin/dispatch.rs` 三重重复改为 const 数组引用
- **错误处理**：15+ 文件的错误消息改为模板函数调用
- **测试回归**：需通过 compile_test 43 + stage_test 10 + ast_test 8 + common_test 5 + bs_spec 15 + ep_full 121 = 202/202 全通过，sass-spec 基线 2828/5362 不回归
- **新增依赖**：无（不引入 phf 等新 crate）
