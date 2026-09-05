//! CSS Color 4 `颜色函数解析：lab/lch/oklab/oklch/color()`。
//!
//! 从 Sass 值参数解析为 `Value::Color`，sRGB 近似值用 color crate 计算。

use crate::error::{Result, SassError};
use crate::eval::error_msgs::{err_not_a_number, err_requires_args};
use crate::parse::ast::{ColorOutput, ColorSpace, Separator, Value};
use std::collections::HashMap;

use super::color_conv_ops::make_color;

/// 解析 CSS Color 4 `颜色函数：lab/lch/oklab/oklch/color()`。
/// 返回 `Value::Color，sRGB` 近似值用 color crate 计算。
pub fn parse_color_fn(
    name: &str,
    args: &[Value],
    _kw_args: &HashMap<String, Value>,
) -> Result<Value> {
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
/// 同时处理 `SlashLiteral` 分隔的列表（lab(L a b / alpha) 等 CSS Level 4 语法）。
fn flatten_space_list(args: &[Value]) -> Vec<Value> {
    match args.len() == 1 {
        true => {
            // SlashLiteral 分隔：lab(L a b / A) → [Space[L,a,b], A]
            match &args[0] {
                Value::List(items, Separator::SlashLiteral | Separator::Slash, false)
                    if items.len() == 2 =>
                {
                    let mut flat = Vec::new();
                    match &items[0] {
                        Value::List(space_items, Separator::Space, false) => {
                            flat.extend(space_items.iter().cloned());
                        }
                        _ => flat.push(items[0].clone()),
                    }
                    flat.push(items[1].clone());
                    flat
                }
                Value::List(items, Separator::Space, false) => items.clone(),
                _ => args.to_vec(),
            }
        }
        false => args.to_vec(),
    }
}

/// 从 Value 提取 f64 数值或 `none`（返回 NaN）。
/// 支持百分比→0-1 转换和 `none` 关键字。
fn extract_num_or_none(v: &Value, scale_pct: bool) -> Result<f64> {
    match v {
        Value::String(s, false) if s == "none" => Ok(f64::NAN),
        Value::Number(n, Some(u)) if u == "%" && scale_pct => Ok(*n / 100.0),
        Value::Number(n, _) => Ok(*n),
        _ => Err(err_not_a_number("value", v)),
    }
}

/// 从百分比 Value 提取原始值（50% → 50.0，不除以100）。
/// 用于 lab/lch 的 L 分量，spec 中 lab(50% ...) 的 50% 就是 50.0。
fn extract_pct_value(v: &Value) -> Result<f64> {
    match v {
        Value::String(s, false) if s == "none" => Ok(f64::NAN),
        Value::Number(n, Some(u)) if u == "%" => Ok(*n),
        Value::Number(n, _) => Ok(*n),
        _ => Err(err_not_a_number("value", v)),
    }
}

/// 从 Value 提取 hue 值（支持 deg 单位）。
fn extract_hue(v: &Value) -> Result<f64> {
    match v {
        Value::String(s, false) if s == "none" => Ok(f64::NAN),
        Value::Number(n, Some(u)) if u == "deg" => Ok(*n),
        Value::Number(n, _) => Ok(*n),
        _ => Err(err_not_a_number("value", v)),
    }
}

/// lab(L% a b [/ alpha])
fn parse_lab(args: &[Value]) -> Result<Value> {
    let (nums, alpha) = split_alpha(args);
    match nums.len() < 3 {
        true => return Err(err_requires_args("lab", 3, nums.len())),
        false => {}
    }
    let l = extract_pct_value(&nums[0])?;
    let a = extract_num_or_none(&nums[1], false)?;
    let b = extract_num_or_none(&nums[2], false)?;
    Ok(make_color(
        ColorSpace::Lab,
        [l, a, b],
        alpha,
        ColorOutput::Auto,
    ))
}

/// lch(L% C Hdeg [/ alpha])
fn parse_lch(args: &[Value]) -> Result<Value> {
    let (nums, alpha) = split_alpha(args);
    match nums.len() < 3 {
        true => return Err(err_requires_args("lch", 3, nums.len())),
        false => {}
    }
    let l = extract_pct_value(&nums[0])?;
    let c = extract_num_or_none(&nums[1], false)?;
    let h = extract_hue(&nums[2])?;
    Ok(make_color(
        ColorSpace::Lch,
        [l, c, h],
        alpha,
        ColorOutput::Auto,
    ))
}

/// oklab(L% a b [/ alpha])
fn parse_oklab(args: &[Value]) -> Result<Value> {
    let (nums, alpha) = split_alpha(args);
    match nums.len() < 3 {
        true => return Err(err_requires_args("oklab", 3, nums.len())),
        false => {}
    }
    let l = extract_num_or_none(&nums[0], true)?;
    let a = extract_num_or_none(&nums[1], false)?;
    let b = extract_num_or_none(&nums[2], false)?;
    Ok(make_color(
        ColorSpace::Oklab,
        [l, a, b],
        alpha,
        ColorOutput::Auto,
    ))
}

/// oklch(L% C Hdeg [/ alpha])
fn parse_oklch(args: &[Value]) -> Result<Value> {
    let (nums, alpha) = split_alpha(args);
    match nums.len() < 3 {
        true => return Err(err_requires_args("oklch", 3, nums.len())),
        false => {}
    }
    let l = extract_num_or_none(&nums[0], true)?;
    let c = extract_num_or_none(&nums[1], false)?;
    let h = extract_hue(&nums[2])?;
    Ok(make_color(
        ColorSpace::Oklch,
        [l, c, h],
        alpha,
        ColorOutput::Auto,
    ))
}

/// color(space r g b [/ alpha])
/// 单参数为 Value::Color 时直接透传。
fn parse_color_space(args: &[Value]) -> Result<Value> {
    // 单参数透传：color(some-color) → some-color
    match args.len() == 1 {
        true => match &args[0] {
            Value::Color(_) => return Ok(args[0].clone()),
            _ => {}
        },
        false => {}
    }
    let (nums, alpha) = split_alpha(args);
    match nums.len() < 4 {
        true => return Err(err_requires_args("color", 4, nums.len())),
        false => {}
    }
    let space = match &nums[0] {
        Value::String(s, _) => s.clone(),
        _ => {
            return Err(SassError::Eval(
                "color() first argument must be a color space name".into(),
            ));
        }
    };
    let r = extract_num_or_none(&nums[1], false)?;
    let g = extract_num_or_none(&nums[2], false)?;
    let b = extract_num_or_none(&nums[3], false)?;
    let cs = ColorSpace::from_str(&space)
        .ok_or_else(|| SassError::Eval(format!("Unknown color space: {space}")))?;
    Ok(make_color(cs, [r, g, b], alpha, ColorOutput::Auto))
}

/// 分离 alpha 分量：参数末尾可能有 / alpha。
/// 返回 (颜色分量, alpha值)。
/// 同时匹配 Slash 和 `SlashLiteral` 分隔符（声明值中 / 被解析为 `SlashLiteral`）。
fn split_alpha(args: &[Value]) -> (Vec<Value>, f64) {
    match args.len() >= 2 {
        true => {
            let last = &args[args.len() - 1];
            match last {
                Value::List(items, Separator::Slash | Separator::SlashLiteral, false)
                    if items.len() == 2 =>
                {
                    let mut nums = args[..args.len() - 1].to_vec();
                    nums.push(items[0].clone());
                    let alpha = match &items[1] {
                        Value::Number(n, Some(u)) if u == "%" => *n / 100.0,
                        Value::Number(n, _) => *n,
                        _ => 1.0,
                    };
                    (nums, alpha)
                }
                _ => (args.to_vec(), 1.0),
            }
        }
        false => (args.to_vec(), 1.0),
    }
}
