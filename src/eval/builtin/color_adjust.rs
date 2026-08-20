//! `color.adjust` / `color.change` / `color.scale` 实现。
//!
//! 支持所有 CSS Color 4 颜色空间：
//! - Legacy: RGB, HSL, HWB
//! - Modern: Lab, Lch, Oklab, Oklch, DisplayP3, sRGB, sRGB-Linear, etc.
//!
//! 现代空间直接在 ColorFormat 中修改通道值，保留原始格式输出。

use crate::error::{Result, SassError};
use crate::parse::ast::{Color, ColorFormat, Value};
use im::HashMap;

use super::super::Evaluator;

/// 从 kw_args 中提取数值参数。
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

/// `color.adjust($color, $kwargs)` — 调整颜色通道（增量）。
pub fn adjust_color(args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Value> {
    let c = match args.first().or_else(|| kw_args.get("$color")) {
        Some(Value::Color(c)) => c.clone(),
        Some(v) => return Err(SassError::Eval(format!("$color: {} is not a color.", v))),
        None => return Err(SassError::Eval("Missing argument $color.".into())),
    };

    // 检查是否有现代颜色空间的通道参数
    let modern_channels = ["lightness", "chroma", "a", "b", "x", "y", "z"];
    let has_modern = modern_channels.iter().any(|ch| kw_args.contains_key(*ch));

    // 根据颜色格式选择处理路径
    match c.format {
        ColorFormat::Oklch(_, _, _) => adjust_oklch(&c, kw_args),
        ColorFormat::Oklab(_, _, _) => adjust_oklab(&c, kw_args),
        ColorFormat::Lch(_, _, _) => adjust_lch(&c, kw_args),
        ColorFormat::Lab(_, _, _) => adjust_lab(&c, kw_args),
        ColorFormat::DisplayP3(_, _, _)
        | ColorFormat::Srgb(_, _, _)
        | ColorFormat::SrgbLinear(_, _, _)
        | ColorFormat::DisplayP3Linear(_, _, _)
        | ColorFormat::A98Rgb(_, _, _)
        | ColorFormat::ProphotoRgb(_, _, _)
        | ColorFormat::Rec2020(_, _, _)
        | ColorFormat::XyzD65(_, _, _)
        | ColorFormat::XyzD50(_, _, _) => adjust_modern_rgb_space(&c, kw_args),
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
        Some(v) => return Err(SassError::Eval(format!("$color: {} is not a color.", v))),
        None => return Err(SassError::Eval("Missing argument $color.".into())),
    };

    match c.format {
        ColorFormat::Oklch(_, _, _) => change_oklch(&c, kw_args),
        ColorFormat::Oklab(_, _, _) => change_oklab(&c, kw_args),
        ColorFormat::Lch(_, _, _) => change_lch(&c, kw_args),
        ColorFormat::Lab(_, _, _) => change_lab(&c, kw_args),
        ColorFormat::DisplayP3(_, _, _)
        | ColorFormat::Srgb(_, _, _)
        | ColorFormat::SrgbLinear(_, _, _)
        | ColorFormat::DisplayP3Linear(_, _, _)
        | ColorFormat::A98Rgb(_, _, _)
        | ColorFormat::ProphotoRgb(_, _, _)
        | ColorFormat::Rec2020(_, _, _)
        | ColorFormat::XyzD65(_, _, _)
        | ColorFormat::XyzD50(_, _, _) => change_modern_rgb_space(&c, kw_args),
        _ => change_legacy(&c, kw_args),
    }
}

/// `color.scale($color, $kwargs)` — 按比例缩放颜色通道。
pub fn scale_color(args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Value> {
    let c = match args.first().or_else(|| kw_args.get("$color")) {
        Some(Value::Color(c)) => c.clone(),
        Some(v) => return Err(SassError::Eval(format!("$color: {} is not a color.", v))),
        None => return Err(SassError::Eval("Missing argument $color.".into())),
    };

    match c.format {
        ColorFormat::Oklch(_, _, _) => scale_oklch(&c, kw_args),
        ColorFormat::Oklab(_, _, _) => scale_oklab(&c, kw_args),
        ColorFormat::Lch(_, _, _) => scale_lch(&c, kw_args),
        ColorFormat::Lab(_, _, _) => scale_lab(&c, kw_args),
        ColorFormat::DisplayP3(_, _, _)
        | ColorFormat::Srgb(_, _, _)
        | ColorFormat::SrgbLinear(_, _, _)
        | ColorFormat::DisplayP3Linear(_, _, _)
        | ColorFormat::A98Rgb(_, _, _)
        | ColorFormat::ProphotoRgb(_, _, _)
        | ColorFormat::Rec2020(_, _, _)
        | ColorFormat::XyzD65(_, _, _)
        | ColorFormat::XyzD50(_, _, _) => scale_modern_rgb_space(&c, kw_args),
        _ => scale_legacy(&c, kw_args),
    }
}

// ── Oklch ──

fn adjust_oklch(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (l, ch, h) = match c.format {
        ColorFormat::Oklch(l, ch, h) => (l, ch, h),
        _ => unreachable!(),
    };
    let mut l = l;
    let mut ch = ch;
    let mut h = h;
    let mut a = c.a;

    if let Some(v) = get_pct_or_num(kw_args, "lightness")? { l = (l + v).clamp(0.0, 1.0); }
    if let Some(v) = get_num(kw_args, "chroma")? { ch = (ch + v).max(0.0); }
    if let Some(v) = get_num(kw_args, "hue")? { h = (h + v).rem_euclid(360.0); }
    if let Some(v) = get_num(kw_args, "alpha")? { a = (a + v).clamp(0.0, 1.0); }

    Ok(Value::Color(Color::rgba_fmt(c.r, c.g, c.b, a, ColorFormat::Oklch(l, ch, h))))
}

fn change_oklch(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (l, ch, h) = match c.format {
        ColorFormat::Oklch(l, ch, h) => (l, ch, h),
        _ => unreachable!(),
    };
    let mut l = l;
    let mut ch = ch;
    let mut h = h;
    let mut a = c.a;

    if let Some(v) = get_pct_or_num(kw_args, "lightness")? { l = v.clamp(0.0, 1.0); }
    if let Some(v) = get_num(kw_args, "chroma")? { ch = v.max(0.0); }
    if let Some(v) = get_num(kw_args, "hue")? { h = v.rem_euclid(360.0); }
    if let Some(v) = get_num(kw_args, "alpha")? { a = v.clamp(0.0, 1.0); }

    Ok(Value::Color(Color::rgba_fmt(c.r, c.g, c.b, a, ColorFormat::Oklch(l, ch, h))))
}

fn scale_oklch(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (l, ch, h) = match c.format {
        ColorFormat::Oklch(l, ch, h) => (l, ch, h),
        _ => unreachable!(),
    };
    let mut l = l;
    let mut ch = ch;
    let mut a = c.a;

    let scale_val = |val: f64, max: f64, key: &str| -> Result<f64> {
        if let Some(Value::Number(n, _)) = kw_args.get(key) {
            let pct = *n / 100.0;
            if pct >= 0.0 { Ok(val + (max - val) * pct) }
            else { Ok(val + val * pct) }
        } else { Ok(val) }
    };
    l = scale_val(l, 1.0, "lightness")?.clamp(0.0, 1.0);
    ch = scale_val(ch, f64::MAX, "chroma")?.max(0.0);
    a = scale_val(a, 1.0, "alpha")?.clamp(0.0, 1.0);

    Ok(Value::Color(Color::rgba_fmt(c.r, c.g, c.b, a, ColorFormat::Oklch(l, ch, h))))
}

// ── Oklab ──

fn adjust_oklab(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (l, a_v, b_v) = match c.format {
        ColorFormat::Oklab(l, a, b) => (l, a, b),
        _ => unreachable!(),
    };
    let mut l = l;
    let mut a_v = a_v;
    let mut b_v = b_v;
    let mut a = c.a;

    if let Some(v) = get_pct_or_num(kw_args, "lightness")? { l = (l + v).clamp(0.0, 1.0); }
    if let Some(v) = get_num(kw_args, "a")? { a_v += v; }
    if let Some(v) = get_num(kw_args, "b")? { b_v += v; }
    if let Some(v) = get_num(kw_args, "alpha")? { a = (a + v).clamp(0.0, 1.0); }

    Ok(Value::Color(Color::rgba_fmt(c.r, c.g, c.b, a, ColorFormat::Oklab(l, a_v, b_v))))
}

fn change_oklab(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (l, a_v, b_v) = match c.format {
        ColorFormat::Oklab(l, a, b) => (l, a, b),
        _ => unreachable!(),
    };
    let mut l = l;
    let mut a_v = a_v;
    let mut b_v = b_v;
    let mut a = c.a;

    if let Some(v) = get_pct_or_num(kw_args, "lightness")? { l = v.clamp(0.0, 1.0); }
    if let Some(v) = get_num(kw_args, "a")? { a_v = v; }
    if let Some(v) = get_num(kw_args, "b")? { b_v = v; }
    if let Some(v) = get_num(kw_args, "alpha")? { a = v.clamp(0.0, 1.0); }

    Ok(Value::Color(Color::rgba_fmt(c.r, c.g, c.b, a, ColorFormat::Oklab(l, a_v, b_v))))
}

fn scale_oklab(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (l, a_v, b_v) = match c.format {
        ColorFormat::Oklab(l, a, b) => (l, a, b),
        _ => unreachable!(),
    };
    let mut l = l;
    let mut a_v = a_v;
    let mut b_v = b_v;
    let mut a = c.a;

    let scale_val = |val: f64, max: f64, key: &str| -> Result<f64> {
        if let Some(Value::Number(n, _)) = kw_args.get(key) {
            let pct = *n / 100.0;
            if pct >= 0.0 { Ok(val + (max - val) * pct) }
            else { Ok(val + val * pct) }
        } else { Ok(val) }
    };
    l = scale_val(l, 1.0, "lightness")?.clamp(0.0, 1.0);
    a_v = scale_val(a_v, if a_v >= 0.0 { 0.5 } else { -0.5 }, "a")?;
    b_v = scale_val(b_v, if b_v >= 0.0 { 0.5 } else { -0.5 }, "b")?;
    a = scale_val(a, 1.0, "alpha")?.clamp(0.0, 1.0);

    Ok(Value::Color(Color::rgba_fmt(c.r, c.g, c.b, a, ColorFormat::Oklab(l, a_v, b_v))))
}

// ── Lch ──

fn adjust_lch(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (l, ch, h) = match c.format {
        ColorFormat::Lch(l, ch, h) => (l, ch, h),
        _ => unreachable!(),
    };
    let mut l = l;
    let mut ch = ch;
    let mut h = h;
    let mut a = c.a;

    if let Some(v) = get_pct_or_num(kw_args, "lightness")? { l = (l + v * 100.0).clamp(0.0, 100.0); }
    if let Some(v) = get_num(kw_args, "chroma")? { ch = (ch + v).max(0.0); }
    if let Some(v) = get_num(kw_args, "hue")? { h = (h + v).rem_euclid(360.0); }
    if let Some(v) = get_num(kw_args, "alpha")? { a = (a + v).clamp(0.0, 1.0); }

    Ok(Value::Color(Color::rgba_fmt(c.r, c.g, c.b, a, ColorFormat::Lch(l, ch, h))))
}

fn change_lch(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (l, ch, h) = match c.format {
        ColorFormat::Lch(l, ch, h) => (l, ch, h),
        _ => unreachable!(),
    };
    let mut l = l;
    let mut ch = ch;
    let mut h = h;
    let mut a = c.a;

    if let Some(v) = get_pct_or_num(kw_args, "lightness")? { l = (v * 100.0).clamp(0.0, 100.0); }
    if let Some(v) = get_num(kw_args, "chroma")? { ch = v.max(0.0); }
    if let Some(v) = get_num(kw_args, "hue")? { h = v.rem_euclid(360.0); }
    if let Some(v) = get_num(kw_args, "alpha")? { a = v.clamp(0.0, 1.0); }

    Ok(Value::Color(Color::rgba_fmt(c.r, c.g, c.b, a, ColorFormat::Lch(l, ch, h))))
}

fn scale_lch(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (l, ch, h) = match c.format {
        ColorFormat::Lch(l, ch, h) => (l, ch, h),
        _ => unreachable!(),
    };
    let mut l = l;
    let mut ch = ch;
    let mut a = c.a;

    let scale_val = |val: f64, max: f64, key: &str| -> Result<f64> {
        if let Some(Value::Number(n, _)) = kw_args.get(key) {
            let pct = *n / 100.0;
            if pct >= 0.0 { Ok(val + (max - val) * pct) }
            else { Ok(val + val * pct) }
        } else { Ok(val) }
    };
    l = scale_val(l, 100.0, "lightness")?.clamp(0.0, 100.0);
    ch = scale_val(ch, f64::MAX, "chroma")?.max(0.0);
    a = scale_val(a, 1.0, "alpha")?.clamp(0.0, 1.0);

    Ok(Value::Color(Color::rgba_fmt(c.r, c.g, c.b, a, ColorFormat::Lch(l, ch, h))))
}

// ── Lab ──

fn adjust_lab(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (l, a_v, b_v) = match c.format {
        ColorFormat::Lab(l, a, b) => (l, a, b),
        _ => unreachable!(),
    };
    let mut l = l;
    let mut a_v = a_v;
    let mut b_v = b_v;
    let mut a = c.a;

    if let Some(v) = get_pct_or_num(kw_args, "lightness")? { l = (l + v * 100.0).clamp(0.0, 100.0); }
    if let Some(v) = get_num(kw_args, "a")? { a_v += v; }
    if let Some(v) = get_num(kw_args, "b")? { b_v += v; }
    if let Some(v) = get_num(kw_args, "alpha")? { a = (a + v).clamp(0.0, 1.0); }

    Ok(Value::Color(Color::rgba_fmt(c.r, c.g, c.b, a, ColorFormat::Lab(l, a_v, b_v))))
}

fn change_lab(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (l, a_v, b_v) = match c.format {
        ColorFormat::Lab(l, a, b) => (l, a, b),
        _ => unreachable!(),
    };
    let mut l = l;
    let mut a_v = a_v;
    let mut b_v = b_v;
    let mut a = c.a;

    if let Some(v) = get_pct_or_num(kw_args, "lightness")? { l = (v * 100.0).clamp(0.0, 100.0); }
    if let Some(v) = get_num(kw_args, "a")? { a_v = v; }
    if let Some(v) = get_num(kw_args, "b")? { b_v = v; }
    if let Some(v) = get_num(kw_args, "alpha")? { a = v.clamp(0.0, 1.0); }

    Ok(Value::Color(Color::rgba_fmt(c.r, c.g, c.b, a, ColorFormat::Lab(l, a_v, b_v))))
}

fn scale_lab(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (l, a_v, b_v) = match c.format {
        ColorFormat::Lab(l, a, b) => (l, a, b),
        _ => unreachable!(),
    };
    let mut l = l;
    let mut a_v = a_v;
    let mut b_v = b_v;
    let mut a = c.a;

    let scale_val = |val: f64, max: f64, key: &str| -> Result<f64> {
        if let Some(Value::Number(n, _)) = kw_args.get(key) {
            let pct = *n / 100.0;
            if pct >= 0.0 { Ok(val + (max - val) * pct) }
            else { Ok(val + val * pct) }
        } else { Ok(val) }
    };
    l = scale_val(l, 100.0, "lightness")?.clamp(0.0, 100.0);
    a_v = scale_val(a_v, if a_v >= 0.0 { 125.0 } else { -125.0 }, "a")?;
    b_v = scale_val(b_v, if b_v >= 0.0 { 125.0 } else { -125.0 }, "b")?;
    a = scale_val(a, 1.0, "alpha")?.clamp(0.0, 1.0);

    Ok(Value::Color(Color::rgba_fmt(c.r, c.g, c.b, a, ColorFormat::Lab(l, a_v, b_v))))
}

// ── Modern RGB 空间 (DisplayP3, sRGB, A98, ProPhoto, Rec2020) ──

fn get_rgb_channels(fmt: &ColorFormat) -> (f64, f64, f64) {
    match fmt {
        ColorFormat::DisplayP3(r, g, b) | ColorFormat::Srgb(r, g, b)
        | ColorFormat::A98Rgb(r, g, b) | ColorFormat::ProphotoRgb(r, g, b)
        | ColorFormat::Rec2020(r, g, b) => (*r, *g, *b),
        ColorFormat::SrgbLinear(r, g, b) | ColorFormat::DisplayP3Linear(r, g, b) => (*r, *g, *b),
        _ => (0.0, 0.0, 0.0),
    }
}

fn adjust_modern_rgb_space(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (mut r, mut g, mut b) = get_rgb_channels(&c.format);
    let mut a = c.a;

    if let Some(v) = get_num(kw_args, "red")? { r += v; }
    if let Some(v) = get_num(kw_args, "green")? { g += v; }
    if let Some(v) = get_num(kw_args, "blue")? { b += v; }
    if let Some(v) = get_num(kw_args, "alpha")? { a = (a + v).clamp(0.0, 1.0); }

    Ok(Value::Color(Color::rgba_fmt(c.r, c.g, c.b, a, c.format.clone_with(r, g, b))))
}

fn change_modern_rgb_space(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (mut r, mut g, mut b) = get_rgb_channels(&c.format);
    let mut a = c.a;

    if let Some(v) = get_num(kw_args, "red")? { r = v; }
    if let Some(v) = get_num(kw_args, "green")? { g = v; }
    if let Some(v) = get_num(kw_args, "blue")? { b = v; }
    if let Some(v) = get_num(kw_args, "alpha")? { a = v.clamp(0.0, 1.0); }

    Ok(Value::Color(Color::rgba_fmt(c.r, c.g, c.b, a, c.format.clone_with(r, g, b))))
}

fn scale_modern_rgb_space(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let (mut r, mut g, mut b) = get_rgb_channels(&c.format);
    let mut a = c.a;

    let scale_val = |val: f64, max: f64, key: &str| -> Result<f64> {
        if let Some(Value::Number(n, _)) = kw_args.get(key) {
            let pct = *n / 100.0;
            if pct >= 0.0 { Ok(val + (max - val) * pct) }
            else { Ok(val + val * pct) }
        } else { Ok(val) }
    };
    r = scale_val(r, 1.0, "red")?;
    g = scale_val(g, 1.0, "green")?;
    b = scale_val(b, 1.0, "blue")?;
    a = scale_val(a, 1.0, "alpha")?.clamp(0.0, 1.0);

    Ok(Value::Color(Color::rgba_fmt(c.r, c.g, c.b, a, c.format.clone_with(r, g, b))))
}

// ── Legacy (RGB/HSL/HWB) ──

fn adjust_legacy(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let mut r = c.r as f64;
    let mut g = c.g as f64;
    let mut b = c.b as f64;
    let mut a = c.a;
    let mut has_hsl = false;
    let (mut h, mut s, mut l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);
    let mut has_hwb = false;
    let (mut hw, mut hb) = {
        let r = c.r as f64 / 255.0;
        let g = c.g as f64 / 255.0;
        let b = c.b as f64 / 255.0;
        (r.min(g).min(b), 1.0 - r.max(g).max(b))
    };

    if let Some(v) = get_num(kw_args, "red")? { r += v; }
    if let Some(v) = get_num(kw_args, "green")? { g += v; }
    if let Some(v) = get_num(kw_args, "blue")? { b += v; }
    if let Some(v) = get_num(kw_args, "alpha")? { a += v; }
    if let Some(v) = get_num(kw_args, "hue")? { h = (h + v).rem_euclid(360.0); has_hsl = true; has_hwb = true; }
    if let Some(v) = get_pct_or_num(kw_args, "saturation")? { s = (s + v).clamp(0.0, 1.0); has_hsl = true; }
    if let Some(v) = get_pct_or_num(kw_args, "lightness")? { l = (l + v).clamp(0.0, 1.0); has_hsl = true; }
    if let Some(v) = get_pct_or_num(kw_args, "whiteness")? { hw = (hw + v).clamp(0.0, 1.0); has_hwb = true; }
    if let Some(v) = get_pct_or_num(kw_args, "blackness")? { hb = (hb + v).clamp(0.0, 1.0); has_hwb = true; }
    if has_hwb {
        let new_c = Evaluator::hwb_to_rgb(h, hw, hb, 1.0);
        r = new_c.r as f64; g = new_c.g as f64; b = new_c.b as f64;
    } else if has_hsl {
        let new_c = Evaluator::hsl_to_rgb(h, s, l);
        r = new_c.r as f64; g = new_c.g as f64; b = new_c.b as f64;
    }
    let fmt = if has_hsl || has_hwb { ColorFormat::RgbPercent(h, s, l) } else { ColorFormat::Auto };
    Ok(Value::Color(Color::rgba_fmt(
        r.round().clamp(0.0, 255.0) as u8,
        g.round().clamp(0.0, 255.0) as u8,
        b.round().clamp(0.0, 255.0) as u8,
        a.clamp(0.0, 1.0),
        fmt,
    )))
}

fn change_legacy(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let mut r = c.r as f64;
    let mut g = c.g as f64;
    let mut b = c.b as f64;
    let mut a = c.a;
    let mut has_hsl = false;
    let (mut h, mut s, mut l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);
    let mut has_hwb = false;
    let (mut hw, mut hb) = {
        let r = c.r as f64 / 255.0;
        let g = c.g as f64 / 255.0;
        let b = c.b as f64 / 255.0;
        (r.min(g).min(b), 1.0 - r.max(g).max(b))
    };

    if let Some(v) = get_num(kw_args, "red")? { r = v; }
    if let Some(v) = get_num(kw_args, "green")? { g = v; }
    if let Some(v) = get_num(kw_args, "blue")? { b = v; }
    if let Some(v) = get_num(kw_args, "alpha")? { a = v; }
    if let Some(v) = get_num(kw_args, "hue")? { h = v.rem_euclid(360.0); has_hsl = true; has_hwb = true; }
    if let Some(v) = get_pct_or_num(kw_args, "saturation")? { s = v.clamp(0.0, 1.0); has_hsl = true; }
    if let Some(v) = get_pct_or_num(kw_args, "lightness")? { l = v.clamp(0.0, 1.0); has_hsl = true; }
    if let Some(v) = get_pct_or_num(kw_args, "whiteness")? { hw = v.clamp(0.0, 1.0); has_hwb = true; }
    if let Some(v) = get_pct_or_num(kw_args, "blackness")? { hb = v.clamp(0.0, 1.0); has_hwb = true; }
    if has_hwb {
        let new_c = Evaluator::hwb_to_rgb(h, hw, hb, 1.0);
        r = new_c.r as f64; g = new_c.g as f64; b = new_c.b as f64;
    } else if has_hsl {
        let new_c = Evaluator::hsl_to_rgb(h, s, l);
        r = new_c.r as f64; g = new_c.g as f64; b = new_c.b as f64;
    }
    let fmt = if has_hsl || has_hwb { ColorFormat::RgbPercent(h, s, l) } else { ColorFormat::Auto };
    Ok(Value::Color(Color::rgba_fmt(
        r.round().clamp(0.0, 255.0) as u8,
        g.round().clamp(0.0, 255.0) as u8,
        b.round().clamp(0.0, 255.0) as u8,
        a.clamp(0.0, 1.0),
        fmt,
    )))
}

fn scale_legacy(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let mut r = c.r as f64;
    let mut g = c.g as f64;
    let mut b = c.b as f64;
    let mut a = c.a;
    let mut has_hsl = false;
    let (h, mut s, mut l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);

    let scale_val = |val: f64, max: f64, kw: &str| -> Result<f64> {
        if let Some(Value::Number(n, _)) = kw_args.get(kw) {
            let pct = *n / 100.0;
            if pct >= 0.0 { Ok(val + (max - val) * pct) }
            else { Ok(val + val * pct) }
        } else { Ok(val) }
    };
    r = scale_val(r, 255.0, "red")?;
    g = scale_val(g, 255.0, "green")?;
    b = scale_val(b, 255.0, "blue")?;
    a = scale_val(a, 1.0, "alpha")?;
    if kw_args.contains_key("saturation") {
        s = scale_val(s, 1.0, "saturation")?.clamp(0.0, 1.0);
        has_hsl = true;
    }
    if kw_args.contains_key("lightness") {
        l = scale_val(l, 1.0, "lightness")?.clamp(0.0, 1.0);
        has_hsl = true;
    }
    if has_hsl {
        let new_c = Evaluator::hsl_to_rgb(h, s, l);
        r = new_c.r as f64; g = new_c.g as f64; b = new_c.b as f64;
    }
    let fmt = if has_hsl { ColorFormat::RgbPercent(h, s, l) } else { ColorFormat::Auto };
    Ok(Value::Color(Color::rgba_fmt(
        r.round().clamp(0.0, 255.0) as u8,
        g.round().clamp(0.0, 255.0) as u8,
        b.round().clamp(0.0, 255.0) as u8,
        a.clamp(0.0, 1.0),
        fmt,
    )))
}
