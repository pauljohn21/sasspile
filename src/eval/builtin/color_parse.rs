//! CSS Color 4 颜色函数解析：lab/lch/oklab/oklch/color()。
//!
//! 从 Sass 值参数解析为 `Value::Color`，sRGB 近似值用 color crate 计算。

use crate::error::{Result, SassError};
use crate::parse::ast::{ColorFormat, Value, Separator};
use im::HashMap;

use super::color_conv_ops::make_color;

/// 解析 CSS Color 4 颜色函数：lab/lch/oklab/oklch/color()。
/// 返回 Value::Color，sRGB 近似值用 color crate 计算。
pub fn parse_color_fn(name: &str, args: &[Value], _kw_args: &HashMap<String, Value>) -> Result<Value> {
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
        && let Value::List(items, Separator::Space, false) = &args[0] {
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
        "display-p3-linear" => ColorFormat::DisplayP3Linear(r, g, b),
        "srgb" => ColorFormat::Srgb(r, g, b),
        "srgb-linear" => ColorFormat::SrgbLinear(r, g, b),
        "a98-rgb" => ColorFormat::A98Rgb(r, g, b),
        "prophoto-rgb" => ColorFormat::ProphotoRgb(r, g, b),
        "rec2020" => ColorFormat::Rec2020(r, g, b),
        "xyz" | "xyz-d65" => ColorFormat::XyzD65(r, g, b),
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
        if let Value::List(items, Separator::Slash, false) = &args[args.len() - 1] {
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
