//! 颜色格式、颜色结构和格式化辅助函数。
//!
//! 从 `ast/mod.rs` 拆分出来，保持单文件 ≤ 500 行。

/// 颜色格式——追踪颜色创建方式，影响序列化输出。
#[derive(Debug, Clone, Default)]
pub enum ColorFormat {
    /// 自动：hex / 命名颜色 / rgba（默认行为）。
    #[default]
    Auto,
    /// rgb(r, g, b) / rgba(r, g, b, a)——不转 hex 或命名。
    Rgb,
    /// rgb(r%, g%, b%) / rgba(r%, g%, b%, a)——百分比输出。
    /// 存储 HSL 值用于精确百分比计算 (h: 0-360, s/l: 0-1)。
    RgbPercent(f64, f64, f64),
    /// hsl(h, s%, l%) / hsla(h, s%, l%, a)——存储原始 HSL 值 (h: 0-360, s/l: 0-1)。
    Hsl(f64, f64, f64),
    /// hwb(h w% b% / a)——存储原始 HWB 值 (h: 0-360, w/b: 0-1)。
    Hwb(f64, f64, f64),
    /// lab(L% a b)——CSS Color 4 Lab 空间 (L: 0-100, a/b: 任意)。
    Lab(f64, f64, f64),
    /// lch(L% C Hdeg)——CSS Color 4 LCH 空间 (L: 0-100, C: 任意, H: 0-360)。
    Lch(f64, f64, f64),
    /// oklab(L% a b)——CSS Color 4 OKLab 空间 (L: 0-1→0-100%, a/b: 任意)。
    Oklab(f64, f64, f64),
    /// oklch(L% C Hdeg)——CSS Color 4 OKLCH 空间 (L: 0-1→0-100%, C: 任意, H: 0-360)。
    Oklch(f64, f64, f64),
    /// color(display-p3 r g b)——Display P3 空间 (r/g/b: 0-1)。
    DisplayP3(f64, f64, f64),
    /// color(display-p3-linear r g b)——线性 Display P3 空间 (r/g/b: 0-1)。
    DisplayP3Linear(f64, f64, f64),
    /// color(srgb r g b)——sRGB 空间 (r/g/b: 0-1)。
    Srgb(f64, f64, f64),
    /// color(srgb-linear r g b)——线性 sRGB 空间 (r/g/b: 0-1)。
    SrgbLinear(f64, f64, f64),
    /// color(a98-rgb r g b)——A98 RGB 空间 (r/g/b: 0-1)。
    A98Rgb(f64, f64, f64),
    /// color(prophoto-rgb r g b)——ProPhoto RGB 空间 (r/g/b: 0-1)。
    ProphotoRgb(f64, f64, f64),
    /// color(rec2020 r g b)——Rec2020 空间 (r/g/b: 0-1)。
    Rec2020(f64, f64, f64),
    /// color(xyz r g b)——XYZ D65 空间 (r/g/b: 任意)。
    XyzD65(f64, f64, f64),
    /// color(xyz-d50 r g b)——XYZ D50 空间 (r/g/b: 任意)。
    XyzD50(f64, f64, f64),
}

impl ColorFormat {
    /// 用新的 RGB 通道值克隆当前格式（用于现代 RGB 空间）。
    pub fn clone_with(&self, r: f64, g: f64, b: f64) -> Self {
        match self {
            ColorFormat::DisplayP3(_, _, _) => ColorFormat::DisplayP3(r, g, b),
            ColorFormat::Srgb(_, _, _) => ColorFormat::Srgb(r, g, b),
            ColorFormat::SrgbLinear(_, _, _) => ColorFormat::SrgbLinear(r, g, b),
            ColorFormat::DisplayP3Linear(_, _, _) => ColorFormat::DisplayP3Linear(r, g, b),
            ColorFormat::A98Rgb(_, _, _) => ColorFormat::A98Rgb(r, g, b),
            ColorFormat::ProphotoRgb(_, _, _) => ColorFormat::ProphotoRgb(r, g, b),
            ColorFormat::Rec2020(_, _, _) => ColorFormat::Rec2020(r, g, b),
            ColorFormat::XyzD65(_, _, _) => ColorFormat::XyzD65(r, g, b),
            ColorFormat::XyzD50(_, _, _) => ColorFormat::XyzD50(r, g, b),
            _ => self.clone(),
        }
    }
}

/// 颜色。
#[derive(Debug, Clone)]
pub struct Color {
    /// 红色通道（0.0-255.0）。
    pub r: f64,
    /// 绿色通道（0.0-255.0）。
    pub g: f64,
    /// 蓝色通道（0.0-255.0）。
    pub b: f64,
    /// Alpha 通道（0.0-1.0）。
    pub a: f64,
    /// 颜色格式（追踪创建方式）。
    pub format: ColorFormat,
}

/// 颜色相等性仅比较 RGBA 值（浮点容差 0.5），忽略格式。
impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        (self.r - other.r).abs() < 0.5
            && (self.g - other.g).abs() < 0.5
            && (self.b - other.b).abs() < 0.5
            && (self.a - other.a).abs() < 0.0001
    }
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
            format: ColorFormat::Auto,
        }
    }
}

impl Color {
    /// 创建 RGB 颜色。
    pub fn rgb(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b, a: 1.0, format: ColorFormat::Auto }
    }
    /// 创建 RGBA 颜色。
    pub fn rgba(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a, format: ColorFormat::Auto }
    }
    /// 创建带格式的 RGB 颜色。
    pub fn rgb_fmt(r: f64, g: f64, b: f64, format: ColorFormat) -> Self {
        Self { r, g, b, a: 1.0, format }
    }
    /// 创建带格式的 RGBA 颜色。
    pub fn rgba_fmt(r: f64, g: f64, b: f64, a: f64, format: ColorFormat) -> Self {
        Self { r, g, b, a, format }
    }
}

/// 格式化 hue 值——截断到 10 位小数（与 Dart Sass 一致）。
pub(crate) fn format_hue(h: f64) -> String {
    let h = (h * 1e10).round() / 1e10;
    if h.fract() == 0.0 {
        format!("{}", h as i64)
    } else {
        format!("{h}")
    }
}

/// 格式化百分比值（0.0-1.0 → 0%-100%），浮点精度截断到 11 位小数。
pub(crate) fn format_pct(v: f64) -> String {
    let pct = v * 100.0;
    // 修复浮点精度问题（如 60.00000000000001 → 60）
    let pct = (pct * 1e10).round() / 1e10;
    if pct.fract() == 0.0 {
        format!("{}", pct as i64)
    } else {
        format!("{pct}")
    }
}

/// 格式化百分比值（0.0-100.0 → 0%-100%），用于 rgb(%) 输出。
/// Sass spec 保留最多 10 位小数（如 83.3333333333%）。
pub(crate) fn format_pct_val(v: f64) -> String {
    let v = (v * 1e10).round() / 1e10;
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

/// HSL → RGB 百分比转换（用于百分比输出）。
/// 返回 (r%, g%, b%)，范围 0.0-100.0。
/// 与 Evaluator::hsl_to_rgb 相同算法，但返回百分比而非 u8。
pub(crate) fn hsl_to_rgb_percent(h: f64, s: f64, l: f64) -> (f64, f64, f64) {
    let h = h.rem_euclid(360.0);
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r1, g1, b1) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    ((r1 + m) * 100.0, (g1 + m) * 100.0, (b1 + m) * 100.0)
}

/// 格式化 alpha 值。
pub(crate) fn format_alpha(a: f64) -> String {
    if a.fract() == 0.0 {
        format!("{}", a as i64)
    } else {
        let s = format!("{a}");
        s
    }
}
