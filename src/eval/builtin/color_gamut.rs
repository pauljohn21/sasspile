//! `color.to-gamut` 实现。
//!
//! 将颜色映射到目标色域内。
//! 支持的 method:
//! - `clip`: 直接将通道值 clamp 到目标空间范围
//! - `local-minde`: 局部最小 ΔE 映射（简化实现）
//!
//! 参考: CSS Color 4 规范 §12.3 Gamut Mapping

use crate::error::{Result, SassError};
use crate::eval::error_msgs::{
    err_expected_exactly, err_expected_unquoted_str_display, err_missing_arg, err_not_a_color,
    err_not_a_string, err_unknown_color_space_quoted, err_wrong_arg_count_plural,
};
use crate::parse::ast::{Color, ColorSpace, Value};
use std::collections::HashMap;

use super::color_conv;
use super::color_conv_ops::{is_same_space, space_to_srgb_f64};

/// `color.to-gamut($color, $space: null, $method: null)`
pub fn to_gamut(args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Option<Value>> {
    let pos_count = args.len();
    let kw_count = kw_args.len();
    match pos_count + kw_count > 3 {
        true => return Err(err_wrong_arg_count_plural(3, pos_count + kw_count)),
        false => {}
    }

    let color_arg = args
        .first()
        .or_else(|| kw_args.get("color").or_else(|| kw_args.get("$color")));
    let space_arg = args
        .get(1)
        .or_else(|| kw_args.get("space").or_else(|| kw_args.get("space")));
    let method_arg = args
        .get(2)
        .or_else(|| kw_args.get("method").or_else(|| kw_args.get("method")));

    let c = match color_arg {
        Some(Value::Color(c)) => c.clone(),
        Some(v) => return Err(err_not_a_color("color", v)),
        None => return Err(err_missing_arg("color")),
    };

    // 解析 $space
    let target_space: Option<String> = match space_arg {
        Some(Value::String(s, quoted)) => {
            match *quoted {
                true => return Err(err_expected_unquoted_str_display("space", s)),
                false => {}
            }
            Some(s.clone())
        }
        Some(Value::Null) => None,
        Some(v) => return Err(err_not_a_string("space", v)),
        None => None,
    };

    // 解析 $method
    let method: String = match method_arg {
        Some(Value::String(s, quoted)) => {
            match *quoted {
                true => return Err(err_expected_unquoted_str_display("method", s)),
                false => {}
            }
            s.clone()
        }
        Some(Value::Null) => "local-minde".to_string(),
        Some(v) => return Err(err_not_a_string("method", v)),
        None => "local-minde".to_string(),
    };

    // 验证 method
    match method != "clip" && method != "local-minde" {
        true => return Err(err_expected_exactly(
            "method",
            &method,
            &["clip", "local-minde"],
        )),
        false => {}
    }

    // 如果指定了 $space，验证是否已知
    if let Some(ref sp) = target_space
        && !is_known_space(sp)
    {
        return Err(err_unknown_color_space_quoted(sp));
    }

    // 确定实际目标空间
    let effective_space = target_space.clone().unwrap_or_else(|| {
        // 默认为颜色自身的空间
        c.space.as_str().to_string()
    });

    // 如果颜色已在目标空间中且在色域内，直接返回
    match target_space.is_none() || is_same_space(c.space, &effective_space) {
        true => {
            match is_in_gamut(&c, &effective_space) {
                true => return Ok(Some(Value::Color(c.clone()))),
                false => {}
            }
        }
        false => {}
    }

    // 如果指定了空间且与颜色空间不同，需要先转换
    let working_color = if target_space.is_some() && !is_same_space(c.space, &effective_space) {
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
    matches!(
        space,
        "rgb"
            | "srgb"
            | "srgb-linear"
            | "display-p3"
            | "display-p3-linear"
            | "a98-rgb"
            | "prophoto-rgb"
            | "rec2020"
            | "hsl"
            | "hwb"
            | "lab"
            | "lch"
            | "oklab"
            | "oklch"
            | "xyz"
            | "xyz-d65"
            | "xyz-d50"
    )
}

/// 检查颜色是否在目标色域内。
fn is_in_gamut(c: &Color, space: &str) -> bool {
    match space {
        "rgb" | "srgb" | "display-p3" | "a98-rgb" | "prophoto-rgb" | "rec2020" => {
            let (r, g, b) = get_rgb_channels(c, space);
            (0.0..=1.0).contains(&r) && (0.0..=1.0).contains(&g) && (0.0..=1.0).contains(&b)
        }
        "srgb-linear" | "display-p3-linear" => {
            let (r, g, b) = get_rgb_channels(c, space);
            r >= 0.0 && g >= 0.0 && b >= 0.0
        }
        "hsl" => match c.space == ColorSpace::Hsl {
            true => {
                let (_, s, l) = (c.channels[0], c.channels[1], c.channels[2]);
                (0.0..=1.0).contains(&s) && (0.0..=1.0).contains(&l)
            }
            false => true,
        }
        "hwb" => match c.space == ColorSpace::Hwb {
            true => {
                let (_, w, b) = (c.channels[0], c.channels[1], c.channels[2]);
                (0.0..=1.0).contains(&w) && (0.0..=1.0).contains(&b) && (w + b) <= 1.0
            }
            false => true,
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
    match c.space {
        ColorSpace::Srgb
        | ColorSpace::DisplayP3
        | ColorSpace::A98Rgb
        | ColorSpace::ProphotoRgb
        | ColorSpace::Rec2020 => (c.channels[0], c.channels[1], c.channels[2]),
        ColorSpace::SrgbLinear | ColorSpace::DisplayP3Linear => {
            (c.channels[0], c.channels[1], c.channels[2])
        }
        _ => {
            // Legacy RGB → 0-1
            match space == "rgb" || space == "srgb" {
                true => (
                    c.legacy_rgb[0] / 255.0,
                    c.legacy_rgb[1] / 255.0,
                    c.legacy_rgb[2] / 255.0,
                ),
                false => space_to_srgb_f64(c.space, c.channels, c.legacy_rgb),
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
            Value::Color(c.clone_with_rgb(r, g, b))
        }
        "hsl" => {
            let (h, s, l) = if c.space == ColorSpace::Hsl {
                (
                    c.channels[0],
                    c.channels[1].clamp(0.0, 1.0),
                    c.channels[2].clamp(0.0, 1.0),
                )
            } else {
                (0.0, 0.0, 0.0)
            };
            Value::Color(Color::with_hsl(h, s, l, c.a, c.output, c.legacy_rgb))
        }
        "hwb" => {
            let (h, w, b) = if c.space == ColorSpace::Hwb {
                let (h, w, b) = (c.channels[0], c.channels[1], c.channels[2]);
                let w = w.clamp(0.0, 1.0);
                let b = b.clamp(0.0, 1.0);
                let sum = w + b;
                match sum > 1.0 {
                    true => (h, w / sum, b / sum),
                    false => (h, w, b),
                }
            } else {
                (0.0, 0.0, 0.0)
            };
            Value::Color(Color::with_hwb(h, w, b, c.a, c.legacy_rgb))
        }
        _ => Value::Color(c.clone()),
    }
}

/// local-minde 方法：局部最小 ΔE 映射。
/// 简化实现：通过减小 chroma 直到颜色在色域内。
fn local_minde_mapping(c: &Color, space: &str) -> Value {
    // 先检查是否在色域内
    match is_in_gamut(c, space) {
        true => return Value::Color(c.clone()),
        false => {}
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
        let mid = f64::midpoint(lo, hi);
        let (r_test, g_test, b_test) = color_conv::oklch_to_srgb(l_ok, mid, h_ok);
        match (0.0..=1.0).contains(&r_test)
            && (0.0..=1.0).contains(&g_test)
            && (0.0..=1.0).contains(&b_test)
        {
            true => { lo = mid; }
            false => { hi = mid; }
        }
        match (hi - lo).abs() < epsilon {
            true => break,
            false => {}
        }
    }

    let final_chroma = lo;
    let (r_new, g_new, b_new) = color_conv::oklch_to_srgb(l_ok, final_chroma, h_ok);
    let (r_new, g_new, b_new) = (
        r_new.clamp(0.0, 1.0),
        g_new.clamp(0.0, 1.0),
        b_new.clamp(0.0, 1.0),
    );

    Value::Color(c.clone_with_rgb(r_new, g_new, b_new))
}

/// 转换颜色到目标空间。
fn convert_to_space(c: &Color, target_space: &str) -> Result<Color> {
    use super::color_conv_ops::convert_space;
    match convert_space(c, target_space)? {
        Value::Color(new_c) => Ok(new_c),
        _ => Err(SassError::Eval("Color conversion failed".into())),
    }
}
