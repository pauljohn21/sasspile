#![allow(
    clippy::many_single_char_names,
    clippy::single_char_pattern,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
//! `color.adjust` / `color.change` / `color.scale` 实现。
//!
//! 支持所有 CSS Color 4 颜色空间：
//! - Legacy: RGB, HSL, HWB
//! - Modern: Lab, Lch, Oklab, Oklch, `DisplayP3`, sRGB, sRGB-Linear, etc.
//!
//! 现代空间直接在 channels 中修改通道值，保留原始格式输出。

use crate::error::{Result, SassError};
use crate::parse::ast::{Color, ColorOutput, ColorSpace, Value};
use std::collections::HashMap;

use super::super::Evaluator;

/// 从 `kw_args` 中提取数值参数。
fn get_num(kw_args: &HashMap<String, Value>, key: &str) -> Result<Option<f64>> {
    match kw_args.get(key) {
        Some(Value::Number(n, _)) => Ok(Some(*n)),
        Some(_) => Err(SassError::Eval(format!("{key} requires a number"))),
        None => Ok(None),
    }
}

/// 提取百分比参数，返回 0-1 范围的值。
/// 对于带 `%` 单位的值，除以 100；对于无单位值，直接返回。
fn get_pct_or_num(kw_args: &HashMap<String, Value>, key: &str) -> Result<Option<f64>> {
    match kw_args.get(key) {
        Some(Value::Number(n, Some(unit))) if unit == "%" => Ok(Some(*n / 100.0)),
        Some(Value::Number(n, None)) => Ok(Some(*n)),
        Some(Value::Number(n, Some(_))) => Ok(Some(*n)), // 其他单位直接用值
        Some(_) => Err(SassError::Eval(format!("{key} requires a number"))),
        None => Ok(None),
    }
}

/// 统一处理关键字参数应用——消除 `let mut x = ...; if let Some(v) = ... { x = f(x, v) }` 重复模式。
fn apply_kw(
    initial: f64,
    kw: &HashMap<String, Value>,
    key: &str,
    f: impl Fn(f64, f64) -> f64,
) -> Result<f64> {
    Ok(match get_num(kw, key)? {
        Some(v) => f(initial, v),
        None => initial,
    })
}

/// 同上，但使用 `get_pct_or_num` 提取百分比参数。
fn apply_pct_kw(
    initial: f64,
    kw: &HashMap<String, Value>,
    key: &str,
    f: impl Fn(f64, f64) -> f64,
) -> Result<f64> {
    Ok(match get_pct_or_num(kw, key)? {
        Some(v) => f(initial, v),
        None => initial,
    })
}

/// `color.adjust($color, $kwargs)` — 调整颜色通道（增量）。
pub fn adjust_color(args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Value> {
    let c = match args.first().or_else(|| kw_args.get("$color")) {
        Some(Value::Color(c)) => c.clone(),
        Some(v) => return Err(SassError::Eval(format!("$color: {v} is not a color."))),
        None => return Err(SassError::Eval("Missing argument $color.".into())),
    };

    // 检查是否有现代颜色空间的通道参数
    let modern_channels = ["lightness", "chroma", "a", "b", "x", "y", "z"];
    let has_modern = modern_channels.iter().any(|ch| kw_args.contains_key(*ch));

    // 根据颜色空间选择处理路径
    match c.space {
        ColorSpace::Oklch => adjust_oklch(&c, kw_args),
        ColorSpace::Oklab => adjust_oklab(&c, kw_args),
        ColorSpace::Lch => adjust_lch(&c, kw_args),
        ColorSpace::Lab => adjust_lab(&c, kw_args),
        ColorSpace::DisplayP3
        | ColorSpace::Srgb
        | ColorSpace::SrgbLinear
        | ColorSpace::DisplayP3Linear
        | ColorSpace::A98Rgb
        | ColorSpace::ProphotoRgb
        | ColorSpace::Rec2020
        | ColorSpace::XyzD65
        | ColorSpace::XyzD50 => adjust_modern_rgb_space(&c, kw_args),
        _ if has_modern => {
            // Legacy 颜色但有现代通道参数——需要先转换
            // 对于 HSL/HWB 颜色，lightness 参数仍然走 legacy 路径
            adjust_legacy(&c, kw_args)
        }
        _ => adjust_legacy(&c, kw_args),
    }
}

/// `color.change($color, $kwargs)` — 设置颜色通道（绝对值）。
pub fn change_color(args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Value> {
    let c = match args.first().or_else(|| kw_args.get("$color")) {
        Some(Value::Color(c)) => c.clone(),
        Some(v) => return Err(SassError::Eval(format!("$color: {v} is not a color."))),
        None => return Err(SassError::Eval("Missing argument $color.".into())),
    };

    match c.space {
        ColorSpace::Oklch => change_oklch(&c, kw_args),
        ColorSpace::Oklab => change_oklab(&c, kw_args),
        ColorSpace::Lch => change_lch(&c, kw_args),
        ColorSpace::Lab => change_lab(&c, kw_args),
        ColorSpace::DisplayP3
        | ColorSpace::Srgb
        | ColorSpace::SrgbLinear
        | ColorSpace::DisplayP3Linear
        | ColorSpace::A98Rgb
        | ColorSpace::ProphotoRgb
        | ColorSpace::Rec2020
        | ColorSpace::XyzD65
        | ColorSpace::XyzD50 => change_modern_rgb_space(&c, kw_args),
        _ => change_legacy(&c, kw_args),
    }
}

/// `color.scale($color, $kwargs)` — 按比例缩放颜色通道。
pub fn scale_color(args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Value> {
    let c = match args.first().or_else(|| kw_args.get("$color")) {
        Some(Value::Color(c)) => c.clone(),
        Some(v) => return Err(SassError::Eval(format!("$color: {v} is not a color."))),
        None => return Err(SassError::Eval("Missing argument $color.".into())),
    };

    match c.space {
        ColorSpace::Oklch => scale_oklch(&c, kw_args),
        ColorSpace::Oklab => scale_oklab(&c, kw_args),
        ColorSpace::Lch => scale_lch(&c, kw_args),
        ColorSpace::Lab => scale_lab(&c, kw_args),
        ColorSpace::DisplayP3
        | ColorSpace::Srgb
        | ColorSpace::SrgbLinear
        | ColorSpace::DisplayP3Linear
        | ColorSpace::A98Rgb
        | ColorSpace::ProphotoRgb
        | ColorSpace::Rec2020
        | ColorSpace::XyzD65
        | ColorSpace::XyzD50 => scale_modern_rgb_space(&c, kw_args),
        _ => scale_legacy(&c, kw_args),
    }
}

// ── Oklch ──

fn adjust_oklch(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (l, ch, h) = (c.channels[0], c.channels[1], c.channels[2]);
    let l = apply_pct_kw(l, kw_args, "lightness", |v, d| (v + d).clamp(0.0, 1.0))?;
    let ch = apply_kw(ch, kw_args, "chroma", |v, d| (v + d).max(0.0))?;
    let h = apply_kw(h, kw_args, "hue", |v, d| (v + d).rem_euclid(360.0))?;
    let a = apply_kw(c.a, kw_args, "alpha", |v, d| (v + d).clamp(0.0, 1.0))?;

    Ok(Value::Color(Color::with_space(
        ColorSpace::Oklch,
        [l, ch, h],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

fn change_oklch(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (l, ch, h) = (c.channels[0], c.channels[1], c.channels[2]);
    let l = apply_pct_kw(l, kw_args, "lightness", |_v, d| d.clamp(0.0, 1.0))?;
    let ch = apply_kw(ch, kw_args, "chroma", |_v, d| d.max(0.0))?;
    let h = apply_kw(h, kw_args, "hue", |_v, d| d.rem_euclid(360.0))?;
    let a = apply_kw(c.a, kw_args, "alpha", |_v, d| d.clamp(0.0, 1.0))?;

    Ok(Value::Color(Color::with_space(
        ColorSpace::Oklch,
        [l, ch, h],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

fn scale_oklch(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (l, ch, h) = (c.channels[0], c.channels[1], c.channels[2]);
    let scale_val = |val: f64, max: f64, key: &str| -> Result<f64> {
        if let Some(Value::Number(n, _)) = kw_args.get(key) {
            let pct = *n / 100.0;
            if pct >= 0.0 {
                Ok(val + (max - val) * pct)
            } else {
                Ok(val + val * pct)
            }
        } else {
            Ok(val)
        }
    };
    let l = scale_val(l, 1.0, "lightness")?.clamp(0.0, 1.0);
    let ch = scale_val(ch, f64::MAX, "chroma")?.max(0.0);
    let a = scale_val(c.a, 1.0, "alpha")?.clamp(0.0, 1.0);

    Ok(Value::Color(Color::with_space(
        ColorSpace::Oklch,
        [l, ch, h],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

// ── Oklab ──

fn adjust_oklab(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (l, a_v, b_v) = (c.channels[0], c.channels[1], c.channels[2]);
    let l = apply_pct_kw(l, kw_args, "lightness", |v, d| (v + d).clamp(0.0, 1.0))?;
    let a_v = apply_kw(a_v, kw_args, "a", |v, d| v + d)?;
    let b_v = apply_kw(b_v, kw_args, "b", |v, d| v + d)?;
    let a = apply_kw(c.a, kw_args, "alpha", |v, d| (v + d).clamp(0.0, 1.0))?;

    Ok(Value::Color(Color::with_space(
        ColorSpace::Oklab,
        [l, a_v, b_v],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

fn change_oklab(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (l, a_v, b_v) = (c.channels[0], c.channels[1], c.channels[2]);
    let l = apply_pct_kw(l, kw_args, "lightness", |_v, d| d.clamp(0.0, 1.0))?;
    let a_v = apply_kw(a_v, kw_args, "a", |_v, d| d)?;
    let b_v = apply_kw(b_v, kw_args, "b", |_v, d| d)?;
    let a = apply_kw(c.a, kw_args, "alpha", |_v, d| d.clamp(0.0, 1.0))?;

    Ok(Value::Color(Color::with_space(
        ColorSpace::Oklab,
        [l, a_v, b_v],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

fn scale_oklab(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (l, a_v, b_v) = (c.channels[0], c.channels[1], c.channels[2]);
    let scale_val = |val: f64, max: f64, key: &str| -> Result<f64> {
        if let Some(Value::Number(n, _)) = kw_args.get(key) {
            let pct = *n / 100.0;
            if pct >= 0.0 {
                Ok(val + (max - val) * pct)
            } else {
                Ok(val + val * pct)
            }
        } else {
            Ok(val)
        }
    };
    let l = scale_val(l, 1.0, "lightness")?.clamp(0.0, 1.0);
    let a_v = scale_val(a_v, if a_v >= 0.0 { 0.5 } else { -0.5 }, "a")?;
    let b_v = scale_val(b_v, if b_v >= 0.0 { 0.5 } else { -0.5 }, "b")?;
    let a = scale_val(c.a, 1.0, "alpha")?.clamp(0.0, 1.0);

    Ok(Value::Color(Color::with_space(
        ColorSpace::Oklab,
        [l, a_v, b_v],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

// ── Lch ──

fn adjust_lch(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (l, ch, h) = (c.channels[0], c.channels[1], c.channels[2]);
    let l = apply_pct_kw(l, kw_args, "lightness", |v, d| {
        (v + d * 100.0).clamp(0.0, 100.0)
    })?;
    let ch = apply_kw(ch, kw_args, "chroma", |v, d| (v + d).max(0.0))?;
    let h = apply_kw(h, kw_args, "hue", |v, d| (v + d).rem_euclid(360.0))?;
    let a = apply_kw(c.a, kw_args, "alpha", |v, d| (v + d).clamp(0.0, 1.0))?;

    Ok(Value::Color(Color::with_space(
        ColorSpace::Lch,
        [l, ch, h],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

fn change_lch(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (l, ch, h) = (c.channels[0], c.channels[1], c.channels[2]);
    let l = apply_pct_kw(l, kw_args, "lightness", |_v, d| {
        (d * 100.0).clamp(0.0, 100.0)
    })?;
    let ch = apply_kw(ch, kw_args, "chroma", |_v, d| d.max(0.0))?;
    let h = apply_kw(h, kw_args, "hue", |_v, d| d.rem_euclid(360.0))?;
    let a = apply_kw(c.a, kw_args, "alpha", |_v, d| d.clamp(0.0, 1.0))?;

    Ok(Value::Color(Color::with_space(
        ColorSpace::Lch,
        [l, ch, h],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

fn scale_lch(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (l, ch, h) = (c.channels[0], c.channels[1], c.channels[2]);
    let scale_val = |val: f64, max: f64, key: &str| -> Result<f64> {
        if let Some(Value::Number(n, _)) = kw_args.get(key) {
            let pct = *n / 100.0;
            if pct >= 0.0 {
                Ok(val + (max - val) * pct)
            } else {
                Ok(val + val * pct)
            }
        } else {
            Ok(val)
        }
    };
    let l = scale_val(l, 100.0, "lightness")?.clamp(0.0, 100.0);
    let ch = scale_val(ch, f64::MAX, "chroma")?.max(0.0);
    let a = scale_val(c.a, 1.0, "alpha")?.clamp(0.0, 1.0);

    Ok(Value::Color(Color::with_space(
        ColorSpace::Lch,
        [l, ch, h],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

// ── Lab ──

fn adjust_lab(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (l, a_v, b_v) = (c.channels[0], c.channels[1], c.channels[2]);
    let l = apply_pct_kw(l, kw_args, "lightness", |v, d| {
        (v + d * 100.0).clamp(0.0, 100.0)
    })?;
    let a_v = apply_kw(a_v, kw_args, "a", |v, d| v + d)?;
    let b_v = apply_kw(b_v, kw_args, "b", |v, d| v + d)?;
    let a = apply_kw(c.a, kw_args, "alpha", |v, d| (v + d).clamp(0.0, 1.0))?;

    Ok(Value::Color(Color::with_space(
        ColorSpace::Lab,
        [l, a_v, b_v],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

fn change_lab(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (l, a_v, b_v) = (c.channels[0], c.channels[1], c.channels[2]);
    let l = apply_pct_kw(l, kw_args, "lightness", |_v, d| {
        (d * 100.0).clamp(0.0, 100.0)
    })?;
    let a_v = apply_kw(a_v, kw_args, "a", |_v, d| d)?;
    let b_v = apply_kw(b_v, kw_args, "b", |_v, d| d)?;
    let a = apply_kw(c.a, kw_args, "alpha", |_v, d| d.clamp(0.0, 1.0))?;

    Ok(Value::Color(Color::with_space(
        ColorSpace::Lab,
        [l, a_v, b_v],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

fn scale_lab(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (l, a_v, b_v) = (c.channels[0], c.channels[1], c.channels[2]);
    let scale_val = |val: f64, max: f64, key: &str| -> Result<f64> {
        if let Some(Value::Number(n, _)) = kw_args.get(key) {
            let pct = *n / 100.0;
            if pct >= 0.0 {
                Ok(val + (max - val) * pct)
            } else {
                Ok(val + val * pct)
            }
        } else {
            Ok(val)
        }
    };
    let l = scale_val(l, 100.0, "lightness")?.clamp(0.0, 100.0);
    let a_v = scale_val(a_v, if a_v >= 0.0 { 125.0 } else { -125.0 }, "a")?;
    let b_v = scale_val(b_v, if b_v >= 0.0 { 125.0 } else { -125.0 }, "b")?;
    let a = scale_val(c.a, 1.0, "alpha")?.clamp(0.0, 1.0);

    Ok(Value::Color(Color::with_space(
        ColorSpace::Lab,
        [l, a_v, b_v],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

// ── Modern RGB 空间 (DisplayP3, sRGB, A98, ProPhoto, Rec2020) ──

fn get_rgb_channels(c: &Color) -> (f64, f64, f64) {
    (c.channels[0], c.channels[1], c.channels[2])
}

fn adjust_modern_rgb_space(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (r, g, b) = get_rgb_channels(c);
    let r = apply_kw(r, kw_args, "red", |v, d| v + d)?;
    let g = apply_kw(g, kw_args, "green", |v, d| v + d)?;
    let b = apply_kw(b, kw_args, "blue", |v, d| v + d)?;
    let _a = apply_kw(c.a, kw_args, "alpha", |v, d| (v + d).clamp(0.0, 1.0))?;

    Ok(Value::Color(c.clone_with_rgb(r, g, b)))
}

fn change_modern_rgb_space(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (r, g, b) = get_rgb_channels(c);
    let r = apply_kw(r, kw_args, "red", |_v, d| d)?;
    let g = apply_kw(g, kw_args, "green", |_v, d| d)?;
    let b = apply_kw(b, kw_args, "blue", |_v, d| d)?;
    let _a = apply_kw(c.a, kw_args, "alpha", |_v, d| d.clamp(0.0, 1.0))?;

    Ok(Value::Color(c.clone_with_rgb(r, g, b)))
}

fn scale_modern_rgb_space(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (r, g, b) = get_rgb_channels(c);
    let scale_val = |val: f64, max: f64, key: &str| -> Result<f64> {
        if let Some(Value::Number(n, _)) = kw_args.get(key) {
            let pct = *n / 100.0;
            if pct >= 0.0 {
                Ok(val + (max - val) * pct)
            } else {
                Ok(val + val * pct)
            }
        } else {
            Ok(val)
        }
    };
    let r = scale_val(r, 1.0, "red")?;
    let g = scale_val(g, 1.0, "green")?;
    let b = scale_val(b, 1.0, "blue")?;
    let _a = scale_val(c.a, 1.0, "alpha")?.clamp(0.0, 1.0);

    Ok(Value::Color(c.clone_with_rgb(r, g, b)))
}

// ── Legacy (RGB/HSL/HWB) ──

fn adjust_legacy(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let r = apply_kw(c.legacy_rgb[0], kw_args, "red", |v, d| v + d)?;
    let g = apply_kw(c.legacy_rgb[1], kw_args, "green", |v, d| v + d)?;
    let b = apply_kw(c.legacy_rgb[2], kw_args, "blue", |v, d| v + d)?;
    let a = apply_kw(c.a, kw_args, "alpha", |v, d| v + d)?;
    let (h_init, s_init, l_init) =
        Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
    let h = apply_kw(h_init, kw_args, "hue", |v, d| (v + d).rem_euclid(360.0))?;
    let s = apply_pct_kw(s_init, kw_args, "saturation", |v, d| (v + d).clamp(0.0, 1.0))?;
    let l = apply_pct_kw(l_init, kw_args, "lightness", |v, d| (v + d).clamp(0.0, 1.0))?;
    let (hw_init, hb_init) = {
        let r = c.legacy_rgb[0] / 255.0;
        let g = c.legacy_rgb[1] / 255.0;
        let b = c.legacy_rgb[2] / 255.0;
        (r.min(g).min(b), 1.0 - r.max(g).max(b))
    };
    let hw = apply_pct_kw(hw_init, kw_args, "whiteness", |v, d| (v + d).clamp(0.0, 1.0))?;
    let hb = apply_pct_kw(hb_init, kw_args, "blackness", |v, d| (v + d).clamp(0.0, 1.0))?;

    let has_hwb = kw_args.contains_key("hue")
        || kw_args.contains_key("whiteness")
        || kw_args.contains_key("blackness");
    let has_hsl = (kw_args.contains_key("hue")
        || kw_args.contains_key("saturation")
        || kw_args.contains_key("lightness"))
        && !has_hwb;
    let h_changed = kw_args.contains_key("hue")
        || kw_args.contains_key("whiteness")
        || kw_args.contains_key("blackness")
        || kw_args.contains_key("saturation")
        || kw_args.contains_key("lightness");

    let (r, g, b) = match (has_hwb, has_hsl) {
        (true, _) => {
            let new_c = Evaluator::hwb_to_rgb(h, hw, hb, 1.0);
            (new_c.legacy_rgb[0], new_c.legacy_rgb[1], new_c.legacy_rgb[2])
        }
        (false, true) => {
            let new_c = Evaluator::hsl_to_rgb(h, s, l);
            (new_c.legacy_rgb[0], new_c.legacy_rgb[1], new_c.legacy_rgb[2])
        }
        (false, false) => (r, g, b),
    };
    let (output, space) = if h_changed {
        (ColorOutput::RgbPercent, ColorSpace::Hsl)
    } else {
        (ColorOutput::Auto, ColorSpace::Rgb)
    };
    Ok(Value::Color(Color::with_rgb(
        r.clamp(0.0, 255.0),
        g.clamp(0.0, 255.0),
        b.clamp(0.0, 255.0),
        a.clamp(0.0, 1.0),
        space,
        output,
    )))
}

fn change_legacy(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let r = apply_kw(c.legacy_rgb[0], kw_args, "red", |_v, d| d)?;
    let g = apply_kw(c.legacy_rgb[1], kw_args, "green", |_v, d| d)?;
    let b = apply_kw(c.legacy_rgb[2], kw_args, "blue", |_v, d| d)?;
    let a = apply_kw(c.a, kw_args, "alpha", |_v, d| d)?;
    let (h_init, s_init, l_init) =
        Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
    let h = apply_kw(h_init, kw_args, "hue", |_v, d| d.rem_euclid(360.0))?;
    let s = apply_pct_kw(s_init, kw_args, "saturation", |_v, d| d.clamp(0.0, 1.0))?;
    let l = apply_pct_kw(l_init, kw_args, "lightness", |_v, d| d.clamp(0.0, 1.0))?;
    let (hw_init, hb_init) = {
        let r = c.legacy_rgb[0] / 255.0;
        let g = c.legacy_rgb[1] / 255.0;
        let b = c.legacy_rgb[2] / 255.0;
        (r.min(g).min(b), 1.0 - r.max(g).max(b))
    };
    let hw = apply_pct_kw(hw_init, kw_args, "whiteness", |_v, d| d.clamp(0.0, 1.0))?;
    let hb = apply_pct_kw(hb_init, kw_args, "blackness", |_v, d| d.clamp(0.0, 1.0))?;

    let has_hwb = kw_args.contains_key("hue")
        || kw_args.contains_key("whiteness")
        || kw_args.contains_key("blackness");
    let has_hsl = (kw_args.contains_key("hue")
        || kw_args.contains_key("saturation")
        || kw_args.contains_key("lightness"))
        && !has_hwb;
    let h_changed = kw_args.contains_key("hue")
        || kw_args.contains_key("whiteness")
        || kw_args.contains_key("blackness")
        || kw_args.contains_key("saturation")
        || kw_args.contains_key("lightness");

    let (r, g, b) = match (has_hwb, has_hsl) {
        (true, _) => {
            let new_c = Evaluator::hwb_to_rgb(h, hw, hb, 1.0);
            (new_c.legacy_rgb[0], new_c.legacy_rgb[1], new_c.legacy_rgb[2])
        }
        (false, true) => {
            let new_c = Evaluator::hsl_to_rgb(h, s, l);
            (new_c.legacy_rgb[0], new_c.legacy_rgb[1], new_c.legacy_rgb[2])
        }
        (false, false) => (r, g, b),
    };
    let (output, space) = if h_changed {
        (ColorOutput::RgbPercent, ColorSpace::Hsl)
    } else {
        (ColorOutput::Auto, ColorSpace::Rgb)
    };
    Ok(Value::Color(Color::with_rgb(
        r.clamp(0.0, 255.0),
        g.clamp(0.0, 255.0),
        b.clamp(0.0, 255.0),
        a.clamp(0.0, 1.0),
        space,
        output,
    )))
}

fn scale_legacy(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let scale_val = |val: f64, max: f64, kw: &str| -> Result<f64> {
        match kw_args.get(kw) {
            Some(Value::Number(n, _)) => {
                let pct = *n / 100.0;
                Ok(if pct >= 0.0 { val + (max - val) * pct } else { val + val * pct })
            }
            _ => Ok(val),
        }
    };
    let r = scale_val(c.legacy_rgb[0], 255.0, "red")?;
    let g = scale_val(c.legacy_rgb[1], 255.0, "green")?;
    let b = scale_val(c.legacy_rgb[2], 255.0, "blue")?;
    let a = scale_val(c.a, 1.0, "alpha")?;
    let (h, s_init, l_init) =
        Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);

    let has_hsl = kw_args.contains_key("saturation") || kw_args.contains_key("lightness");
    let s = if kw_args.contains_key("saturation") {
        scale_val(s_init, 1.0, "saturation")?.clamp(0.0, 1.0)
    } else { s_init };
    let l = if kw_args.contains_key("lightness") {
        scale_val(l_init, 1.0, "lightness")?.clamp(0.0, 1.0)
    } else { l_init };

    let (r, g, b) = if has_hsl {
        let new_c = Evaluator::hsl_to_rgb(h, s, l);
        (new_c.legacy_rgb[0], new_c.legacy_rgb[1], new_c.legacy_rgb[2])
    } else {
        (r, g, b)
    };
    let (output, space) = if has_hsl {
        (ColorOutput::RgbPercent, ColorSpace::Hsl)
    } else {
        (ColorOutput::Auto, ColorSpace::Rgb)
    };
    Ok(Value::Color(Color::with_rgb(
        r.clamp(0.0, 255.0),
        g.clamp(0.0, 255.0),
        b.clamp(0.0, 255.0),
        a.clamp(0.0, 1.0),
        space,
        output,
    )))
}
