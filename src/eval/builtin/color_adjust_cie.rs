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

use crate::error::Result;
use crate::parse::ast::{Color, ColorSpace, Value};
use std::collections::HashMap;

use super::color_adjust::{apply_kw, apply_pct_kw, scale_channel};

// ── Oklch ────────────────────────────────────────────────────────────────

pub(super) fn adjust_oklch(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
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

pub(super) fn change_oklch(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
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

pub(super) fn scale_oklch(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
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

pub(super) fn adjust_oklab(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
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

pub(super) fn change_oklab(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
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

pub(super) fn scale_oklab(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
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

pub(super) fn adjust_lch(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
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

pub(super) fn change_lch(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
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

pub(super) fn scale_lch(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
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

pub(super) fn adjust_lab(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
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

pub(super) fn change_lab(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
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

pub(super) fn scale_lab(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
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
