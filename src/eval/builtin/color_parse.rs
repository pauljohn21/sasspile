//! CSS Color 4 `颜色函数解析：lab/lch/oklab/oklch/color()`。
//!
//! 从 Sass 值参数解析为 `Value::Color`，sRGB 近似值用 color crate 计算。

use crate::error::{Result, SassError};
use crate::eval::error_msgs::{err_not_a_number, err_requires_args};
use crate::parse::ast::{ColorOutput, ColorSpace, Separator, Value};
use imbl::HashMap;

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

/// 从 Value 提取 f64 数值。
/// 返回 `None` 表示 `calc(infinity/NaN)`——上层据此决定 clamp 行为。
/// - `none` 关键字 → Some(NaN)
/// - `%` 单位 + scale_pct=true → n / 100.0
/// - `%` 单位 + scale_pct=false → n（保留原始数值）
/// - `calc(infinity/‑infinity/NaN)` → None（透传 sentinel）
fn try_extract_num(v: &Value, scale_pct: bool) -> Result<Option<f64>> {
    match v {
        Value::String(s, false) if s == "none" => Ok(Some(f64::NAN)),
        Value::Number(n, Some(u)) if u == "%" && scale_pct => Ok(Some(*n / 100.0)),
        Value::Number(n, Some(u)) if u == "%" => Ok(Some(*n)),
        Value::Number(n, _) => Ok(Some(*n)),
        Value::Calc(s) => {
            let inner = s
                .strip_prefix("calc(")
                .and_then(|s| s.strip_suffix(")"))
                .unwrap_or("");
            match inner {
                "NaN" | "nan" | "infinity" | "-infinity" | "∞" | "-∞" => Ok(None),
                _ => Err(err_not_a_number("value", v)),
            }
        }
        _ => Err(err_not_a_number("value", v)),
    }
}

/// 从 Value 提取 f64 数值或 `none`（返回 NaN）。
/// 用于 color() 空间的 rgb/modern 通道。
fn extract_num_or_none(v: &Value, scale_pct: bool) -> Result<f64> {
    match v {
        Value::String(s, false) if s == "none" => Ok(f64::NAN),
        Value::Number(n, Some(u)) if u == "%" && scale_pct => Ok(*n / 100.0),
        Value::Number(n, _) => Ok(*n),
        _ => Err(err_not_a_number("value", v)),
    }
}

/// 从 calc() 字符串提取对应的 f64 数值。
/// `calc(NaN)` → NaN，`calc(infinity)` → inf，`calc(-infinity)` → -inf。
/// 也支持带单位的变体如 `calc(NaN * 1%)`、`calc(infinity * 1%)`。
pub(crate) fn extract_calc_f64(s: &str) -> Option<f64> {
    let inner = s.strip_prefix("calc(").and_then(|s| s.strip_suffix(")"))?;
    // 先尝试精确匹配
    match inner {
        "NaN" | "nan" => return Some(f64::NAN),
        "infinity" | "∞" => return Some(f64::INFINITY),
        "-infinity" | "-∞" => return Some(f64::NEG_INFINITY),
        _ => {}
    }
    // 带单位的变体：提取特殊值关键字（忽略 `* 1%` 后缀）
    let keyword = inner.split_whitespace().next().unwrap_or("");
    match keyword {
        "NaN" | "nan" => Some(f64::NAN),
        "infinity" | "∞" => Some(f64::INFINITY),
        "-infinity" | "-∞" => Some(f64::NEG_INFINITY),
        _ => None,
    }
}

/// lab 的 a/b 通道：百分比按 max=125 转换（100% → 125）。
/// spec: lab(1% 2% -3%) = lab(1% 2.5 -3.75)
fn extract_lab_ab(v: &Value) -> Result<f64> {
    match v {
        Value::String(s, false) if s == "none" => Ok(f64::NAN),
        Value::Number(n, Some(u)) if u == "%" => Ok(n / 100.0 * 125.0),
        Value::Number(n, _) => {
            // -0 / +0 统一归零；其他数值直接透传
            Ok(if n.abs() < f64::EPSILON { 0.0 } else { *n })
        }
        Value::Calc(s) => extract_calc_f64(s).ok_or_else(|| err_not_a_number("value", v)),
        _ => Err(err_not_a_number("value", v)),
    }
}

/// lch 的 chroma 通道：非负；百分比按 max=150 转换
fn extract_lch_chroma(v: &Value) -> Result<f64> {
    match v {
        Value::String(s, false) if s == "none" => Ok(f64::NAN),
        Value::Number(n, Some(u)) if u == "%" => Ok(n / 100.0 * 150.0),
        Value::Number(n, _) => Ok(if n.abs() < f64::EPSILON { 0.0 } else { n.max(0.0) }),
        Value::Calc(s) => extract_calc_f64(s).ok_or_else(|| err_not_a_number("value", v)),
        _ => Err(err_not_a_number("value", v)),
    }
}

/// oklab/oklch 的 a/b（unitless 0-1）、chroma（0-0.4）。
/// 这些空间的 CIE 通道均为 unitless，percent 按 `max` 比例转换。
fn extract_oklab_ab(v: &Value) -> Result<f64> {
    match v {
        Value::String(s, false) if s == "none" => Ok(f64::NAN),
        Value::Number(n, Some(u)) if u == "%" => Ok(n / 100.0 * 0.4),
        Value::Number(n, _) => Ok(if n.abs() < f64::EPSILON { 0.0 } else { *n }),
        Value::Calc(s) => extract_calc_f64(s).ok_or_else(|| err_not_a_number("value", v)),
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
    if nums.len() < 3 {
        return Err(err_requires_args("lab", 3, nums.len()));
    }
    let l_opt = try_extract_num(&nums[0], false)?;
    let l = match l_opt {
        None => 100.0,
        Some(v) if v.is_nan() => 0.0,
        Some(v) if v.is_infinite() && v > 0.0 => 100.0,
        Some(v) if v.is_infinite() && v < 0.0 => 0.0,
        Some(v) => v.clamp(0.0, 100.0),
    };
    let a = match extract_lab_ab(&nums[1])? {
        v if v.is_nan() => 0.0,
        v => v,
    };
    let b = match extract_lab_ab(&nums[2])? {
        v if v.is_nan() => 0.0,
        v => v,
    };
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
    if nums.len() < 3 {
        return Err(err_requires_args("lch", 3, nums.len()));
    }
    let l_opt = try_extract_num(&nums[0], false)?;
    let l = match l_opt {
        None => 100.0,
        Some(v) if v.is_nan() => 0.0,
        Some(v) if v.is_infinite() && v > 0.0 => 100.0,
        Some(v) if v.is_infinite() && v < 0.0 => 0.0,
        Some(v) => v.clamp(0.0, 100.0),
    };
    let c = match extract_lch_chroma(&nums[1])? {
        v if v.is_nan() => 0.0,
        v if v.is_infinite() => f64::MAX,
        v => v.max(0.0),
    };
    let h = extract_hue(&nums[2]).unwrap_or(f64::NAN);
    Ok(make_color(
        ColorSpace::Lch,
        [l, c, h],
        alpha,
        ColorOutput::Auto,
    ))
}

/// oklab(L a b [/ alpha])
/// L 是 unitless 0-1（percent → n/100）。
fn parse_oklab(args: &[Value]) -> Result<Value> {
    let (nums, alpha) = split_alpha(args);
    if nums.len() < 3 {
        return Err(err_requires_args("oklab", 3, nums.len()));
    }
    let l_opt = try_extract_num(&nums[0], true)?;
    let l = match l_opt {
        None => 1.0,
        Some(v) if v.is_nan() => 0.0,
        Some(v) if v.is_infinite() && v > 0.0 => 1.0,
        Some(v) if v.is_infinite() && v < 0.0 => 0.0,
        Some(v) => v.clamp(0.0, 1.0),
    };
    let a = match extract_oklab_ab(&nums[1])? {
        v if v.is_nan() => 0.0,
        v => v,
    };
    let b = match extract_oklab_ab(&nums[2])? {
        v if v.is_nan() => 0.0,
        v => v,
    };
    Ok(make_color(
        ColorSpace::Oklab,
        [l, a, b],
        alpha,
        ColorOutput::Auto,
    ))
}

/// oklch(L C Hdeg [/ alpha])
fn parse_oklch(args: &[Value]) -> Result<Value> {
    let (nums, alpha) = split_alpha(args);
    if nums.len() < 3 {
        return Err(err_requires_args("oklch", 3, nums.len()));
    }
    let l_opt = try_extract_num(&nums[0], true)?;
    let l = match l_opt {
        None => 1.0,
        Some(v) if v.is_nan() => 0.0,
        Some(v) if v.is_infinite() && v > 0.0 => 1.0,
        Some(v) if v.is_infinite() && v < 0.0 => 0.0,
        Some(v) => v.clamp(0.0, 1.0),
    };
    let c = match extract_oklab_ab(&nums[1])? {
        v if v.is_nan() => 0.0,
        v if v.is_infinite() => f64::MAX,
        v => v.max(0.0),
    };
    let h = extract_hue(&nums[2]).unwrap_or(f64::NAN);
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
