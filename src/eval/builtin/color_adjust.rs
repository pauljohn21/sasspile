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

// ── 辅助函数 ─────────────────────────────────────────────────────────────

/// 从 `kw_args` 中提取数值参数。
fn get_num(kw_args: &HashMap<String, Value>, key: &str) -> Result<Option<f64>> {
    match kw_args.get(key) {
        Some(Value::Number(n, _)) => Ok(Some(*n)),
        Some(_) => Err(SassError::Eval(format!("{key} requires a number"))),
        None => Ok(None),
    }
}

/// 提取百分比参数，返回 0-1 范围的值。
fn get_pct_or_num(kw_args: &HashMap<String, Value>, key: &str) -> Result<Option<f64>> {
    match kw_args.get(key) {
        Some(Value::Number(n, Some(unit))) if unit == "%" => Ok(Some(*n / 100.0)),
        Some(Value::Number(n, None)) => Ok(Some(*n)),
        Some(Value::Number(n, Some(_))) => Ok(Some(*n)),
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

/// 缩放通道值——按百分比向最大值方向缩放（正）或向 0 方向缩放（负）。
fn scale_channel(val: f64, max: f64, kw_args: &HashMap<String, Value>, key: &str) -> Result<f64> {
    Ok(match get_num(kw_args, key)? {
        Some(n) => {
            let pct = n / 100.0;
            match pct >= 0.0 {
                true => val + (max - val) * pct,
                false => val + val * pct,
            }
        }
        None => val,
    })
}

/// 从 args/kw_args 提取颜色参数。
fn extract_color<'a>(args: &'a [Value], kw_args: &'a HashMap<String, Value>) -> Result<&'a Color> {
    match args.first().or_else(|| kw_args.get("color")) {
        Some(Value::Color(c)) => Ok(c),
        Some(v) => Err(SassError::Eval(format!("$color: {v} is not a color."))),
        None => Err(SassError::Eval("Missing argument $color.".into())),
    }
}

// ── 入口函数 ─────────────────────────────────────────────────────────────

/// `color.adjust($color, $kwargs)` — 调整颜色通道（增量）。
pub fn adjust_color(args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Value> {
    let c = extract_color(args, kw_args)?;

    let modern_channels = ["lightness", "chroma", "a", "b", "x", "y", "z"];
    let has_modern = modern_channels.iter().any(|ch| kw_args.contains_key(*ch));

    match c.space {
        ColorSpace::Oklch => adjust_oklch(c, kw_args),
        ColorSpace::Oklab => adjust_oklab(c, kw_args),
        ColorSpace::Lch => adjust_lch(c, kw_args),
        ColorSpace::Lab => adjust_lab(c, kw_args),
        ColorSpace::DisplayP3
        | ColorSpace::Srgb
        | ColorSpace::SrgbLinear
        | ColorSpace::DisplayP3Linear
        | ColorSpace::A98Rgb
        | ColorSpace::ProphotoRgb
        | ColorSpace::Rec2020
        | ColorSpace::XyzD65
        | ColorSpace::XyzD50 => adjust_modern_rgb_space(c, kw_args),
        _ if has_modern => adjust_legacy(c, kw_args),
        _ => adjust_legacy(c, kw_args),
    }
}

/// `color.change($color, $kwargs)` — 设置颜色通道（绝对值）。
pub fn change_color(args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Value> {
    let c = extract_color(args, kw_args)?;

    match c.space {
        ColorSpace::Oklch => change_oklch(c, kw_args),
        ColorSpace::Oklab => change_oklab(c, kw_args),
        ColorSpace::Lch => change_lch(c, kw_args),
        ColorSpace::Lab => change_lab(c, kw_args),
        ColorSpace::DisplayP3
        | ColorSpace::Srgb
        | ColorSpace::SrgbLinear
        | ColorSpace::DisplayP3Linear
        | ColorSpace::A98Rgb
        | ColorSpace::ProphotoRgb
        | ColorSpace::Rec2020
        | ColorSpace::XyzD65
        | ColorSpace::XyzD50 => change_modern_rgb_space(c, kw_args),
        _ => change_legacy(c, kw_args),
    }
}

/// `color.scale($color, $kwargs)` — 按比例缩放颜色通道。
pub fn scale_color(args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Value> {
    let c = extract_color(args, kw_args)?;

    match c.space {
        ColorSpace::Oklch => scale_oklch(c, kw_args),
        ColorSpace::Oklab => scale_oklab(c, kw_args),
        ColorSpace::Lch => scale_lch(c, kw_args),
        ColorSpace::Lab => scale_lab(c, kw_args),
        ColorSpace::DisplayP3
        | ColorSpace::Srgb
        | ColorSpace::SrgbLinear
        | ColorSpace::DisplayP3Linear
        | ColorSpace::A98Rgb
        | ColorSpace::ProphotoRgb
        | ColorSpace::Rec2020
        | ColorSpace::XyzD65
        | ColorSpace::XyzD50 => scale_modern_rgb_space(c, kw_args),
        _ => scale_legacy(c, kw_args),
    }
}

// ── Oklch ────────────────────────────────────────────────────────────────

fn adjust_oklch(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let l = apply_pct_kw(c.channels[0], kw_args, "lightness", |v, d| (v + d).clamp(0.0, 1.0))?;
    let ch = apply_kw(c.channels[1], kw_args, "chroma", |v, d| (v + d).max(0.0))?;
    let h = apply_kw(c.channels[2], kw_args, "hue", |v, d| (v + d).rem_euclid(360.0))?;
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
    let l = apply_pct_kw(c.channels[0], kw_args, "lightness", |_v, d| d.clamp(0.0, 1.0))?;
    let ch = apply_kw(c.channels[1], kw_args, "chroma", |_v, d| d.max(0.0))?;
    let h = apply_kw(c.channels[2], kw_args, "hue", |_v, d| d.rem_euclid(360.0))?;
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
    let l = scale_channel(c.channels[0], 1.0, kw_args, "lightness")?.clamp(0.0, 1.0);
    let ch = scale_channel(c.channels[1], f64::MAX, kw_args, "chroma")?.max(0.0);
    let a = scale_channel(c.a, 1.0, kw_args, "alpha")?.clamp(0.0, 1.0);

    Ok(Value::Color(Color::with_space(
        ColorSpace::Oklch,
        [l, ch, c.channels[2]],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

// ── Oklab ────────────────────────────────────────────────────────────────

fn adjust_oklab(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let l = apply_pct_kw(c.channels[0], kw_args, "lightness", |v, d| (v + d).clamp(0.0, 1.0))?;
    let a_v = apply_kw(c.channels[1], kw_args, "a", |v, d| v + d)?;
    let b_v = apply_kw(c.channels[2], kw_args, "b", |v, d| v + d)?;
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
    let l = apply_pct_kw(c.channels[0], kw_args, "lightness", |_v, d| d.clamp(0.0, 1.0))?;
    let a_v = apply_kw(c.channels[1], kw_args, "a", |_v, d| d)?;
    let b_v = apply_kw(c.channels[2], kw_args, "b", |_v, d| d)?;
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
    let l = scale_channel(c.channels[0], 1.0, kw_args, "lightness")?.clamp(0.0, 1.0);
    let a_max = if c.channels[1] >= 0.0 { 0.5 } else { -0.5 };
    let b_max = if c.channels[2] >= 0.0 { 0.5 } else { -0.5 };
    let a_v = scale_channel(c.channels[1], a_max, kw_args, "a")?;
    let b_v = scale_channel(c.channels[2], b_max, kw_args, "b")?;
    let a = scale_channel(c.a, 1.0, kw_args, "alpha")?.clamp(0.0, 1.0);

    Ok(Value::Color(Color::with_space(
        ColorSpace::Oklab,
        [l, a_v, b_v],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

// ── Lch ──────────────────────────────────────────────────────────────────

fn adjust_lch(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let l = apply_pct_kw(c.channels[0], kw_args, "lightness", |v, d| {
        (v + d * 100.0).clamp(0.0, 100.0)
    })?;
    let ch = apply_kw(c.channels[1], kw_args, "chroma", |v, d| (v + d).max(0.0))?;
    let h = apply_kw(c.channels[2], kw_args, "hue", |v, d| (v + d).rem_euclid(360.0))?;
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
    let l = apply_pct_kw(c.channels[0], kw_args, "lightness", |_v, d| {
        (d * 100.0).clamp(0.0, 100.0)
    })?;
    let ch = apply_kw(c.channels[1], kw_args, "chroma", |_v, d| d.max(0.0))?;
    let h = apply_kw(c.channels[2], kw_args, "hue", |_v, d| d.rem_euclid(360.0))?;
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
    let l = scale_channel(c.channels[0], 100.0, kw_args, "lightness")?.clamp(0.0, 100.0);
    let ch = scale_channel(c.channels[1], f64::MAX, kw_args, "chroma")?.max(0.0);
    let a = scale_channel(c.a, 1.0, kw_args, "alpha")?.clamp(0.0, 1.0);

    Ok(Value::Color(Color::with_space(
        ColorSpace::Lch,
        [l, ch, c.channels[2]],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

// ── Lab ──────────────────────────────────────────────────────────────────

fn adjust_lab(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let l = apply_pct_kw(c.channels[0], kw_args, "lightness", |v, d| {
        (v + d * 100.0).clamp(0.0, 100.0)
    })?;
    let a_v = apply_kw(c.channels[1], kw_args, "a", |v, d| v + d)?;
    let b_v = apply_kw(c.channels[2], kw_args, "b", |v, d| v + d)?;
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
    let l = apply_pct_kw(c.channels[0], kw_args, "lightness", |_v, d| {
        (d * 100.0).clamp(0.0, 100.0)
    })?;
    let a_v = apply_kw(c.channels[1], kw_args, "a", |_v, d| d)?;
    let b_v = apply_kw(c.channels[2], kw_args, "b", |_v, d| d)?;
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
    let l = scale_channel(c.channels[0], 100.0, kw_args, "lightness")?.clamp(0.0, 100.0);
    let a_max = if c.channels[1] >= 0.0 { 125.0 } else { -125.0 };
    let b_max = if c.channels[2] >= 0.0 { 125.0 } else { -125.0 };
    let a_v = scale_channel(c.channels[1], a_max, kw_args, "a")?;
    let b_v = scale_channel(c.channels[2], b_max, kw_args, "b")?;
    let a = scale_channel(c.a, 1.0, kw_args, "alpha")?.clamp(0.0, 1.0);

    Ok(Value::Color(Color::with_space(
        ColorSpace::Lab,
        [l, a_v, b_v],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

// ── Modern RGB 空间 (DisplayP3, sRGB, A98, ProPhoto, Rec2020) ─────────────

fn adjust_modern_rgb_space(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let r = apply_kw(c.channels[0], kw_args, "red", |v, d| v + d)?;
    let g = apply_kw(c.channels[1], kw_args, "green", |v, d| v + d)?;
    let b = apply_kw(c.channels[2], kw_args, "blue", |v, d| v + d)?;
    let a = apply_kw(c.a, kw_args, "alpha", |v, d| (v + d).clamp(0.0, 1.0))?;

    Ok(Value::Color(Color::with_space(
        c.space,
        [r, g, b],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

fn change_modern_rgb_space(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let r = apply_kw(c.channels[0], kw_args, "red", |_v, d| d)?;
    let g = apply_kw(c.channels[1], kw_args, "green", |_v, d| d)?;
    let b = apply_kw(c.channels[2], kw_args, "blue", |_v, d| d)?;
    let a = apply_kw(c.a, kw_args, "alpha", |_v, d| d.clamp(0.0, 1.0))?;

    Ok(Value::Color(Color::with_space(
        c.space,
        [r, g, b],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

fn scale_modern_rgb_space(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let r = scale_channel(c.channels[0], 1.0, kw_args, "red")?;
    let g = scale_channel(c.channels[1], 1.0, kw_args, "green")?;
    let b = scale_channel(c.channels[2], 1.0, kw_args, "blue")?;
    let a = scale_channel(c.a, 1.0, kw_args, "alpha")?.clamp(0.0, 1.0);

    Ok(Value::Color(Color::with_space(
        c.space,
        [r, g, b],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

// ── Legacy (RGB/HSL/HWB) ─────────────────────────────────────────────────

fn adjust_legacy(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    // RGB 通道调整
    let r = apply_kw(c.legacy_rgb[0], kw_args, "red", |v, d| v + d)?;
    let g = apply_kw(c.legacy_rgb[1], kw_args, "green", |v, d| v + d)?;
    let b = apply_kw(c.legacy_rgb[2], kw_args, "blue", |v, d| v + d)?;
    let alpha = apply_kw(c.a, kw_args, "alpha", |v, d| v + d)?;

    // HSL 通道调整
    let (h_init, s_init, l_init) =
        Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
    let h = apply_kw(h_init, kw_args, "hue", |v, d| (v + d).rem_euclid(360.0))?;
    let s = apply_pct_kw(s_init, kw_args, "saturation", |v, d| (v + d).clamp(0.0, 1.0))?;
    let l = apply_pct_kw(l_init, kw_args, "lightness", |v, d| (v + d).clamp(0.0, 1.0))?;

    // HWB 通道调整
    let rgb_norm = [c.legacy_rgb[0] / 255.0, c.legacy_rgb[1] / 255.0, c.legacy_rgb[2] / 255.0];
    let hw_init = rgb_norm.iter().copied().fold(f64::INFINITY, f64::min);
    let hb_init = 1.0 - rgb_norm.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let hw = apply_pct_kw(hw_init, kw_args, "whiteness", |v, d| (v + d).clamp(0.0, 1.0))?;
    let hb = apply_pct_kw(hb_init, kw_args, "blackness", |v, d| (v + d).clamp(0.0, 1.0))?;

    // 确定输出路径
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

    let rgb_result = match (has_hwb, has_hsl) {
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

    let (output, space) = match h_changed {
        true => (ColorOutput::RgbPercent, ColorSpace::Hsl),
        false => (ColorOutput::Auto, ColorSpace::Rgb),
    };

    Ok(Value::Color(Color::with_rgb(
        rgb_result.0.clamp(0.0, 255.0),
        rgb_result.1.clamp(0.0, 255.0),
        rgb_result.2.clamp(0.0, 255.0),
        alpha.clamp(0.0, 1.0),
        space,
        output,
    )))
}

fn change_legacy(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    // RGB 通道设置
    let r = apply_kw(c.legacy_rgb[0], kw_args, "red", |_v, d| d)?;
    let g = apply_kw(c.legacy_rgb[1], kw_args, "green", |_v, d| d)?;
    let b = apply_kw(c.legacy_rgb[2], kw_args, "blue", |_v, d| d)?;
    let alpha = apply_kw(c.a, kw_args, "alpha", |_v, d| d)?;

    // HSL 通道设置
    let (h_init, s_init, l_init) =
        Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
    let h = apply_kw(h_init, kw_args, "hue", |_v, d| d.rem_euclid(360.0))?;
    let s = apply_pct_kw(s_init, kw_args, "saturation", |_v, d| d.clamp(0.0, 1.0))?;
    let l = apply_pct_kw(l_init, kw_args, "lightness", |_v, d| d.clamp(0.0, 1.0))?;

    // HWB 通道设置
    let rgb_norm = [c.legacy_rgb[0] / 255.0, c.legacy_rgb[1] / 255.0, c.legacy_rgb[2] / 255.0];
    let hw_init = rgb_norm.iter().copied().fold(f64::INFINITY, f64::min);
    let hb_init = 1.0 - rgb_norm.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let hw = apply_pct_kw(hw_init, kw_args, "whiteness", |_v, d| d.clamp(0.0, 1.0))?;
    let hb = apply_pct_kw(hb_init, kw_args, "blackness", |_v, d| d.clamp(0.0, 1.0))?;

    // 确定输出路径
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

    let rgb_result = match (has_hwb, has_hsl) {
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

    let (output, space) = match h_changed {
        true => (ColorOutput::RgbPercent, ColorSpace::Hsl),
        false => (ColorOutput::Auto, ColorSpace::Rgb),
    };

    Ok(Value::Color(Color::with_rgb(
        rgb_result.0.clamp(0.0, 255.0),
        rgb_result.1.clamp(0.0, 255.0),
        rgb_result.2.clamp(0.0, 255.0),
        alpha.clamp(0.0, 1.0),
        space,
        output,
    )))
}

fn scale_legacy(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    // RGB 通道缩放
    let r = scale_channel(c.legacy_rgb[0], 255.0, kw_args, "red")?;
    let g = scale_channel(c.legacy_rgb[1], 255.0, kw_args, "green")?;
    let b = scale_channel(c.legacy_rgb[2], 255.0, kw_args, "blue")?;
    let alpha = scale_channel(c.a, 1.0, kw_args, "alpha")?;

    // HSL 通道缩放
    let (h, s_init, l_init) =
        Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
    let has_hsl = kw_args.contains_key("saturation") || kw_args.contains_key("lightness");
    let s = scale_channel(s_init, 1.0, kw_args, "saturation")?;
    let l = scale_channel(l_init, 1.0, kw_args, "lightness")?;

    let rgb_result = match has_hsl {
        true => {
            let new_c = Evaluator::hsl_to_rgb(h, s.clamp(0.0, 1.0), l.clamp(0.0, 1.0));
            (new_c.legacy_rgb[0], new_c.legacy_rgb[1], new_c.legacy_rgb[2])
        }
        false => (r, g, b),
    };

    let (output, space) = match has_hsl {
        true => (ColorOutput::RgbPercent, ColorSpace::Hsl),
        false => (ColorOutput::Auto, ColorSpace::Rgb),
    };

    Ok(Value::Color(Color::with_rgb(
        rgb_result.0.clamp(0.0, 255.0),
        rgb_result.1.clamp(0.0, 255.0),
        rgb_result.2.clamp(0.0, 255.0),
        alpha.clamp(0.0, 1.0),
        space,
        output,
    )))
}
