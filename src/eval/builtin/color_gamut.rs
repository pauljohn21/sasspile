//! `color.to-gamut` 实现。
//!
//! 将颜色映射到目标色域内。
//! 支持的 method:
//! - `clip`: 直接将通道值 clamp 到目标空间范围
//! - `local-minde`: 局部最小 ΔE 映射（简化实现）
//!
//! 参考: CSS Color 4 规范 §12.3 Gamut Mapping

use crate::error::{Result, SassError};
use crate::parse::ast::{Color, ColorFormat, Value};
use std::collections::HashMap;

use super::color_conv;
use super::color_conv_ops::{is_same_space, format_to_srgb_f64};

/// `color.to-gamut($color, $space: null, $method: null)`
pub fn to_gamut(args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Option<Value>> {
    let pos_count = args.len();
    let kw_count = kw_args.len();
    if pos_count + kw_count > 3 {
        return Err(SassError::Eval(format!(
            "Only 3 arguments allowed, but {} were passed.", pos_count + kw_count
        )));
    }

    let color_arg = args.first().or_else(|| kw_args.get("color").or_else(|| kw_args.get("$color")));
    let space_arg = args.get(1).or_else(|| kw_args.get("space").or_else(|| kw_args.get("$space")));
    let method_arg = args.get(2).or_else(|| kw_args.get("method").or_else(|| kw_args.get("$method")));

    let c = match color_arg {
        Some(Value::Color(c)) => c.clone(),
        Some(v) => return Err(SassError::Eval(format!("$color: {} is not a color.", v))),
        None => return Err(SassError::Eval("Missing argument $color.".into())),
    };

    // 解析 $space
    let target_space: Option<String> = match space_arg {
        Some(Value::String(s, quoted)) => {
            if *quoted {
                return Err(SassError::Eval(format!(
                    "$space: Expected \"{}\" to be an unquoted string.", s
                )));
            }
            Some(s.clone())
        }
        Some(Value::Null) => None,
        Some(v) => return Err(SassError::Eval(format!("$space: {} is not a string.", v))),
        None => None,
    };

    // 解析 $method
    let method: String = match method_arg {
        Some(Value::String(s, quoted)) => {
            if *quoted {
                return Err(SassError::Eval(format!(
                    "$method: Expected \"{}\" to be an unquoted string.", s
                )));
            }
            s.clone()
        }
        Some(Value::Null) => "local-minde".to_string(),
        Some(v) => return Err(SassError::Eval(format!("$method: {} is not a string.", v))),
        None => "local-minde".to_string(),
    };

    // 验证 method
    if method != "clip" && method != "local-minde" {
        return Err(SassError::Eval(format!(
            "$method: Expected {} to be exactly \"clip\" or \"local-minde\".", method
        )));
    }

    // 如果指定了 $space，验证是否已知
    if let Some(ref sp) = target_space {
        if !is_known_space(sp) {
            return Err(SassError::Eval(format!(
                "$space: Unknown color space \"{}\".", sp
            )));
        }
    }

    // 确定实际目标空间
    let effective_space = target_space.clone().unwrap_or_else(|| {
        // 默认为颜色自身的空间
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
        }.to_string()
    });

    // 如果颜色已在目标空间中且在色域内，直接返回
    if target_space.is_none() || is_same_space(&c.format, &effective_space) {
        // 检查是否在色域内
        if is_in_gamut(&c, &effective_space) {
            return Ok(Some(Value::Color(c.clone())));
        }
    }

    // 如果指定了空间且与颜色空间不同，需要先转换
    let working_color = if target_space.is_some() && !is_same_space(&c.format, &effective_space) {
        convert_to_space(&c, &effective_space)?
    } else {
        c.clone()
    };

    // 执行 gamut mapping
    let result = if method == "clip" {
        clip_to_gamut(&working_color, &effective_space)
    } else {
        local_minde_mapping(&working_color, &effective_space)
    };

    Ok(Some(result))
}

/// 检查颜色空间名称是否已知。
fn is_known_space(space: &str) -> bool {
    matches!(space,
        "rgb" | "srgb" | "srgb-linear" | "display-p3" | "display-p3-linear"
        | "a98-rgb" | "prophoto-rgb" | "rec2020"
        | "hsl" | "hwb" | "lab" | "lch" | "oklab" | "oklch"
        | "xyz" | "xyz-d65" | "xyz-d50"
    )
}

/// 检查颜色是否在目标色域内。
fn is_in_gamut(c: &Color, space: &str) -> bool {
    match space {
        "rgb" | "srgb" | "display-p3" | "a98-rgb" | "prophoto-rgb" | "rec2020" => {
            let (r, g, b) = get_rgb_channels(c, space);
            r >= 0.0 && r <= 1.0 && g >= 0.0 && g <= 1.0 && b >= 0.0 && b <= 1.0
        }
        "srgb-linear" | "display-p3-linear" => {
            let (r, g, b) = get_rgb_channels(c, space);
            r >= 0.0 && g >= 0.0 && b >= 0.0
        }
        "hsl" => {
            let (_h, s, l) = match c.format {
                ColorFormat::Hsl(h, s, l) => (h, s, l),
                _ => return true, // legacy 颜色始终在色域内
            };
            s >= 0.0 && s <= 1.0 && l >= 0.0 && l <= 1.0
        }
        "hwb" => {
            let (_h, w, b) = match c.format {
                ColorFormat::Hwb(h, w, b) => (h, w, b),
                _ => return true,
            };
            w >= 0.0 && w <= 1.0 && b >= 0.0 && b <= 1.0 && (w + b) <= 1.0
        }
        "lab" | "oklab" | "lch" | "oklch" | "xyz" | "xyz-d65" | "xyz-d50" => {
            // 这些空间没有明确色域限制
            true
        }
        _ => true,
    }
}

/// 获取 RGB 通道值。
fn get_rgb_channels(c: &Color, space: &str) -> (f64, f64, f64) {
    match c.format {
        ColorFormat::Srgb(r, g, b) | ColorFormat::DisplayP3(r, g, b)
        | ColorFormat::A98Rgb(r, g, b) | ColorFormat::ProphotoRgb(r, g, b)
        | ColorFormat::Rec2020(r, g, b) => (r, g, b),
        ColorFormat::SrgbLinear(r, g, b) | ColorFormat::DisplayP3Linear(r, g, b) => (r, g, b),
        _ => {
            // Legacy RGB → 0-1
            if space == "rgb" || space == "srgb" {
                (c.r / 255.0, c.g / 255.0, c.b / 255.0)
            } else {
                format_to_srgb_f64(&c.format, c.r, c.g, c.b)
            }
        }
    }
}

/// clip 方法：直接将通道值 clamp 到 [0, 1]。
fn clip_to_gamut(c: &Color, space: &str) -> Value {
    match space {
        "rgb" | "srgb" | "display-p3" | "a98-rgb" | "prophoto-rgb" | "rec2020" => {
            let (r, g, b) = get_rgb_channels(c, space);
            let (r, g, b) = (r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0));
            let fmt = c.format.clone_with(r, g, b);
            Value::Color(Color::rgba_fmt(c.r, c.g, c.b, c.a, fmt))
        }
        "hsl" => {
            let (h, s, l) = match c.format {
                ColorFormat::Hsl(h, s, l) => (h, s.clamp(0.0, 1.0), l.clamp(0.0, 1.0)),
                _ => (0.0, 0.0, 0.0),
            };
            Value::Color(Color::rgba_fmt(c.r, c.g, c.b, c.a, ColorFormat::Hsl(h, s, l)))
        }
        "hwb" => {
            let (h, w, b) = match c.format {
                ColorFormat::Hwb(h, w, b) => {
                    let w = w.clamp(0.0, 1.0);
                    let b = b.clamp(0.0, 1.0);
                    let sum = w + b;
                    if sum > 1.0 {
                        (h, w / sum, b / sum)
                    } else {
                        (h, w, b)
                    }
                }
                _ => (0.0, 0.0, 0.0),
            };
            Value::Color(Color::rgba_fmt(c.r, c.g, c.b, c.a, ColorFormat::Hwb(h, w, b)))
        }
        _ => Value::Color(c.clone()),
    }
}

/// local-minde 方法：局部最小 ΔE 映射。
/// 简化实现：通过减小 chroma 直到颜色在色域内。
fn local_minde_mapping(c: &Color, space: &str) -> Value {
    // 先检查是否在色域内
    if is_in_gamut(c, space) {
        return Value::Color(c.clone());
    }

    match space {
        "rgb" | "srgb" | "display-p3" | "a98-rgb" | "prophoto-rgb" | "rec2020" => {
            // 对于 RGB 空间，使用 Oklch 空间减小 chroma
            local_minde_rgb(c, space)
        }
        "hsl" | "hwb" => {
            // HSL/HWB 直接 clip
            clip_to_gamut(c, space)
        }
        _ => Value::Color(c.clone()),
    }
}

/// RGB 空间的 local-minde 映射。
/// 在 Oklch 空间中减小 chroma，直到颜色落入 sRGB 色域。
fn local_minde_rgb(c: &Color, space: &str) -> Value {
    let (r, g, b) = get_rgb_channels(c, space);

    // 转换到 Oklch
    let (l_ok, c_ok, h_ok) = color_conv::srgb_to_oklch(r, g, b);

    // 二分搜索：找到最大 chroma 使得颜色在色域内
    let mut lo = 0.0_f64;
    let mut hi = c_ok;
    let epsilon = 0.0001;

    for _ in 0..50 {
        let mid = (lo + hi) / 2.0;
        let (r_test, g_test, b_test) = color_conv::oklch_to_srgb(l_ok, mid, h_ok);
        if r_test >= 0.0 && r_test <= 1.0 && g_test >= 0.0 && g_test <= 1.0 && b_test >= 0.0 && b_test <= 1.0 {
            lo = mid;
        } else {
            hi = mid;
        }
        if (hi - lo).abs() < epsilon {
            break;
        }
    }

    let final_chroma = lo;
    let (r_new, g_new, b_new) = color_conv::oklch_to_srgb(l_ok, final_chroma, h_ok);
    let (r_new, g_new, b_new) = (
        r_new.clamp(0.0, 1.0),
        g_new.clamp(0.0, 1.0),
        b_new.clamp(0.0, 1.0),
    );

    let fmt = c.format.clone_with(r_new, g_new, b_new);
    Value::Color(Color::rgba_fmt(c.r, c.g, c.b, c.a, fmt))
}

/// 转换颜色到目标空间。
fn convert_to_space(c: &Color, target_space: &str) -> Result<Color> {
    use super::color_conv_ops::convert_space;
    match convert_space(c, target_space)? {
        Value::Color(new_c) => Ok(new_c),
        _ => Err(SassError::Eval("Color conversion failed".into())),
    }
}
