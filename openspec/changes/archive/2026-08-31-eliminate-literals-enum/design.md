## Context

sasspile 当前使用 `ColorFormat` enum 同时承担两个职责：
1. **空间标识**（`Hsl` vs `Lab` vs `Srgb` ...）— 决定序列化格式和通道语义
2. **数据载体**（`Hsl(h, s, l)` 存储 HSL 值）— 携带实际通道数据

这导致色彩空间名作为 `&str` 在 6 个文件中各重复一遍 match arm，通道名 `"hue"`/`"saturation"` 等散布在 `color_space.rs`、`color_adjust.rs`、`color_gamut.rs` 中。同时 `ColorFormat::Auto`/`Rgb`/`RgbPercent` 三个变体不携带空间信息，仅控制输出模式，混入空间 enum 中造成语义混乱。

**当前数据流**：
```
Lexer → Parser → Value::Color(Color{r,g,b,a,format: ColorFormat}) → Evaluator → Serializer
                                         ↑ 空间标识+数据+输出模式 三合一
```

**约束**：
- 250+ 处 `ColorFormat::` 引用分布在 9 个文件
- `display.rs` 15 个序列化 match arm 依赖 ColorFormat 变体
- 测试基线 202/202 + sass-spec 2828/5362 必须不回归
- 不引入新 crate 依赖

## Goals / Non-Goals

**Goals:**
- 用 `ColorSpace` enum（16 变体）替代所有色彩空间 `&str` 比较，编译器保证穷尽性
- 用 `ChannelSet` enum（按空间分组）替代通道名 `&str` 比较
- 用 `ColorOutput` enum 独立表达 `Auto`/`Hex`/`Rgb`/`RgbPercent` 输出模式
- `Color` 结构体重构为 `{ space, channels[3], alpha, output }`
- 命名颜色表合并为单一数据源
- @规则名、CSS at-rules、内建函数名用 enum/const 替代散布字面量
- 数值常量提取到 `consts.rs`
- 错误消息模板化

**Non-Goals:**
- 不改变编译器的对外行为（输入 SCSS → 输出 CSS 不变）
- 不引入 `phf` crate（用 const 数组 + 线性扫描）
- 不重构 `Env` 或 stage 管线架构
- 不恢复 `builtin-dispatch-macro` proc-macro（保持手工 dispatch）
- 不处理 `tests/` 目录中的字面量（仅清理 `src/`）

## Decisions

### D1: Color 结构体重构

**选择**：`Color { space: ColorSpace, channels: [f64; 3], alpha: f64, output: ColorOutput }`

```rust
struct Color {
    pub space: ColorSpace,      // 空间标识
    pub channels: [f64; 3],    // 统一存储 3 个通道值
    pub alpha: f64,
    pub output: ColorOutput,   // 输出模式
    pub legacy_rgb: [f64; 3],  // sRGB 0-255 缓存（用于 hex/命名色输出）
}
```

**替代方案**：保留 ColorFormat 仅添加 space 字段 → 拒绝，因为 ColorFormat 的变体携带数据，无法与 space 字段保持一致。

**legacy_rgb 字段**：`Auto` 输出模式需要 hex/命名色查找，必须保留 sRGB 0-255 值。现代空间（Lab/Oklch 等）的 `channels` 不是 0-255 RGB，所以需要独立缓存。

### D2: ColorSpace enum 设计

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    Rgb,            // legacy RGB (0-255)
    Srgb,           // sRGB (0-1)
    SrgbLinear,     // linear sRGB
    DisplayP3,
    DisplayP3Linear,
    A98Rgb,
    ProphotoRgb,
    Rec2020,
    XyzD65,
    XyzD50,
    Hsl,
    Hwb,
    Lab,
    Lch,
    Oklab,
    Oklch,
}

impl ColorSpace {
    fn from_str(s: &str) -> Option<Self>;
    fn as_str(&self) -> &'static str;
    fn channels(&self) -> &'static [ChannelKind];
    fn is_legacy(&self) -> bool;
    fn is_rgb_like(&self) -> bool;  // Rgb/Srgb/DisplayP3/A98/Prophoto/Rec2020
}
```

**替代方案**：将 `Auto`/`Rgb`/`RgbPercent` 作为 ColorSpace 变体 → 拒绝，因为它们是输出模式而非空间。用独立的 `ColorOutput` enum 表达。

### D3: ChannelSet 按空间分组

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelSet {
    Hsl(HslChannel),     // Hue, Saturation, Lightness
    Hwb(HwbChannel),     // Hue, Whiteness, Blackness
    Rgb(RgbChannel),     // Red, Green, Blue
    Lab(LabChannel),     // Lightness, A, B
    Lch(LchChannel),     // Lightness, Chroma, Hue
    Oklab(OklabChannel), // Lightness, A, B
    Oklch(OklchChannel), // Lightness, Chroma, Hue
    Xyz(XyzChannel),     // X, Y, Z
}

impl ChannelSet {
    fn from_str(space: ColorSpace, s: &str) -> Option<Self>;
    fn as_str(&self) -> &'static str;
}
```

**替代方案**：扁平枚举 `enum Channel { Hue, Saturation, Red, ... }` → 拒绝，因为 Hue 在 HSL 和 HWB 中语义不同，分组保证编译时空间-通道组合合法。

**入口解析**：用户传入 `color.channel($color, "hue", "hsl")` 时，`ChannelSet::from_str(ColorSpace::Hsl, "hue")` 返回 `ChannelSet::Hsl(HslChannel::Hue)`。

### D4: ColorOutput enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorOutput {
    #[default]
    Auto,           // hex/命名色（默认行为）
    RgbExplicit,    // 强制 rgb() 输出
    RgbPercent,     // rgb(r%, g%, b%) 百分比输出
    // HslPercent 已包含在 ColorSpace::Hsl 中
}
```

`RgbPercent` 变体额外存储 HSL 值用于精确百分比计算 → 改为 `Color` 的 `channels` 字段在 `space == Hsl` 时直接存储 HSL 值。

### D5: 命名颜色表单一数据源

```rust
// named_colors.rs
const NAMED_COLORS: &[(&str, u8, u8, u8)] = &[
    ("aliceblue", 240, 248, 255),
    ("antiquewhite", 250, 235, 215),
    ...
];

pub fn lookup(name: &str) -> Option<(u8, u8, u8)>;
pub fn reverse_lookup(r: f64, g: f64, b: f64) -> Option<&'static str>;
```

单一 `const` 数组，正反向查找共用。`reverse_lookup` 遍历同一数组做容差匹配。

### D6: AtRuleKind / CssAtRule enum

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AtRuleKind {
    If, For, Each, While, Mixin, Include, Content,
    Function, Return, Use, Forward, Import, Extend,
    AtRoot, Warn, Debug, Error,
    Other(String),  // 未知 @规则
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CssAtRule {
    Media, Supports, Container, Import, Charset,
    Page, FontFace, FontFeatureValues, Keyframes,
    Layer, Scope, StartingStyle, PositionTry,
    Property, Namespace, Document,
    Other(String),
}
```

`parse_at_rule()` 入口将 `name: String` 解析为 `AtRuleKind`，后续 match 用 enum。

### D7: consts.rs 数值常量

```rust
pub const RGB_MAX: f64 = 255.0;
pub const ALPHA_TOLERANCE: f64 = 0.0001;
pub const COLOR_MATCH_TOLERANCE: f64 = 0.5;
pub const HUE_MAX: f64 = 360.0;
pub const FLOAT_PRECISION: f64 = 1e-10;
pub const PROPHOTO_ET: f64 = 1.0 / 512.0;
pub const LAB_EPSILON: f64 = 216.0 / 24389.0;
pub const LAB_KAPPA: f64 = 24389.0 / 27.0;
// ... 等
```

### D8: error_msgs.rs 错误模板

```rust
pub fn err_not_a_string(param: &str, val: &Value) -> SassError;
pub fn err_not_a_number(param: &str, val: &Value) -> SassError;
pub fn err_not_an_int(param: &str, val: &Value) -> SassError;
pub fn err_not_a_color(param: &str, val: &Value) -> SassError;
pub fn err_missing_arg(param: &str) -> SassError;
pub fn err_wrong_arg_count(expected: usize, actual: usize, singular: bool) -> SassError;
pub fn err_expected_no_units(param: &str, val: &Value) -> SassError;
pub fn err_no_channel(space: &str, channel: &str, color: &Color) -> SassError;
```

## Risks / Trade-offs

- **[Color 结构体重构波及 250+ 引用]** → 一次性全量重构，编译器驱动修复，每步 `cargo check` 验证
- **[display.rs 序列化逻辑全重写]** → 逐空间迁移，每完成一个空间运行对应测试
- **[legacy_rgb 缓存一致性]** → 在 `Color` 构造函数中统一计算 sRGB 缓存，`convert_space` 时更新
- **[ChannelSet 入口解析开销]** → 仅在 `color.channel()` 入口做一次 `from_str`，内部全用 enum 匹配
- **[一次性全量重构无回退点]** → 用 git 分支隔离，每完成一个子系统提交一次，保持可 bisect
- **[ep_full 38 秒回归测试]** → 每个里程碑运行一次，不每次改动都跑
- **[sass-spec 2828/5362 基线]** → 最终验收时运行全量

## Migration Plan

一次性全量重构，但按子系统分批提交：

1. **批次 1 — 基础设施**：新建 `consts.rs`、`error_msgs.rs`、`named_colors.rs`，迁移数值常量和错误消息模板
2. **批次 2 — Color 结构体重构**：重构 `color_types.rs`，定义 `ColorSpace`/`ColorOutput`/`ChannelSet`，迁移 `Color` 和 `ColorFormat`
3. **批次 3 — 序列化重写**：重写 `display.rs` 所有 ColorFormat match arm
4. **批次 4 — 颜色系统迁移**：迁移 `color_space.rs`/`color_gamut.rs`/`color_conv_ops.rs`/`color_adjust.rs`/`color.rs`/`color_conv.rs`/`color_parse.rs`/`builtin/color.rs`
5. **批次 5 — 解析器枚举化**：`AtRuleKind`/`CssAtRule` 替代 `at_rules.rs`/`plain_css.rs` 字面量
6. **批次 6 — 内建函数注册**：`dispatch.rs` 用 const 数组替代内联字面量列表
7. **批次 7 — 回归验证**：202/202 + sass-spec 2828/5362

## Open Questions

- `RgbPercent` 输出模式需要 HSL 值做精确百分比计算。重构后 `space == Hsl` 时 `channels` 直接存 HSL 值，`output == RgbPercent` 时从 HSL channels 计算百分比。需确认这个路径不会丢失精度。
- `ColorFormat::Auto` 在命名色查找时需要 sRGB 0-255 值。`legacy_rgb` 字段在现代空间（如 Oklch）创建时需要从 channels 计算 sRGB 近似值，这个计算是否与当前 `format_to_srgb_f64` 逻辑完全等价。
