//! Sass Level 4 颜色空间函数：channel / to-space / space / same。
//!
//! 使用 `color` crate v0.3 做底层色彩空间转换计算，
//! sasspile 封装自己的序列化格式输出。
//! 支持 CSS Color Level 4 全部色彩空间。

use crate::error::{Result, SassError};
use crate::parse::ast::{Color, ColorFormat, Value};
use im::HashMap;

use super::super::Evaluator;
use super::color_conv_ops::{color_name, convert_space};

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
            Ok(Some(Value::Bool((a.r - b.r).abs() < 0.5 && (a.g - b.g).abs() < 0.5 && (a.b - b.b).abs() < 0.5 && (a.a - b.a).abs() < 0.0001)))
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
                    let r = c.r / 255.0;
                    let g = c.g / 255.0;
                    let b = c.b / 255.0;
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
                    let r = c.r / 255.0;
                    let g = c.g / 255.0;
                    let bl = c.b / 255.0;
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
                    let r = c.r / 255.0;
                    let g = c.g / 255.0;
                    let bl = c.b / 255.0;
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
                    let r = c.r / 255.0;
                    let g = c.g / 255.0;
                    let bl = c.b / 255.0;
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
                    let r = c.r / 255.0;
                    let g = c.g / 255.0;
                    let bl = c.b / 255.0;
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
    let r = c.r / 255.0;
    let g = c.g / 255.0;
    let b = c.b / 255.0;
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
    let r = c.r / 255.0;
    let g = c.g / 255.0;
    let b = c.b / 255.0;
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
        "red" => Ok(Value::Number(c.r, None)),
        "green" => Ok(Value::Number(c.g, None)),
        "blue" => Ok(Value::Number(c.b, None)),
        "alpha" => Ok(Value::Number(c.a, None)),
        _ => Err(SassError::Eval(format!(
            "$channel: Color {} has no channel named {}.",
            color_name(c), channel
        ))),
    }
}


