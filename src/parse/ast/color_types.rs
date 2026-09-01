//! 颜色空间、输出模式、通道集、颜色结构和格式化辅助函数。
//!
//! 架构：
//! - `ColorSpace` enum：标识色彩空间（不携带数据）
//! - `ColorOutput` enum：控制序列化输出格式
//! - `ChannelSet` enum：按空间分组通道名
//! - `Color` struct：`{ space, channels[3], alpha, output, legacy_rgb[3] }`

use crate::consts::{ALPHA_TOLERANCE, COLOR_MATCH_TOLERANCE};

// ── ColorSpace ────────────────────────────────────────────

/// CSS Color 4 色彩空间标识（不携带通道数据）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    /// Legacy RGB (0-255)。
    Rgb,
    /// sRGB (0-1)。
    Srgb,
    /// 线性 sRGB。
    SrgbLinear,
    /// Display P3。
    DisplayP3,
    /// 线性 Display P3。
    DisplayP3Linear,
    /// A98 RGB。
    A98Rgb,
    /// ProPhoto RGB。
    ProphotoRgb,
    /// Rec2020。
    Rec2020,
    /// XYZ D65。
    XyzD65,
    /// XYZ D50。
    XyzD50,
    /// HSL。
    Hsl,
    /// HWB。
    Hwb,
    /// CIE Lab。
    Lab,
    /// CIE Lch。
    Lch,
    /// OKLab。
    Oklab,
    /// OKLch。
    Oklch,
}

impl ColorSpace {
    /// 从字符串解析色彩空间名。
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "rgb" => Some(Self::Rgb),
            "srgb" => Some(Self::Srgb),
            "srgb-linear" => Some(Self::SrgbLinear),
            "display-p3" => Some(Self::DisplayP3),
            "display-p3-linear" => Some(Self::DisplayP3Linear),
            "a98-rgb" => Some(Self::A98Rgb),
            "prophoto-rgb" => Some(Self::ProphotoRgb),
            "rec2020" => Some(Self::Rec2020),
            "xyz" | "xyz-d65" => Some(Self::XyzD65),
            "xyz-d50" => Some(Self::XyzD50),
            "hsl" => Some(Self::Hsl),
            "hwb" => Some(Self::Hwb),
            "lab" => Some(Self::Lab),
            "lch" => Some(Self::Lch),
            "oklab" => Some(Self::Oklab),
            "oklch" => Some(Self::Oklch),
            _ => None,
        }
    }

    /// 返回空间的规范字符串名。
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rgb => "rgb",
            Self::Srgb => "srgb",
            Self::SrgbLinear => "srgb-linear",
            Self::DisplayP3 => "display-p3",
            Self::DisplayP3Linear => "display-p3-linear",
            Self::A98Rgb => "a98-rgb",
            Self::ProphotoRgb => "prophoto-rgb",
            Self::Rec2020 => "rec2020",
            Self::XyzD65 => "xyz",
            Self::XyzD50 => "xyz-d50",
            Self::Hsl => "hsl",
            Self::Hwb => "hwb",
            Self::Lab => "lab",
            Self::Lch => "lch",
            Self::Oklab => "oklab",
            Self::Oklch => "oklch",
        }
    }

    /// 是否为 legacy 空间（RGB/HSL/HWB）。
    pub fn is_legacy(&self) -> bool {
        matches!(self, Self::Rgb | Self::Hsl | Self::Hwb)
    }

    /// 是否为 RGB 类空间（使用 red/green/blue 通道名）。
    pub fn is_rgb_like(&self) -> bool {
        matches!(
            self,
            Self::Srgb | Self::SrgbLinear | Self::DisplayP3 | Self::DisplayP3Linear
                | Self::A98Rgb | Self::ProphotoRgb | Self::Rec2020
        )
    }
}

// ── ColorOutput ───────────────────────────────────────────

/// 颜色序列化输出模式（独立于空间）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorOutput {
    /// 自动：hex / 命名颜色 / rgba（默认行为）。
    #[default]
    Auto,
    /// 强制 rgb()/rgba() 输出。
    RgbExplicit,
    /// rgb(r%, g%, b%) 百分比输出（HSL 操作结果）。
    RgbPercent,
}

// ── ChannelSet ────────────────────────────────────────────

/// HSL 通道。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HslChannel {
    Hue,
    Saturation,
    Lightness,
}

/// HWB 通道。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HwbChannel {
    Hue,
    Whiteness,
    Blackness,
}

/// RGB 通道。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RgbChannel {
    Red,
    Green,
    Blue,
}

/// Lab 通道。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabChannel {
    Lightness,
    A,
    B,
}

/// Lch 通道。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LchChannel {
    Lightness,
    Chroma,
    Hue,
}

/// Oklab 通道。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OklabChannel {
    Lightness,
    A,
    B,
}

/// Oklch 通道。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OklchChannel {
    Lightness,
    Chroma,
    Hue,
}

/// XYZ 通道。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XyzChannel {
    X,
    Y,
    Z,
}

/// 按空间分组的通道集。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelSet {
    Hsl(HslChannel),
    Hwb(HwbChannel),
    Rgb(RgbChannel),
    Lab(LabChannel),
    Lch(LchChannel),
    Oklab(OklabChannel),
    Oklch(OklchChannel),
    Xyz(XyzChannel),
}

impl ChannelSet {
    /// 从空间和通道名解析。
    pub fn from_str(space: ColorSpace, s: &str) -> Option<Self> {
        match space {
            ColorSpace::Hsl | ColorSpace::Rgb if s == "hue" => Some(Self::Hsl(HslChannel::Hue)),
            ColorSpace::Hsl => match s {
                "hue" => Some(Self::Hsl(HslChannel::Hue)),
                "saturation" => Some(Self::Hsl(HslChannel::Saturation)),
                "lightness" => Some(Self::Hsl(HslChannel::Lightness)),
                _ => None,
            },
            ColorSpace::Hwb => match s {
                "hue" => Some(Self::Hwb(HwbChannel::Hue)),
                "whiteness" => Some(Self::Hwb(HwbChannel::Whiteness)),
                "blackness" => Some(Self::Hwb(HwbChannel::Blackness)),
                _ => None,
            },
            ColorSpace::Rgb => match s {
                "red" => Some(Self::Rgb(RgbChannel::Red)),
                "green" => Some(Self::Rgb(RgbChannel::Green)),
                "blue" => Some(Self::Rgb(RgbChannel::Blue)),
                _ => None,
            },
            ColorSpace::Lab => match s {
                "lightness" => Some(Self::Lab(LabChannel::Lightness)),
                "a" => Some(Self::Lab(LabChannel::A)),
                "b" => Some(Self::Lab(LabChannel::B)),
                _ => None,
            },
            ColorSpace::Lch => match s {
                "lightness" => Some(Self::Lch(LchChannel::Lightness)),
                "chroma" => Some(Self::Lch(LchChannel::Chroma)),
                "hue" => Some(Self::Lch(LchChannel::Hue)),
                _ => None,
            },
            ColorSpace::Oklab => match s {
                "lightness" => Some(Self::Oklab(OklabChannel::Lightness)),
                "a" => Some(Self::Oklab(OklabChannel::A)),
                "b" => Some(Self::Oklab(OklabChannel::B)),
                _ => None,
            },
            ColorSpace::Oklch => match s {
                "lightness" => Some(Self::Oklch(OklchChannel::Lightness)),
                "chroma" => Some(Self::Oklch(OklchChannel::Chroma)),
                "hue" => Some(Self::Oklch(OklchChannel::Hue)),
                _ => None,
            },
            ColorSpace::XyzD65 | ColorSpace::XyzD50 => match s {
                "x" => Some(Self::Xyz(XyzChannel::X)),
                "y" => Some(Self::Xyz(XyzChannel::Y)),
                "z" => Some(Self::Xyz(XyzChannel::Z)),
                _ => None,
            },
            _ => match s {
                "red" => Some(Self::Rgb(RgbChannel::Red)),
                "green" => Some(Self::Rgb(RgbChannel::Green)),
                "blue" => Some(Self::Rgb(RgbChannel::Blue)),
                _ => None,
            },
        }
    }

    /// 返回通道名的字符串形式。
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Hsl(HslChannel::Hue) => "hue",
            Self::Hsl(HslChannel::Saturation) => "saturation",
            Self::Hsl(HslChannel::Lightness) => "lightness",
            Self::Hwb(HwbChannel::Hue) => "hue",
            Self::Hwb(HwbChannel::Whiteness) => "whiteness",
            Self::Hwb(HwbChannel::Blackness) => "blackness",
            Self::Rgb(RgbChannel::Red) => "red",
            Self::Rgb(RgbChannel::Green) => "green",
            Self::Rgb(RgbChannel::Blue) => "blue",
            Self::Lab(LabChannel::Lightness) => "lightness",
            Self::Lab(LabChannel::A) => "a",
            Self::Lab(LabChannel::B) => "b",
            Self::Lch(LchChannel::Lightness) => "lightness",
            Self::Lch(LchChannel::Chroma) => "chroma",
            Self::Lch(LchChannel::Hue) => "hue",
            Self::Oklab(OklabChannel::Lightness) => "lightness",
            Self::Oklab(OklabChannel::A) => "a",
            Self::Oklab(OklabChannel::B) => "b",
            Self::Oklch(OklchChannel::Lightness) => "lightness",
            Self::Oklch(OklchChannel::Chroma) => "chroma",
            Self::Oklch(OklchChannel::Hue) => "hue",
            Self::Xyz(XyzChannel::X) => "x",
            Self::Xyz(XyzChannel::Y) => "y",
            Self::Xyz(XyzChannel::Z) => "z",
        }
    }
}

// ── Color ─────────────────────────────────────────────────

/// 颜色。
///
/// 架构：`{ space, channels[3], alpha, output, legacy_rgb[3] }`
/// - `space`：色彩空间标识
/// - `channels`：通道值（语义随 space 变化）
/// - `output`：输出模式
/// - `legacy_rgb`：sRGB 0-255 缓存（用于 hex/命名色输出）
#[derive(Debug, Clone)]
pub struct Color {
    /// 色彩空间标识。
    pub space: ColorSpace,
    /// 通道值（语义随 space 变化）。
    pub channels: [f64; 3],
    /// Alpha 通道（0.0-1.0）。
    pub a: f64,
    /// 输出模式。
    pub output: ColorOutput,
    /// sRGB 0-255 缓存（用于 hex/命名色输出）。
    pub legacy_rgb: [f64; 3],
}

/// 颜色相等性仅比较 RGBA 值（浮点容差），忽略格式。
impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        (self.legacy_rgb[0] - other.legacy_rgb[0]).abs() < COLOR_MATCH_TOLERANCE
            && (self.legacy_rgb[1] - other.legacy_rgb[1]).abs() < COLOR_MATCH_TOLERANCE
            && (self.legacy_rgb[2] - other.legacy_rgb[2]).abs() < COLOR_MATCH_TOLERANCE
            && (self.a - other.a).abs() < ALPHA_TOLERANCE
    }
}

impl Default for Color {
    fn default() -> Self {
        Self {
            space: ColorSpace::Rgb,
            channels: [0.0, 0.0, 0.0],
            a: 1.0,
            output: ColorOutput::Auto,
            legacy_rgb: [0.0, 0.0, 0.0],
        }
    }
}

impl Color {
    /// 创建 RGB 颜色（Auto 输出）。
    pub fn rgb(r: f64, g: f64, b: f64) -> Self {
        Self {
            space: ColorSpace::Rgb,
            channels: [r, g, b],
            a: 1.0,
            output: ColorOutput::Auto,
            legacy_rgb: [r, g, b],
        }
    }

    /// 创建 RGBA 颜色（Auto 输出）。
    pub fn rgba(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self {
            space: ColorSpace::Rgb,
            channels: [r, g, b],
            a,
            output: ColorOutput::Auto,
            legacy_rgb: [r, g, b],
        }
    }

    /// 创建带空间和输出模式指定的 RGB 颜色。
    pub fn with_rgb(r: f64, g: f64, b: f64, a: f64, space: ColorSpace, output: ColorOutput) -> Self {
        Self {
            space,
            channels: [r, g, b],
            a,
            output,
            legacy_rgb: [r, g, b],
        }
    }

    /// 创建带空间和通道的 HSL 颜色（legacy_rgb 自动计算）。
    pub fn with_hsl(h: f64, s: f64, l: f64, a: f64, output: ColorOutput, legacy_rgb: [f64; 3]) -> Self {
        Self {
            space: ColorSpace::Hsl,
            channels: [h, s, l],
            a,
            output,
            legacy_rgb,
        }
    }

    /// 创建带空间和通道的 HWB 颜色（legacy_rgb 自动计算）。
    pub fn with_hwb(h: f64, w: f64, bk: f64, a: f64, legacy_rgb: [f64; 3]) -> Self {
        Self {
            space: ColorSpace::Hwb,
            channels: [h, w, bk],
            a,
            output: ColorOutput::Auto,
            legacy_rgb,
        }
    }

    /// 创建现代色彩空间颜色（channels + legacy_rgb 分开传入）。
    pub fn with_space(
        space: ColorSpace,
        channels: [f64; 3],
        a: f64,
        output: ColorOutput,
        legacy_rgb: [f64; 3],
    ) -> Self {
        Self {
            space,
            channels,
            a,
            output,
            legacy_rgb,
        }
    }

    /// 用新的 RGB 通道值克隆当前颜色（用于现代 RGB 空间）。
    pub fn clone_with_rgb(&self, r: f64, g: f64, b: f64) -> Self {
        Self {
            space: self.space,
            channels: [r, g, b],
            a: self.a,
            output: self.output,
            legacy_rgb: self.legacy_rgb,
        }
    }

    /// 获取空间标识。
    pub fn space(&self) -> ColorSpace {
        self.space
    }

    /// 获取输出模式。
    pub fn output_mode(&self) -> ColorOutput {
        self.output
    }

    // ── 兼容性 accessor ──

    /// 获取 red 通道（legacy_rgb[0]）。
    pub fn r(&self) -> f64 {
        self.legacy_rgb[0]
    }

    /// 获取 green 通道（legacy_rgb[1]）。
    pub fn g(&self) -> f64 {
        self.legacy_rgb[1]
    }

    /// 获取 blue 通道（legacy_rgb[2]）。
    pub fn b(&self) -> f64 {
        self.legacy_rgb[2]
    }
}
