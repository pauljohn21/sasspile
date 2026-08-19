//! Sass Level 4 颜色空间函数：channel / to-space / space / same。
//!
//! 使用 `color` crate v0.3 做底层色彩空间转换计算，
//! sasspile 封装自己的序列化格式输出。
//! 支持 CSS Color Level 4 全部色彩空间。

use crate::error::{Result, SassError};
use crate::parse::ast::{Color, ColorFormat, Value};
use im::HashMap;

use super::super::Evaluator;

/// 从 kw_args 中查找参数值，同时支持带 $ 和不带 $ 的 key。
fn kw_get<'a>(kw_args: &'a HashMap<String, Value>, key: &str) -> Option<&'a Value> {
    kw_args.get(key).or_else(|| kw_args.get(&format!("${key}")))
}

/// `color.channel($color, $channel, $space: null)` — 提取颜色通道值。
pub fn channel(args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Option<Value>> {
    let pos_count = args.len();
    let kw_count = kw_args.len();
    if pos_count + kw_count > 3 {
        return Err(SassError::Eval(format!(
            "Only 3 arguments allowed, but {} were passed.", pos_count + kw_count
        )));
    }

    let color_arg = args.first().or_else(|| kw_get(kw_args, "color"));
    let channel_arg = args.get(1).or_else(|| kw_get(kw_args, "channel"));
    let space_arg = args.get(2).or_else(|| kw_get(kw_args, "space"));

    let c = match color_arg {
        Some(Value::Color(c)) => c.clone(),
        Some(v) => return Err(SassError::Eval(format!("$color: {} is not a color.", v))),
        None => return Err(SassError::Eval("Missing argument $color.".into())),
    };

    let ch = match channel_arg {
        Some(Value::String(s, quoted)) => {
            if !quoted {
                return Err(SassError::Eval(format!(
                    "$channel: Expected {} to be a quoted string.", s
                )));
            }
            s.clone()
        }
        Some(v) => return Err(SassError::Eval(format!("$channel: {} is not a string.", v))),
        None => return Err(SassError::Eval("Missing argument $channel.".into())),
    };

    let space = match space_arg {
        Some(Value::String(s, quoted)) => {
            if *quoted {
                return Err(SassError::Eval(format!(
                    "$space: Expected \"{}\" to be an unquoted string.", s
                )));
            }
            Some(s.clone())
        }
        Some(v) => return Err(SassError::Eval(format!("$space: {} is not a string.", v))),
        None => None,
    };

    get_channel_value(&c, &ch, space.as_deref()).map(Some)
}

/// `color.to-space($color, $space)` — 转换颜色到目标空间。
pub fn to_space(args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Option<Value>> {
    let color_arg = args.first().or_else(|| kw_get(kw_args, "color"));
    let space_arg = args.get(1).or_else(|| kw_get(kw_args, "space"));

    match (color_arg, space_arg) {
        (Some(Value::Color(c)), Some(Value::String(space, _))) => {
            convert_space(c, space).map(Some)
        }
        (Some(v), _) => Err(SassError::Eval(format!("$color: {} is not a color.", v))),
        _ => Err(SassError::Eval("Missing argument $space.".into())),
    }
}

/// `color.space($color)` — 返回颜色的空间名称。
pub fn space(args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Option<Value>> {
    let color_arg = args.first().or_else(|| kw_get(kw_args, "color"));
    match color_arg {
        Some(Value::Color(c)) => {
            let space_name = match c.format {
                ColorFormat::Hsl(_, _, _) => "hsl",
                ColorFormat::Hwb(_, _, _) => "hwb",
                ColorFormat::Lab(_, _, _) => "lab",
                ColorFormat::Lch(_, _, _) => "lch",
                ColorFormat::Oklab(_, _, _) => "oklab",
                ColorFormat::Oklch(_, _, _) => "oklch",
                ColorFormat::DisplayP3(_, _, _) => "display-p3",
                ColorFormat::Srgb(_, _, _) => "srgb",
                ColorFormat::SrgbLinear(_, _, _) => "srgb-linear",
                ColorFormat::A98Rgb(_, _, _) => "a98-rgb",
                ColorFormat::ProphotoRgb(_, _, _) => "prophoto-rgb",
                ColorFormat::Rec2020(_, _, _) => "rec2020",
                ColorFormat::XyzD65(_, _, _) => "xyz",
                ColorFormat::XyzD50(_, _, _) => "xyz-d50",
                _ => "rgb",
            };
            Ok(Some(Value::String(space_name.to_string(), false)))
        }
        Some(v) => Err(SassError::Eval(format!("$color: {} is not a color.", v))),
        None => Err(SassError::Eval("Missing argument $color.".into())),
    }
}

/// `color.same($color1, $color2)` — 比较两个颜色是否相同。
pub fn same(args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Option<Value>> {
    let c1 = args.first().or_else(|| kw_get(kw_args, "color1"));
    let c2 = args.get(1).or_else(|| kw_get(kw_args, "color2"));
    match (c1, c2) {
        (Some(Value::Color(a)), Some(Value::Color(b))) => {
            Ok(Some(Value::Bool(a.r == b.r && a.g == b.g && a.b == b.b && (a.a - b.a).abs() < 0.0001)))
        }
        (Some(v), _) => Err(SassError::Eval(format!("$color1: {} is not a color.", v))),
        _ => Err(SassError::Eval("color.same requires 2 color arguments".into())),
    }
}

/// 从颜色中提取通道值，带正确单位。
fn get_channel_value(c: &Color, channel: &str, space: Option<&str>) -> Result<Value> {
    // alpha 通道对所有颜色空间通用
    if channel == "alpha" {
        return Ok(Value::Number(c.a, None));
    }
    let effective_space = space.unwrap_or_else(|| {
        match c.format {
            ColorFormat::Hsl(_, _, _) => "hsl",
            ColorFormat::Hwb(_, _, _) => "hwb",
            ColorFormat::Lab(_, _, _) => "lab",
            ColorFormat::Lch(_, _, _) => "lch",
            ColorFormat::Oklab(_, _, _) => "oklab",
            ColorFormat::Oklch(_, _, _) => "oklch",
            ColorFormat::DisplayP3(_, _, _) => "display-p3",
            ColorFormat::Srgb(_, _, _) => "srgb",
            ColorFormat::SrgbLinear(_, _, _) => "srgb-linear",
            ColorFormat::A98Rgb(_, _, _) => "a98-rgb",
            ColorFormat::ProphotoRgb(_, _, _) => "prophoto-rgb",
            ColorFormat::Rec2020(_, _, _) => "rec2020",
            ColorFormat::XyzD65(_, _, _) => "xyz",
            ColorFormat::XyzD50(_, _, _) => "xyz-d50",
            _ => "rgb",
        }
    });

    match effective_space {
        "rgb" | "srgb" => get_rgb_channel(c, channel),
        "hsl" => {
            let (h, s, l) = match c.format {
                ColorFormat::Hsl(h, s, l) => (h, s, l),
                _ => Evaluator::rgb_to_hsl(c.r, c.g, c.b),
            };
            match channel {
                "hue" => Ok(Value::Number(h, Some("deg".into()))),
                "saturation" => Ok(Value::Number(s * 100.0, Some("%".into()))),
                "lightness" => Ok(Value::Number(l * 100.0, Some("%".into()))),
                _ => Err(SassError::Eval(format!(
                    "$channel: Color {} has no channel named {}.",
                    color_name(c), channel
                ))),
            }
        }
        "hwb" => {
            let (h, w, bk) = match c.format {
                ColorFormat::Hwb(h, w, bk) => (h, w, bk),
                _ => {
                    let (h, _s, _l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);
                    let r = c.r as f64 / 255.0;
                    let g = c.g as f64 / 255.0;
                    let b = c.b as f64 / 255.0;
                    let w = r.min(g).min(b);
                    let bk = 1.0 - r.max(g).max(b);
                    (h, w, bk)
                }
            };
            match channel {
                "hue" => Ok(Value::Number(h, Some("deg".into()))),
                "whiteness" => Ok(Value::Number(w * 100.0, Some("%".into()))),
                "blackness" => Ok(Value::Number(bk * 100.0, Some("%".into()))),
                _ => Err(SassError::Eval(format!(
                    "$channel: Color {} has no channel named {}.",
                    color_name(c), channel
                ))),
            }
        }
        "lab" => {
            let (l, a, b) = match c.format {
                ColorFormat::Lab(l, a, b) => (l, a, b),
                _ => {
                    use super::color_conv;
                    let r = c.r as f64 / 255.0;
                    let g = c.g as f64 / 255.0;
                    let bl = c.b as f64 / 255.0;
                    color_conv::srgb_to_lab(r, g, bl)
                }
            };
            match channel {
                "lightness" => Ok(Value::Number(l, Some("%".into()))),
                "a" => Ok(Value::Number(a, None)),
                "b" => Ok(Value::Number(b, None)),
                _ => Err(SassError::Eval(format!(
                    "$channel: Color {} has no channel named {}.",
                    color_name(c), channel
                ))),
            }
        }
        "lch" => {
            let (l, ch, h) = match c.format {
                ColorFormat::Lch(l, ch, h) => (l, ch, h),
                _ => {
                    use super::color_conv;
                    let r = c.r as f64 / 255.0;
                    let g = c.g as f64 / 255.0;
                    let bl = c.b as f64 / 255.0;
                    color_conv::srgb_to_lch(r, g, bl)
                }
            };
            match channel {
                "lightness" => Ok(Value::Number(l, Some("%".into()))),
                "chroma" => Ok(Value::Number(ch, None)),
                "hue" => Ok(Value::Number(h, Some("deg".into()))),
                _ => Err(SassError::Eval(format!(
                    "$channel: Color {} has no channel named {}.",
                    color_name(c), channel
                ))),
            }
        }
        "oklab" => {
            let (l, a, b) = match c.format {
                ColorFormat::Oklab(l, a, b) => (l, a, b),
                _ => {
                    use super::color_conv;
                    let r = c.r as f64 / 255.0;
                    let g = c.g as f64 / 255.0;
                    let bl = c.b as f64 / 255.0;
                    color_conv::srgb_to_oklab(r, g, bl)
                }
            };
            match channel {
                "lightness" => Ok(Value::Number(l * 100.0, Some("%".into()))),
                "a" => Ok(Value::Number(a, None)),
                "b" => Ok(Value::Number(b, None)),
                _ => Err(SassError::Eval(format!(
                    "$channel: Color {} has no channel named {}.",
                    color_name(c), channel
                ))),
            }
        }
        "oklch" => {
            let (l, ch, h) = match c.format {
                ColorFormat::Oklch(l, ch, h) => (l, ch, h),
                _ => {
                    use super::color_conv;
                    let r = c.r as f64 / 255.0;
                    let g = c.g as f64 / 255.0;
                    let bl = c.b as f64 / 255.0;
                    color_conv::srgb_to_oklch(r, g, bl)
                }
            };
            match channel {
                "lightness" => Ok(Value::Number(l * 100.0, Some("%".into()))),
                "chroma" => Ok(Value::Number(ch, None)),
                "hue" => Ok(Value::Number(h, Some("deg".into()))),
                _ => Err(SassError::Eval(format!(
                    "$channel: Color {} has no channel named {}.",
                    color_name(c), channel
                ))),
            }
        }
        "display-p3" | "a98-rgb" | "prophoto-rgb" | "rec2020" | "srgb-linear" => {
            // 这些空间用 red/green/blue 通道名，值为 0-1
            let (r, g, b) = get_normalized_rgb(c, effective_space);
            match channel {
                "red" => Ok(Value::Number(r, None)),
                "green" => Ok(Value::Number(g, None)),
                "blue" => Ok(Value::Number(b, None)),
                _ => Err(SassError::Eval(format!(
                    "$channel: Color {} has no channel named {}.",
                    color_name(c), channel
                ))),
            }
        }
        "xyz" | "xyz-d65" | "xyz-d50" => {
            let (x, y, z) = get_xyz(c, effective_space);
            match channel {
                "x" => Ok(Value::Number(x, None)),
                "y" => Ok(Value::Number(y, None)),
                "z" => Ok(Value::Number(z, None)),
                _ => Err(SassError::Eval(format!(
                    "$channel: Color {} has no channel named {}.",
                    color_name(c), channel
                ))),
            }
        }
        _ => Err(SassError::Eval(format!(
            "$space: Unknown color space: {effective_space}."
        ))),
    }
}

/// 获取归一化 RGB (0-1) 值。
fn get_normalized_rgb(c: &Color, space: &str) -> (f64, f64, f64) {
    use super::color_conv;
    let r = c.r as f64 / 255.0;
    let g = c.g as f64 / 255.0;
    let b = c.b as f64 / 255.0;
    match space {
        "display-p3" => {
            if let ColorFormat::DisplayP3(r, g, b) = c.format {
                (r, g, b)
            } else {
                color_conv::srgb_to_display_p3(r, g, b)
            }
        }
        "srgb-linear" => {
            if let ColorFormat::SrgbLinear(r, g, b) = c.format {
                (r, g, b)
            } else {
                color_conv::srgb_to_linear_srgb(r, g, b)
            }
        }
        "a98-rgb" => {
            if let ColorFormat::A98Rgb(r, g, b) = c.format {
                (r, g, b)
            } else {
                color_conv::srgb_to_a98_rgb(r, g, b)
            }
        }
        "prophoto-rgb" => {
            if let ColorFormat::ProphotoRgb(r, g, b) = c.format {
                (r, g, b)
            } else {
                color_conv::srgb_to_prophoto(r, g, b)
            }
        }
        "rec2020" => {
            if let ColorFormat::Rec2020(r, g, b) = c.format {
                (r, g, b)
            } else {
                color_conv::srgb_to_rec2020(r, g, b)
            }
        }
        _ => (r, g, b),
    }
}

/// 获取 XYZ 值。
fn get_xyz(c: &Color, space: &str) -> (f64, f64, f64) {
    use super::color_conv;
    let r = c.r as f64 / 255.0;
    let g = c.g as f64 / 255.0;
    let b = c.b as f64 / 255.0;
    match space {
        "xyz" | "xyz-d65" => {
            if let ColorFormat::XyzD65(x, y, z) = c.format {
                (x, y, z)
            } else {
                color_conv::srgb_to_xyz_d65(r, g, b)
            }
        }
        "xyz-d50" => {
            if let ColorFormat::XyzD50(x, y, z) = c.format {
                (x, y, z)
            } else {
                let (x65, y65, z65) = color_conv::srgb_to_xyz_d65(r, g, b);
                color_conv::xyz_d65_to_xyz_d50(x65, y65, z65)
            }
        }
        _ => color_conv::srgb_to_xyz_d65(r, g, b),
    }
}

fn get_rgb_channel(c: &Color, channel: &str) -> Result<Value> {
    match channel {
        "red" => Ok(Value::Number(c.r as f64, None)),
        "green" => Ok(Value::Number(c.g as f64, None)),
        "blue" => Ok(Value::Number(c.b as f64, None)),
        "alpha" => Ok(Value::Number(c.a, None)),
        _ => Err(SassError::Eval(format!(
            "$channel: Color {} has no channel named {}.",
            color_name(c), channel
        ))),
    }
}

/// 判断 ColorFormat 是否与目标空间名称匹配。
fn is_same_space(fmt: &ColorFormat, target: &str) -> bool {
    matches!(
        (fmt, target),
        (ColorFormat::Lab(_, _, _), "lab")
            | (ColorFormat::Lch(_, _, _), "lch")
            | (ColorFormat::Oklab(_, _, _), "oklab")
            | (ColorFormat::Oklch(_, _, _), "oklch")
            | (ColorFormat::DisplayP3(_, _, _), "display-p3")
            | (ColorFormat::Srgb(_, _, _), "srgb")
            | (ColorFormat::SrgbLinear(_, _, _), "srgb-linear")
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
fn convert_space(c: &Color, target_space: &str) -> Result<Value> {
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
fn format_to_srgb_f64(fmt: &ColorFormat, r_u8: u8, g_u8: u8, b_u8: u8) -> (f64, f64, f64) {
    use super::color_conv;
    match fmt {
        ColorFormat::Auto | ColorFormat::Rgb | ColorFormat::RgbPercent(_, _, _) => {
            (r_u8 as f64 / 255.0, g_u8 as f64 / 255.0, b_u8 as f64 / 255.0)
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
fn hwb_to_hsl_via_color(h: f64, w: f64, b: f64) -> (f64, f64, f64) {
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
fn hsl_to_srgb_f64(h: f64, s: f64, l: f64) -> (f64, f64, f64) {
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
fn hwb_to_srgb_f64(h: f64, w: f64, bk: f64) -> (f64, f64, f64) {
    let (r, g, b) = hsl_to_srgb_f64(h, 1.0, 0.5);
    let factor = 1.0 - w - bk;
    (r * factor + w, g * factor + w, b * factor + w)
}

/// 生成颜色的显示名称（用于错误消息）。
fn color_name(c: &Color) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
}

/// 解析 CSS Color 4 颜色函数：lab/lch/oklab/oklch/color()。
/// 返回 Value::Color，sRGB 近似值用 color crate 计算。
pub fn parse_color_fn(name: &str, args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Value> {
    // 展开空格分隔的参数
    let flat = flatten_space_list(args);
    match name {
        "lab" => parse_lab(&flat),
        "lch" => parse_lch(&flat),
        "oklab" => parse_oklab(&flat),
        "oklch" => parse_oklch(&flat),
        "color" => parse_color_space(&flat),
        _ => Err(SassError::UndefinedFunction(name.into())),
    }
}

/// 展开空格分隔的 List 参数。
fn flatten_space_list(args: &[Value]) -> Vec<Value> {
    if args.len() == 1
        && let Value::List(items, crate::parse::ast::Separator::Space, false) = &args[0] {
            return items.clone();
        }
    args.to_vec()
}

/// 从 Value 提取 f64 数值（支持百分比→0-1 转换）。
fn extract_num(v: &Value, scale_pct: bool) -> Result<f64> {
    match v {
        Value::Number(n, Some(u)) if u == "%" && scale_pct => Ok(*n / 100.0),
        Value::Number(n, _) => Ok(*n),
        _ => Err(SassError::Eval(format!("$value: {} is not a number.", v))),
    }
}

/// 从百分比 Value 提取原始值（50% → 50.0，不除以100）。
/// 用于 lab/lch 的 L 分量，spec 中 lab(50% ...) 的 50% 就是 50.0。
fn extract_pct_value(v: &Value) -> Result<f64> {
    match v {
        Value::Number(n, Some(u)) if u == "%" => Ok(*n),
        Value::Number(n, _) => Ok(*n),
        _ => Err(SassError::Eval(format!("$value: {} is not a number.", v))),
    }
}

/// 从 Value 提取 hue 值（支持 deg 单位）。
fn extract_hue(v: &Value) -> Result<f64> {
    match v {
        Value::Number(n, Some(u)) if u == "deg" => Ok(*n),
        Value::Number(n, _) => Ok(*n),
        _ => Err(SassError::Eval(format!("$value: {} is not a number.", v))),
    }
}

/// 从 f64 分量创建 Color（含 sRGB 近似值）。
fn make_color(format: ColorFormat, alpha: f64) -> Value {
    let (r, g, b) = format_to_srgb(&format);
    Value::Color(Color {
        r: (r * 255.0).round().clamp(0.0, 255.0) as u8,
        g: (g * 255.0).round().clamp(0.0, 255.0) as u8,
        b: (b * 255.0).round().clamp(0.0, 255.0) as u8,
        a: alpha,
        format,
    })
}

/// 从 ColorFormat 计算 sRGB 近似值 (0-1)。
fn format_to_srgb(fmt: &ColorFormat) -> (f64, f64, f64) {
    // 对于 Auto/Rgb/RgbPercent/Hsl/Hwb 等 sRGB 格式，r/g/b 是 0（因为这里没有 Color 的 r_u8）。
    // 但这个函数只在 make_color 中使用，此时新格式的 sRGB 值应该从格式本身计算。
    // 对于 sRGB 系列格式（Auto/Rgb 等），在 make_color 调用方已经有 r_u8/g_u8/b_u8。
    // 所以这里只处理非 sRGB 格式。
    match fmt {
        ColorFormat::Lab(_, _, _) | ColorFormat::Lch(_, _, _)
        | ColorFormat::Oklab(_, _, _) | ColorFormat::Oklch(_, _, _)
        | ColorFormat::DisplayP3(_, _, _) | ColorFormat::Srgb(_, _, _)
        | ColorFormat::SrgbLinear(_, _, _) | ColorFormat::A98Rgb(_, _, _)
        | ColorFormat::ProphotoRgb(_, _, _) | ColorFormat::Rec2020(_, _, _)
        | ColorFormat::XyzD65(_, _, _) | ColorFormat::XyzD50(_, _, _) => {
            // 从格式计算 sRGB——使用一个 dummy u8 值，实际计算依赖格式中的分量
            format_to_srgb_f64(fmt, 0, 0, 0)
        }
        // sRGB 系列——返回 0，由 make_color 调用方处理
        _ => (0.0, 0.0, 0.0),
    }
}

/// lab(L% a b [/ alpha])
fn parse_lab(args: &[Value]) -> Result<Value> {
    let (nums, alpha) = split_alpha(args);
    if nums.len() < 3 {
        return Err(SassError::Eval(format!("lab() requires 3 arguments, got {}", nums.len())));
    }
    let l = extract_pct_value(&nums[0])?;  // L% → 0-100
    let a = extract_num(&nums[1], false)?;
    let b = extract_num(&nums[2], false)?;
    Ok(make_color(ColorFormat::Lab(l, a, b), alpha))
}

/// lch(L% C Hdeg [/ alpha])
fn parse_lch(args: &[Value]) -> Result<Value> {
    let (nums, alpha) = split_alpha(args);
    if nums.len() < 3 {
        return Err(SassError::Eval(format!("lch() requires 3 arguments, got {}", nums.len())));
    }
    let l = extract_pct_value(&nums[0])?;
    let c = extract_num(&nums[1], false)?;
    let h = extract_hue(&nums[2])?;
    Ok(make_color(ColorFormat::Lch(l, c, h), alpha))
}

/// oklab(L% a b [/ alpha])
fn parse_oklab(args: &[Value]) -> Result<Value> {
    let (nums, alpha) = split_alpha(args);
    if nums.len() < 3 {
        return Err(SassError::Eval(format!("oklab() requires 3 arguments, got {}", nums.len())));
    }
    let l = extract_num(&nums[0], true)?;  // L% → 0-1
    let a = extract_num(&nums[1], false)?;
    let b = extract_num(&nums[2], false)?;
    Ok(make_color(ColorFormat::Oklab(l, a, b), alpha))
}

/// oklch(L% C Hdeg [/ alpha])
fn parse_oklch(args: &[Value]) -> Result<Value> {
    let (nums, alpha) = split_alpha(args);
    if nums.len() < 3 {
        return Err(SassError::Eval(format!("oklch() requires 3 arguments, got {}", nums.len())));
    }
    let l = extract_num(&nums[0], true)?;
    let c = extract_num(&nums[1], false)?;
    let h = extract_hue(&nums[2])?;
    Ok(make_color(ColorFormat::Oklch(l, c, h), alpha))
}

/// color(space r g b [/ alpha])
fn parse_color_space(args: &[Value]) -> Result<Value> {
    let (nums, alpha) = split_alpha(args);
    if nums.len() < 4 {
        return Err(SassError::Eval(format!("color() requires 4 arguments (space + 3 channels), got {}", nums.len())));
    }
    let space = match &nums[0] {
        Value::String(s, _) => s.clone(),
        _ => return Err(SassError::Eval("color() first argument must be a color space name".into())),
    };
    let r = extract_num(&nums[1], false)?;
    let g = extract_num(&nums[2], false)?;
    let b = extract_num(&nums[3], false)?;
    let fmt = match space.as_str() {
        "display-p3" => ColorFormat::DisplayP3(r, g, b),
        "srgb" => ColorFormat::Srgb(r, g, b),
        "srgb-linear" => ColorFormat::SrgbLinear(r, g, b),
        "a98-rgb" => ColorFormat::A98Rgb(r, g, b),
        "prophoto-rgb" => ColorFormat::ProphotoRgb(r, g, b),
        "rec2020" => ColorFormat::Rec2020(r, g, b),
        "xyz" => ColorFormat::XyzD65(r, g, b),
        "xyz-d50" => ColorFormat::XyzD50(r, g, b),
        _ => return Err(SassError::Eval(format!("Unknown color space: {space}"))),
    };
    Ok(make_color(fmt, alpha))
}

/// 分离 alpha 分量：参数末尾可能有 / alpha。
/// 返回 (颜色分量, alpha值)。
fn split_alpha(args: &[Value]) -> (Vec<Value>, f64) {
    // 只检查 / 分隔符的情况
    if args.len() >= 2 {
        if let Value::List(items, crate::parse::ast::Separator::Slash, false) = &args[args.len() - 1] {
            if items.len() == 2 {
                let mut nums = args[..args.len() - 1].to_vec();
                nums.push(items[0].clone());
                let alpha = match &items[1] {
                    Value::Number(n, Some(u)) if u == "%" => *n / 100.0,
                    Value::Number(n, _) => *n,
                    _ => 1.0,
                };
                return (nums, alpha);
            }
        }
    }
    (args.to_vec(), 1.0)
}
