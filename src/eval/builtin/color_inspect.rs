#![allow(
    clippy::many_single_char_names,
    clippy::single_char_pattern,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
//! 颜色通道检查——is-powerless / is-missing / is-in-gamut / is-legacy。
//!
//! `is_channel_powerless` 检查颜色通道是否"无效"（powerless），
//! 参考 CSS Color 规范：HSL hue 在 saturation ≈ 0 时无效等。

use super::super::Evaluator;
use crate::error::{Result, SassError};
use crate::parse::ast::{Color, Value};
use std::collections::HashMap;

/// is-powerless / is-missing / is-in-gamut / is-legacy 分派。
pub fn call(name: &str, args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Option<Value>> {
    match name {
        "is-powerless" => call_is_powerless(args, kw_args),
        "is-missing" => {
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            let channel_arg = args.get(1).or_else(|| kw_args.get("$channel"));
            match (color_arg, channel_arg) {
                (Some(Value::Color(_)), Some(Value::String(_, _))) => Ok(Some(Value::Bool(false))),
                _ => Err(SassError::Eval(
                    "is-missing requires $color and $channel arguments".into(),
                )),
            }
        }
        "is-in-gamut" => {
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            match color_arg {
                Some(Value::Color(_c)) => Ok(Some(Value::Bool(true))),
                _ => Err(SassError::Eval(
                    "is-in-gamut requires $color argument".into(),
                )),
            }
        }
        "is-legacy" => Ok(Some(Value::Bool(true))),
        _ => Ok(None),
    }
}

fn call_is_powerless(args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Option<Value>> {
    let color_arg = args.first().or_else(|| kw_args.get("$color"));
    let channel = args.get(1).or_else(|| kw_args.get("$channel"));
    match (color_arg, channel) {
        (Some(Value::Color(c)), Some(Value::String(ch, _))) => {
            let powerless = is_channel_powerless(c, ch);
            Ok(Some(Value::Bool(powerless)))
        }
        (Some(Value::String(s, _)), Some(Value::String(ch, _))) => {
            let powerless = is_channel_powerless_str(s, ch);
            match powerless {
                Some(b) => Ok(Some(Value::Bool(b))),
                None => Err(SassError::Eval(
                    "is-powerless requires $color and $channel arguments".into(),
                )),
            }
        }
        _ => Err(SassError::Eval(
            "is-powerless requires $color and $channel arguments".into(),
        )),
    }
}

/// 检查颜色通道是否"无效"（powerless）。
fn is_channel_powerless(c: &Color, channel: &str) -> bool {
    let (_h, s, l) = Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
    let max = c.legacy_rgb[0].max(c.legacy_rgb[1]).max(c.legacy_rgb[2]) / 255.0;
    let min = c.legacy_rgb[0].min(c.legacy_rgb[1]).min(c.legacy_rgb[2]) / 255.0;
    let w = min;
    let b = 1.0 - max;
    let w_b_sum = (w + b).min(1.0);
    let eps = 0.0001;
    match channel {
        "hue" => s < eps || w_b_sum >= 1.0 - eps,
        "saturation" => l < eps || (1.0 - l) < eps,
        "lightness" => false,
        "whiteness" => false,
        "blackness" => false,
        "a" | "b" => false,
        _ => false,
    }
}

/// 从字符串形式的 Level 4 颜色判断通道是否 powerless。
fn is_channel_powerless_str(color_str: &str, channel: &str) -> Option<bool> {
    let paren_start = color_str.find('(')?;
    let func_name = color_str[..paren_start].trim();
    let paren_end = color_str.rfind(')')?;
    let inner = &color_str[paren_start + 1..paren_end];
    let parts: Vec<&str> = inner.split_whitespace().collect();
    let eps = 0.0001;

    match func_name {
        "oklch" | "lch" => match channel {
            "hue" => {
                let chroma_str = parts.get(1)?;
                let chroma = parse_percent_or_number(chroma_str)?;
                Some(chroma.abs() < eps)
            }
            "chroma" | "lightness" => Some(false),
            _ => Some(false),
        },
        "oklab" | "lab" => match channel {
            "a" | "b" | "lightness" => Some(false),
            _ => Some(false),
        },
        _ => None,
    }
}

/// 解析 `50%` 或 `0.1` 形式的数值。
fn parse_percent_or_number(s: &str) -> Option<f64> {
    let s = s.trim();
    if let Some(num_str) = s.strip_suffix('%') {
        num_str.parse::<f64>().ok().map(|v| v / 100.0)
    } else {
        s.parse::<f64>().ok()
    }
}
