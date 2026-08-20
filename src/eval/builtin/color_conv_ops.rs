//! 颜色空间转换工具函数。
//!
//! 包含 ColorFormat 空间判断、空间转换、格式→sRGB 转换、
//! HSL/HWB f64 精度转换、make_color 等。

use crate::error::{Result, SassError};
use crate::parse::ast::{Color, ColorFormat, Value};

use super::super::Evaluator;

/// 判断 ColorFormat 是否与目标空间名称匹配。
pub(crate) fn is_same_space(fmt: &ColorFormat, target: &str) -> bool {
    matches!(
        (fmt, target),
        (ColorFormat::Lab(_, _, _), "lab")
            | (ColorFormat::Lch(_, _, _), "lch")
            | (ColorFormat::Oklab(_, _, _), "oklab")
            | (ColorFormat::Oklch(_, _, _), "oklch")
            | (ColorFormat::DisplayP3(_, _, _), "display-p3")
            | (ColorFormat::Srgb(_, _, _), "srgb")
            | (ColorFormat::SrgbLinear(_, _, _), "srgb-linear")
            | (ColorFormat::DisplayP3Linear(_, _, _), "display-p3-linear")
            | (ColorFormat::A98Rgb(_, _, _), "a98-rgb")
            | (ColorFormat::ProphotoRgb(_, _, _), "prophoto-rgb")
            | (ColorFormat::Rec2020(_, _, _), "rec2020")
            | (ColorFormat::XyzD65(_, _, _), "xyz")
            | (ColorFormat::XyzD65(_, _, _), "xyz-d65")
            | (ColorFormat::XyzD50(_, _, _), "xyz-d50")
            | (ColorFormat::Hsl(_, _, _), "hsl")
            | (ColorFormat::Hwb(_, _, _), "hwb")
    )
}

/// 转换颜色到目标空间，用 f64 精度算法。
pub(crate) fn convert_space(c: &Color, target_space: &str) -> Result<Value> {
    use super::color_conv;

    // 同空间转换——直接返回原始值，避免精度损失
    if is_same_space(&c.format, target_space) {
        return Ok(Value::Color(c.clone()));
    }

    // 获取源 sRGB (0-1) 值
    let (r, g, b) = format_to_srgb_f64(&c.format, c.r, c.g, c.b);

    match target_space {
        "rgb" => {
            Ok(Value::Color(Color::rgba_fmt(c.r, c.g, c.b, c.a, ColorFormat::Auto)))
        }
        "srgb" => {
            Ok(make_color(ColorFormat::Srgb(r, g, b), c.a))
        }
        "srgb-linear" => {
            let (rl, gl, bl) = color_conv::srgb_to_linear_srgb(r, g, b);
            Ok(make_color(ColorFormat::SrgbLinear(rl, gl, bl), c.a))
        }
        "hsl" => {
            let (h, s, l) = match c.format {
                ColorFormat::Hsl(h, s, l) => (h, s, l),
                ColorFormat::Hwb(h, w, bk) => hwb_to_hsl_via_color(h, w, bk),
                _ => Evaluator::rgb_to_hsl(c.r, c.g, c.b),
            };
            Ok(Value::Color(Color::rgba_fmt(c.r, c.g, c.b, c.a, ColorFormat::Hsl(h, s, l))))
        }
        "hwb" => {
            let (h, w, bk) = match c.format {
                ColorFormat::Hwb(h, w, bk) => (h, w, bk),
                _ => {
                    let (r_f, g_f, b_f) = format_to_srgb_f64(&c.format, c.r, c.g, c.b);
                    let (h, _s, _l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);
                    let w = r_f.min(g_f).min(b_f);
                    let bk = 1.0 - r_f.max(g_f).max(b_f);
                    (h, w, bk)
                }
            };
            Ok(Value::Color(Color::rgba_fmt(c.r, c.g, c.b, c.a, ColorFormat::Hwb(h, w, bk))))
        }
        "lab" => {
            let (l, a, b_lab) = color_conv::srgb_to_lab(r, g, b);
            Ok(make_color(ColorFormat::Lab(l, a, b_lab), c.a))
        }
        "lch" => {
            let (l, c_lch, h) = color_conv::srgb_to_lch(r, g, b);
            Ok(make_color(ColorFormat::Lch(l, c_lch, h), c.a))
        }
        "oklab" => {
            let (l, a, b_ok) = color_conv::srgb_to_oklab(r, g, b);
            Ok(make_color(ColorFormat::Oklab(l, a, b_ok), c.a))
        }
        "oklch" => {
            let (l, c_ok, h) = color_conv::srgb_to_oklch(r, g, b);
            Ok(make_color(ColorFormat::Oklch(l, c_ok, h), c.a))
        }
        "display-p3" => {
            let (rp, gp, bp) = color_conv::srgb_to_display_p3(r, g, b);
            Ok(make_color(ColorFormat::DisplayP3(rp, gp, bp), c.a))
        }
        "display-p3-linear" => {
            let (rl, gl, bl) = color_conv::srgb_to_linear_srgb(r, g, b);
            Ok(make_color(ColorFormat::DisplayP3Linear(rl, gl, bl), c.a))
        }
        "a98-rgb" => {
            let (rp, gp, bp) = color_conv::srgb_to_a98_rgb(r, g, b);
            Ok(make_color(ColorFormat::A98Rgb(rp, gp, bp), c.a))
        }
        "prophoto-rgb" => {
            let (rp, gp, bp) = color_conv::srgb_to_prophoto(r, g, b);
            Ok(make_color(ColorFormat::ProphotoRgb(rp, gp, bp), c.a))
        }
        "rec2020" => {
            let (rp, gp, bp) = color_conv::srgb_to_rec2020(r, g, b);
            Ok(make_color(ColorFormat::Rec2020(rp, gp, bp), c.a))
        }
        "xyz" | "xyz-d65" => {
            let (x, y, z) = color_conv::srgb_to_xyz_d65(r, g, b);
            Ok(make_color(ColorFormat::XyzD65(x, y, z), c.a))
        }
        "xyz-d50" => {
            let (x_d65, y_d65, z_d65) = color_conv::srgb_to_xyz_d65(r, g, b);
            let (x, y, z) = color_conv::xyz_d65_to_xyz_d50(x_d65, y_d65, z_d65);
            Ok(make_color(ColorFormat::XyzD50(x, y, z), c.a))
        }
        _ => Err(SassError::Eval(format!("Unknown color space: {target_space}"))),
    }
}

/// 从 ColorFormat 获取 sRGB (0-1) f64 值。
pub(crate) fn format_to_srgb_f64(fmt: &ColorFormat, r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    use super::color_conv;
    match fmt {
        ColorFormat::Auto | ColorFormat::Rgb | ColorFormat::RgbPercent(_, _, _) => {
            (r / 255.0, g / 255.0, b / 255.0)
        }
        ColorFormat::Hsl(h, s, l) => hsl_to_srgb_f64(*h, *s, *l),
        ColorFormat::Hwb(h, w, bk) => hwb_to_srgb_f64(*h, *w, *bk),
        ColorFormat::Lab(l, a, b) => color_conv::lab_to_srgb(*l, *a, *b),
        ColorFormat::Lch(l, c, h) => color_conv::lch_to_srgb(*l, *c, *h),
        ColorFormat::Oklab(l, a, b) => color_conv::oklab_to_srgb(*l, *a, *b),
        ColorFormat::Oklch(l, c, h) => color_conv::oklch_to_srgb(*l, *c, *h),
        ColorFormat::DisplayP3(r, g, b) => color_conv::display_p3_to_srgb(*r, *g, *b),
        ColorFormat::Srgb(r, g, b) => (*r, *g, *b),
        ColorFormat::SrgbLinear(r, g, b) => color_conv::linear_srgb_to_srgb(*r, *g, *b),
        ColorFormat::DisplayP3Linear(r, g, b) => color_conv::linear_srgb_to_srgb(*r, *g, *b),
        ColorFormat::A98Rgb(r, g, b) => color_conv::a98_rgb_to_srgb(*r, *g, *b),
        ColorFormat::ProphotoRgb(r, g, b) => color_conv::prophoto_to_srgb(*r, *g, *b),
        ColorFormat::Rec2020(r, g, b) => color_conv::rec2020_to_srgb(*r, *g, *b),
        ColorFormat::XyzD65(x, y, z) => color_conv::xyz_d65_to_srgb(*x, *y, *z),
        ColorFormat::XyzD50(x, y, z) => {
            let (x_d65, y_d65, z_d65) = color_conv::xyz_d50_to_xyz_d65(*x, *y, *z);
            color_conv::xyz_d65_to_srgb(x_d65, y_d65, z_d65)
        }
    }
}

/// HWB→HSL 转换，直接在 f64 上计算，避免 u8 量化精度损失。
pub(crate) fn hwb_to_hsl_via_color(h: f64, w: f64, b: f64) -> (f64, f64, f64) {
    let h_norm = h.rem_euclid(360.0) / 360.0;
    let (w, b) = if w + b > 1.0 {
        (w / (w + b), b / (w + b))
    } else {
        (w, b)
    };
    let factor = 1.0 - w - b;

    let hue_to_rgb = |hue: f64| -> f64 {
        let mut hue = hue;
        if hue < 0.0 { hue += 1.0; }
        if hue > 1.0 { hue -= 1.0; }
        if hue < 1.0 / 6.0 {
            w + factor * hue * 6.0
        } else if hue < 0.5 {
            w + factor
        } else if hue < 2.0 / 3.0 {
            w + factor * (2.0 / 3.0 - hue) * 6.0
        } else {
            w
        }
    };
    let r = hue_to_rgb(h_norm + 1.0 / 3.0);
    let g = hue_to_rgb(h_norm);
    let bl = hue_to_rgb(h_norm - 1.0 / 3.0);

    let max = r.max(g).max(bl);
    let min = r.min(g).min(bl);
    let l = (max + min) / 2.0;
    let delta = max - min;
    let s = if delta < 1e-12 {
        0.0
    } else {
        delta / (1.0 - (2.0 * l - 1.0).abs())
    };
    (h, s, l)
}

/// HSL → sRGB (0-1) f64 精度转换，不经过 u8 量化。
///
/// 使用 CSS Color 4 规范的 HSL→RGB 算法。
pub(crate) fn hsl_to_srgb_f64(h: f64, s: f64, l: f64) -> (f64, f64, f64) {
    let h = h.rem_euclid(360.0);
    if s == 0.0 {
        return (l, l, l);
    }
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
    (r1 + m, g1 + m, b1 + m)
}

/// HWB → sRGB (0-1) f64 精度转换，不经过 u8 量化。
///
/// 基于 CSS Color 4 规范的 HWB→RGB 算法。
pub(crate) fn hwb_to_srgb_f64(h: f64, w: f64, bk: f64) -> (f64, f64, f64) {
    let (r, g, b) = hsl_to_srgb_f64(h, 1.0, 0.5);
    let factor = 1.0 - w - bk;
    (r * factor + w, g * factor + w, b * factor + w)
}

/// 生成颜色的显示名称（用于错误消息）。
pub(crate) fn color_name(c: &Color) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r.round() as u8, c.g.round() as u8, c.b.round() as u8)
}

/// 从 f64 分量创建 Color（含 sRGB 近似值）。
pub(crate) fn make_color(format: ColorFormat, alpha: f64) -> Value {
    let (r, g, b) = format_to_srgb(&format);
    Value::Color(Color {
        r: (r * 255.0).clamp(0.0, 255.0),
        g: (g * 255.0).clamp(0.0, 255.0),
        b: (b * 255.0).clamp(0.0, 255.0),
        a: alpha,
        format,
    })
}

/// 从 ColorFormat 计算 sRGB 近似值 (0-1)。
fn format_to_srgb(fmt: &ColorFormat) -> (f64, f64, f64) {
    match fmt {
        ColorFormat::Lab(_, _, _) | ColorFormat::Lch(_, _, _)
        | ColorFormat::Oklab(_, _, _) | ColorFormat::Oklch(_, _, _)
        | ColorFormat::DisplayP3(_, _, _) | ColorFormat::Srgb(_, _, _)
        | ColorFormat::SrgbLinear(_, _, _) | ColorFormat::A98Rgb(_, _, _)
        | ColorFormat::ProphotoRgb(_, _, _) | ColorFormat::Rec2020(_, _, _)
        | ColorFormat::XyzD65(_, _, _) | ColorFormat::XyzD50(_, _, _) => {
            format_to_srgb_f64(fmt, 0.0, 0.0, 0.0)
        }
        _ => (0.0, 0.0, 0.0),
    }
}
