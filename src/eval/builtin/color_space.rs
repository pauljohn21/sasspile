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
    let effective_space = space.unwrap_or_else(|| {
        match c.format {
            ColorFormat::Hsl(_, _, _) => "hsl",
            ColorFormat::Hwb(_, _, _) => "hwb",
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
        _ => Err(SassError::Eval(format!(
            "$space: Unknown color space: {effective_space}."
        ))),
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

/// 转换颜色到目标空间，用 `color` crate 做底层计算。
fn convert_space(c: &Color, target_space: &str) -> Result<Value> {
    match target_space {
        "rgb" | "srgb" => {
            Ok(Value::Color(Color::rgba_fmt(c.r, c.g, c.b, c.a, ColorFormat::Auto)))
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
                    let (h, _s, _l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);
                    let r = c.r as f64 / 255.0;
                    let g = c.g as f64 / 255.0;
                    let b = c.b as f64 / 255.0;
                    let w = r.min(g).min(b);
                    let bk = 1.0 - r.max(g).max(b);
                    (h, w, bk)
                }
            };
            Ok(Value::Color(Color::rgba_fmt(c.r, c.g, c.b, c.a, ColorFormat::Hwb(h, w, bk))))
        }
        // 高级空间用 color crate 转换，但 sasspile 暂不支持这些空间的序列化
        // 先报错，后续实现 lab()/oklch() 等函数后可支持
        _ => Err(SassError::Eval(format!("Unknown color space: {target_space}"))),
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

/// 生成颜色的显示名称（用于错误消息）。
fn color_name(c: &Color) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
}
