#![allow(
    clippy::many_single_char_names,
    clippy::single_char_pattern,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
//! Sass Level 4 颜色空间函数：channel / to-space / space / same。
//!
//! 使用 `color` crate v0.3 做底层色彩空间转换计算，
//! sasspile 封装自己的序列化格式输出。
//! 支持 CSS Color Level 4 全部色彩空间。

use crate::consts::{DEG_UNIT, PCT_SCALE, PERCENT_UNIT, RGB_MAX};
use crate::error::{Result, SassError};
use crate::eval::error_msgs::{
    err_expected_quoted_str_display, err_expected_unquoted_str_display, err_missing_arg,
    err_no_channel, err_not_a_color, err_not_a_string, err_unknown_color_space,
    err_wrong_arg_count_plural,
};
use crate::parse::ast::{Color, ColorSpace, Value};
use std::collections::HashMap;

use super::super::Evaluator;
use super::color_conv_ops::{color_name, convert_space};

/// 从 `kw_args` 中查找参数值，同时支持带 $ 和不带 $ 的 key。
fn kw_get<'a>(kw_args: &'a HashMap<String, Value>, key: &str) -> Option<&'a Value> {
    kw_args.get(key).or_else(|| kw_args.get(&format!("${key}")))
}

/// `color.channel($color, $channel, $space: null)` — 提取颜色通道值。
pub fn channel(args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Option<Value>> {
    let pos_count = args.len();
    let kw_count = kw_args.len();
    if pos_count + kw_count > 3 {
        return Err(err_wrong_arg_count_plural(3, pos_count + kw_count));
    }

    let color_arg = args.first().or_else(|| kw_get(kw_args, "color"));
    let channel_arg = args.get(1).or_else(|| kw_get(kw_args, "channel"));
    let space_arg = args.get(2).or_else(|| kw_get(kw_args, "space"));

    let c = match color_arg {
        Some(Value::Color(c)) => c.clone(),
        Some(v) => return Err(err_not_a_color("color", v)),
        None => return Err(err_missing_arg("color")),
    };

    let ch = match channel_arg {
        Some(Value::String(s, quoted)) => {
            if !quoted {
                return Err(err_expected_quoted_str_display("channel", s));
            }
            s.clone()
        }
        Some(v) => return Err(err_not_a_string("channel", v)),
        None => return Err(err_missing_arg("channel")),
    };

    let space = match space_arg {
        Some(Value::String(s, quoted)) => {
            if *quoted {
                return Err(err_expected_unquoted_str_display("space", s));
            }
            Some(s.clone())
        }
        Some(v) => return Err(err_not_a_string("space", v)),
        None => None,
    };

    get_channel_value(&c, &ch, space.as_deref()).map(Some)
}

/// `color.to-space($color, $space)` — 转换颜色到目标空间。
pub fn to_space(args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Option<Value>> {
    let color_arg = args.first().or_else(|| kw_get(kw_args, "color"));
    let space_arg = args.get(1).or_else(|| kw_get(kw_args, "space"));

    match (color_arg, space_arg) {
        (Some(Value::Color(c)), Some(Value::String(space, _))) => convert_space(c, space).map(Some),
        (Some(v), _) => Err(err_not_a_color("color", v)),
        _ => Err(err_missing_arg("space")),
    }
}

/// `color.space($color)` — 返回颜色的空间名称。
pub fn space(args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Option<Value>> {
    let color_arg = args.first().or_else(|| kw_get(kw_args, "color"));
    match color_arg {
        Some(Value::Color(c)) => {
            let space_name = c.space.as_str();
            Ok(Some(Value::String(space_name.to_string(), false)))
        }
        Some(v) => Err(err_not_a_color("color", v)),
        None => Err(err_missing_arg("color")),
    }
}

/// `color.same($color1, $color2)` — 比较两个颜色是否相同。
pub fn same(args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Option<Value>> {
    let c1 = args.first().or_else(|| kw_get(kw_args, "color1"));
    let c2 = args.get(1).or_else(|| kw_get(kw_args, "color2"));
    match (c1, c2) {
        (Some(Value::Color(a)), Some(Value::Color(b))) => Ok(Some(Value::Bool(
            (a.legacy_rgb[0] - b.legacy_rgb[0]).abs() < 0.5
                && (a.legacy_rgb[1] - b.legacy_rgb[1]).abs() < 0.5
                && (a.legacy_rgb[2] - b.legacy_rgb[2]).abs() < 0.5
                && (a.a - b.a).abs() < 0.0001,
        ))),
        (Some(v), _) => Err(err_not_a_color("color1", v)),
        _ => Err(SassError::Eval(
            "color.same requires 2 color arguments".into(),
        )),
    }
}

/// 从颜色中提取通道值，带正确单位。
fn get_channel_value(c: &Color, channel: &str, space: Option<&str>) -> Result<Value> {
    // alpha 通道对所有颜色空间通用
    if channel == "alpha" {
        return Ok(Value::Number(c.a, None));
    }
    let effective_space = space.unwrap_or_else(|| c.space.as_str());

    match effective_space {
        "rgb" | "srgb" => get_rgb_channel(c, channel),
        "hsl" => {
            let (h, s, l) = if c.space == ColorSpace::Hsl {
                (c.channels[0], c.channels[1], c.channels[2])
            } else {
                Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2])
            };
            match channel {
                "hue" => Ok(Value::Number(h, Some(DEG_UNIT.into()))),
                "saturation" => Ok(Value::Number(s * PCT_SCALE, Some(PERCENT_UNIT.into()))),
                "lightness" => Ok(Value::Number(l * PCT_SCALE, Some(PERCENT_UNIT.into()))),
                _ => Err(err_no_channel(&color_name(c), channel)),
            }
        }
        "hwb" => {
            let (h, w, bk) = if c.space == ColorSpace::Hwb {
                (c.channels[0], c.channels[1], c.channels[2])
            } else {
                let (h, _s, _l) =
                    Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
                let r = c.legacy_rgb[0] / RGB_MAX;
                let g = c.legacy_rgb[1] / RGB_MAX;
                let b = c.legacy_rgb[2] / RGB_MAX;
                let w = r.min(g).min(b);
                let bk = 1.0 - r.max(g).max(b);
                (h, w, bk)
            };
            match channel {
                "hue" => Ok(Value::Number(h, Some(DEG_UNIT.into()))),
                "whiteness" => Ok(Value::Number(w * PCT_SCALE, Some(PERCENT_UNIT.into()))),
                "blackness" => Ok(Value::Number(bk * PCT_SCALE, Some(PERCENT_UNIT.into()))),
                _ => Err(err_no_channel(&color_name(c), channel)),
            }
        }
        "lab" => {
            let (l, a, b) = if c.space == ColorSpace::Lab {
                (c.channels[0], c.channels[1], c.channels[2])
            } else {
                use super::color_conv;
                let r = c.legacy_rgb[0] / RGB_MAX;
                let g = c.legacy_rgb[1] / RGB_MAX;
                let bl = c.legacy_rgb[2] / RGB_MAX;
                color_conv::srgb_to_lab(r, g, bl)
            };
            match channel {
                "lightness" => Ok(Value::Number(l, Some(PERCENT_UNIT.into()))),
                "a" => Ok(Value::Number(a, None)),
                "b" => Ok(Value::Number(b, None)),
                _ => Err(err_no_channel(&color_name(c), channel)),
            }
        }
        "lch" => {
            let (l, ch, h) = if c.space == ColorSpace::Lch {
                (c.channels[0], c.channels[1], c.channels[2])
            } else {
                use super::color_conv;
                let r = c.legacy_rgb[0] / RGB_MAX;
                let g = c.legacy_rgb[1] / RGB_MAX;
                let bl = c.legacy_rgb[2] / RGB_MAX;
                color_conv::srgb_to_lch(r, g, bl)
            };
            match channel {
                "lightness" => Ok(Value::Number(l, Some(PERCENT_UNIT.into()))),
                "chroma" => Ok(Value::Number(ch, None)),
                "hue" => Ok(Value::Number(h, Some(DEG_UNIT.into()))),
                _ => Err(err_no_channel(&color_name(c), channel)),
            }
        }
        "oklab" => {
            let (l, a, b) = if c.space == ColorSpace::Oklab {
                (c.channels[0], c.channels[1], c.channels[2])
            } else {
                use super::color_conv;
                let r = c.legacy_rgb[0] / RGB_MAX;
                let g = c.legacy_rgb[1] / RGB_MAX;
                let bl = c.legacy_rgb[2] / RGB_MAX;
                color_conv::srgb_to_oklab(r, g, bl)
            };
            match channel {
                "lightness" => Ok(Value::Number(l * PCT_SCALE, Some(PERCENT_UNIT.into()))),
                "a" => Ok(Value::Number(a, None)),
                "b" => Ok(Value::Number(b, None)),
                _ => Err(err_no_channel(&color_name(c), channel)),
            }
        }
        "oklch" => {
            let (l, ch, h) = if c.space == ColorSpace::Oklch {
                (c.channels[0], c.channels[1], c.channels[2])
            } else {
                use super::color_conv;
                let r = c.legacy_rgb[0] / RGB_MAX;
                let g = c.legacy_rgb[1] / RGB_MAX;
                let bl = c.legacy_rgb[2] / RGB_MAX;
                color_conv::srgb_to_oklch(r, g, bl)
            };
            match channel {
                "lightness" => Ok(Value::Number(l * PCT_SCALE, Some(PERCENT_UNIT.into()))),
                "chroma" => Ok(Value::Number(ch, None)),
                "hue" => Ok(Value::Number(h, Some(DEG_UNIT.into()))),
                _ => Err(err_no_channel(&color_name(c), channel)),
            }
        }
        "display-p3" | "a98-rgb" | "prophoto-rgb" | "rec2020" | "srgb-linear" => {
            // 这些空间用 red/green/blue 通道名，值为 0-1
            let (r, g, b) = get_normalized_rgb(c, effective_space);
            match channel {
                "red" => Ok(Value::Number(r, None)),
                "green" => Ok(Value::Number(g, None)),
                "blue" => Ok(Value::Number(b, None)),
                _ => Err(err_no_channel(&color_name(c), channel)),
            }
        }
        "xyz" | "xyz-d65" | "xyz-d50" => {
            let (x, y, z) = get_xyz(c, effective_space);
            match channel {
                "x" => Ok(Value::Number(x, None)),
                "y" => Ok(Value::Number(y, None)),
                "z" => Ok(Value::Number(z, None)),
                _ => Err(err_no_channel(&color_name(c), channel)),
            }
        }
        _ => Err(err_unknown_color_space(effective_space)),
    }
}

/// 获取归一化 RGB (0-1) 值。
fn get_normalized_rgb(c: &Color, space: &str) -> (f64, f64, f64) {
    use super::color_conv;
    let r = c.legacy_rgb[0] / RGB_MAX;
    let g = c.legacy_rgb[1] / RGB_MAX;
    let b = c.legacy_rgb[2] / RGB_MAX;
    match space {
        "display-p3" => {
            if c.space == ColorSpace::DisplayP3 {
                (c.channels[0], c.channels[1], c.channels[2])
            } else {
                color_conv::srgb_to_display_p3(r, g, b)
            }
        }
        "srgb-linear" => {
            if c.space == ColorSpace::SrgbLinear {
                (c.channels[0], c.channels[1], c.channels[2])
            } else {
                color_conv::srgb_to_linear_srgb(r, g, b)
            }
        }
        "a98-rgb" => {
            if c.space == ColorSpace::A98Rgb {
                (c.channels[0], c.channels[1], c.channels[2])
            } else {
                color_conv::srgb_to_a98_rgb(r, g, b)
            }
        }
        "prophoto-rgb" => {
            if c.space == ColorSpace::ProphotoRgb {
                (c.channels[0], c.channels[1], c.channels[2])
            } else {
                color_conv::srgb_to_prophoto(r, g, b)
            }
        }
        "rec2020" => {
            if c.space == ColorSpace::Rec2020 {
                (c.channels[0], c.channels[1], c.channels[2])
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
    let r = c.legacy_rgb[0] / RGB_MAX;
    let g = c.legacy_rgb[1] / RGB_MAX;
    let b = c.legacy_rgb[2] / RGB_MAX;
    match space {
        "xyz" | "xyz-d65" => {
            if c.space == ColorSpace::XyzD65 {
                (c.channels[0], c.channels[1], c.channels[2])
            } else {
                color_conv::srgb_to_xyz_d65(r, g, b)
            }
        }
        "xyz-d50" => {
            if c.space == ColorSpace::XyzD50 {
                (c.channels[0], c.channels[1], c.channels[2])
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
        "red" => Ok(Value::Number(c.legacy_rgb[0], None)),
        "green" => Ok(Value::Number(c.legacy_rgb[1], None)),
        "blue" => Ok(Value::Number(c.legacy_rgb[2], None)),
        "alpha" => Ok(Value::Number(c.a, None)),
        _ => Err(err_no_channel(&color_name(c), channel)),
    }
}
