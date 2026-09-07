#![allow(
    clippy::many_single_char_names,
    clippy::single_char_pattern,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
//! `color.adjust` / `color.change` / `color.scale` — CIE 空间实现。
//!
//! 包含 Oklch/Oklab/Lch/Lab 四个 CIE 颜色空间的调整/变化/缩放函数。
//! 与 `color_adjust.rs` 共享 `apply_channel` / `scale_channel` 统一链式 API。

use crate::error::Result;
use crate::parse::ast::{Color, ColorSpace, Value};
use std::collections::HashMap;

use super::color_adjust::{
    angle_deg, apply_cie_channel, apply_channel, cie_channel, raw_value, scale_channel,
};

// ── Oklch：lightness/chroma/hue 内部尺度分别为 0-1 / 0-~0.5 / 0-360 ────────

pub(super) fn adjust_oklch(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    // lightness: unitless=n(0-1 delta), percent=n/100(=pp), none→NaN
    let l = apply_cie_channel(c.channels[0], kw_args, "lightness", 1.0, |v, d| {
        (v + d).clamp(0.0, 1.0)
    });
    // chroma: unitless=n(raw), percent=n(percentage points on 0-scale? no—see below)
    let ch = apply_cie_channel(c.channels[1], kw_args, "chroma", 0.4, |v, d| (v + d).max(0.0));
    let h = apply_channel(c.channels[2], kw_args, "hue", angle_deg, |v, d| {
        (v + d).rem_euclid(360.0)
    });
    let a = apply_channel(c.a, kw_args, "alpha", raw_value, |v, d| (v + d).clamp(0.0, 1.0));

    Ok(Value::Color(Color::with_space(
        ColorSpace::Oklch,
        [l, ch, h],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

pub(super) fn change_oklch(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    // lightness: unitless=n(0-1), percent=n/100, none→NaN
    let l = apply_cie_channel(c.channels[0], kw_args, "lightness", 1.0, |_v, d| d.clamp(0.0, 1.0));
    let a = apply_channel(c.a, kw_args, "alpha", raw_value, |_v, d| d.clamp(0.0, 1.0));
    // chroma + hue: handle negative chroma normalization and NaN preservation
    let base_h = apply_channel(c.channels[2], kw_args, "hue", angle_deg, |_v, d| d.rem_euclid(360.0));
    let chroma_input = cie_channel(kw_args, "chroma", 0.4);
    let (ch, h) = match chroma_input {
        None => (c.channels[1], base_h),             // no chroma change
        Some(d) if d.is_nan() => (f64::NAN, base_h),  // none → chroma=NaN, hue unchanged
        Some(d) if d < 0.0 => (-d, (base_h + 180.0).rem_euclid(360.0)), // negative → abs + hue+180
        Some(d) => (d, base_h),                       // positive → use directly
    };

    Ok(Value::Color(Color::with_space(
        ColorSpace::Oklch,
        [l, ch, h],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

pub(super) fn scale_oklch(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let l = scale_channel(c.channels[0], 1.0, kw_args, "lightness").clamp(0.0, 1.0);
    let ch = scale_channel(c.channels[1], f64::MAX, kw_args, "chroma").max(0.0);
    let a = scale_channel(c.a, 1.0, kw_args, "alpha").clamp(0.0, 1.0);

    Ok(Value::Color(Color::with_space(
        ColorSpace::Oklch,
        [l, ch, c.channels[2]],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

// ── Oklab：lightness 0-1 / a,b 约 ±0.4 （实际范围较大，用 0.4 作 max 估算）

pub(super) fn adjust_oklab(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let l = apply_cie_channel(c.channels[0], kw_args, "lightness", 1.0, |v, d| {
        (v + d).clamp(0.0, 1.0)
    });
    let a_v = apply_cie_channel(c.channels[1], kw_args, "a", 0.4, |v, d| v + d);
    let b_v = apply_cie_channel(c.channels[2], kw_args, "b", 0.4, |v, d| v + d);
    let a = apply_channel(c.a, kw_args, "alpha", raw_value, |v, d| (v + d).clamp(0.0, 1.0));

    Ok(Value::Color(Color::with_space(
        ColorSpace::Oklab,
        [l, a_v, b_v],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

pub(super) fn change_oklab(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let l = apply_cie_channel(c.channels[0], kw_args, "lightness", 1.0, |_v, d| d.clamp(0.0, 1.0));
    let a_v = apply_cie_channel(c.channels[1], kw_args, "a", 0.4, |_v, d| d);
    let b_v = apply_cie_channel(c.channels[2], kw_args, "b", 0.4, |_v, d| d);
    let a = apply_channel(c.a, kw_args, "alpha", raw_value, |_v, d| d.clamp(0.0, 1.0));

    Ok(Value::Color(Color::with_space(
        ColorSpace::Oklab,
        [l, a_v, b_v],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

pub(super) fn scale_oklab(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let l = scale_channel(c.channels[0], 1.0, kw_args, "lightness").clamp(0.0, 1.0);
    // 使用对称 max=0.4 涵盖 oklab a/b 的值域
    let a_v = scale_channel(c.channels[1], 0.4, kw_args, "a");
    let b_v = scale_channel(c.channels[2], 0.4, kw_args, "b");
    let a = scale_channel(c.a, 1.0, kw_args, "alpha").clamp(0.0, 1.0);

    Ok(Value::Color(Color::with_space(
        ColorSpace::Oklab,
        [l, a_v, b_v],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

// ── Lch：lightness 0-100 / chroma 无理论上限 / hue 0-360 ──────────────

pub(super) fn adjust_lch(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let l = apply_cie_channel(c.channels[0], kw_args, "lightness", 100.0, |v, d| {
        (v + d).clamp(0.0, 100.0)
    });
    // chroma: unitless=n(raw), percent=n%*150(近似 max), none→NaN
    let ch = apply_cie_channel(c.channels[1], kw_args, "chroma", 150.0, |v, d| (v + d).max(0.0));
    let h = apply_channel(c.channels[2], kw_args, "hue", angle_deg, |v, d| {
        (v + d).rem_euclid(360.0)
    });
    let a = apply_channel(c.a, kw_args, "alpha", raw_value, |v, d| (v + d).clamp(0.0, 1.0));

    Ok(Value::Color(Color::with_space(
        ColorSpace::Lch,
        [l, ch, h],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

pub(super) fn change_lch(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let l = apply_cie_channel(c.channels[0], kw_args, "lightness", 100.0, |_v, d| {
        d.clamp(0.0, 100.0)
    });
    let a = apply_channel(c.a, kw_args, "alpha", raw_value, |_v, d| d.clamp(0.0, 1.0));
    // chroma + hue: handle negative chroma normalization and NaN preservation
    let base_h = apply_channel(c.channels[2], kw_args, "hue", angle_deg, |_v, d| d.rem_euclid(360.0));
    let chroma_input = cie_channel(kw_args, "chroma", 150.0);
    let (ch, h) = match chroma_input {
        None => (c.channels[1], base_h),
        Some(d) if d.is_nan() => (f64::NAN, base_h),
        Some(d) if d < 0.0 => (-d, (base_h + 180.0).rem_euclid(360.0)),
        Some(d) => (d, base_h),
    };

    Ok(Value::Color(Color::with_space(
        ColorSpace::Lch,
        [l, ch, h],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

pub(super) fn scale_lch(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let l = scale_channel(c.channels[0], 100.0, kw_args, "lightness").clamp(0.0, 100.0);
    let ch = scale_channel(c.channels[1], 150.0, kw_args, "chroma").max(0.0);
    let a = scale_channel(c.a, 1.0, kw_args, "alpha").clamp(0.0, 1.0);

    Ok(Value::Color(Color::with_space(
        ColorSpace::Lch,
        [l, ch, c.channels[2]],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

// ── Lab：lightness 0-100 / a,b 范围约 ±125 ──────────────────────────────

pub(super) fn adjust_lab(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let l = apply_cie_channel(c.channels[0], kw_args, "lightness", 100.0, |v, d| {
        (v + d).clamp(0.0, 100.0)
    });
    // a,b: unitless=n(raw), percent=n%*125, none→NaN
    let a_v = apply_cie_channel(c.channels[1], kw_args, "a", 125.0, |v, d| v + d);
    let b_v = apply_cie_channel(c.channels[2], kw_args, "b", 125.0, |v, d| v + d);
    let a = apply_channel(c.a, kw_args, "alpha", raw_value, |v, d| (v + d).clamp(0.0, 1.0));

    Ok(Value::Color(Color::with_space(
        ColorSpace::Lab,
        [l, a_v, b_v],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

pub(super) fn change_lab(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let l = apply_cie_channel(c.channels[0], kw_args, "lightness", 100.0, |_v, d| {
        d.clamp(0.0, 100.0)
    });
    let a_v = apply_cie_channel(c.channels[1], kw_args, "a", 125.0, |_v, d| d);
    let b_v = apply_cie_channel(c.channels[2], kw_args, "b", 125.0, |_v, d| d);
    let a = apply_channel(c.a, kw_args, "alpha", raw_value, |_v, d| d.clamp(0.0, 1.0));

    Ok(Value::Color(Color::with_space(
        ColorSpace::Lab,
        [l, a_v, b_v],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

pub(super) fn scale_lab(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let l = scale_channel(c.channels[0], 100.0, kw_args, "lightness").clamp(0.0, 100.0);
    let a_v = scale_channel(c.channels[1], 125.0, kw_args, "a");
    let b_v = scale_channel(c.channels[2], 125.0, kw_args, "b");
    let a = scale_channel(c.a, 1.0, kw_args, "alpha").clamp(0.0, 1.0);

    Ok(Value::Color(Color::with_space(
        ColorSpace::Lab,
        [l, a_v, b_v],
        a,
        c.output,
        c.legacy_rgb,
    )))
}
