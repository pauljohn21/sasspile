#![allow(
    clippy::many_single_char_names,
    clippy::single_char_pattern,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
//! 颜色格式化辅助函数。
//!
//! hue/百分比/alpha 值的浮点精度截断和格式化，
//! HSL → RGB 百分比转换。

use crate::consts::{FLOAT_PRECISION_INV, HUE_MAX, PCT_SCALE};

/// 格式化 hue 值——截断到 10 位小数。
/// NaN 输出为 `none`（CSS Color 4 missing 通道）。
pub(crate) fn format_hue(h: f64) -> String {
    match h.is_nan() {
        true => return "none".to_string(),
        false => {}
    }
    let h = (h * FLOAT_PRECISION_INV).round() / FLOAT_PRECISION_INV;
    match h.fract() == 0.0 {
        true => format!("{}", h as i64),
        false => format!("{h}"),
    }
}

/// 格式化百分比值（0.0-1.0 → 0%-100%），浮点精度截断。
/// NaN 输出为 `none`。
pub(crate) fn format_pct(v: f64) -> String {
    match v.is_nan() {
        true => return "none".to_string(),
        false => {}
    }
    let pct = v * PCT_SCALE;
    let pct = (pct * FLOAT_PRECISION_INV).round() / FLOAT_PRECISION_INV;
    match pct.fract() == 0.0 {
        true => format!("{}", pct as i64),
        false => format!("{pct}"),
    }
}

/// 格式化百分比值（0.0-100.0 → 0%-100%），用于 rgb(%) 输出。
pub(crate) fn format_pct_val(v: f64) -> String {
    let v = (v * FLOAT_PRECISION_INV).round() / FLOAT_PRECISION_INV;
    match v.fract() == 0.0 {
        true => format!("{}", v as i64),
        false => format!("{v}"),
    }
}

/// HSL → RGB 百分比转换（用于百分比输出）。
/// 返回 (r%, g%, b%)，范围 0.0-100.0。
pub(crate) fn hsl_to_rgb_percent(h: f64, s: f64, l: f64) -> (f64, f64, f64) {
    let h = h.rem_euclid(HUE_MAX);
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r1, g1, b1) = match h {
        h if h < 60.0 => (c, x, 0.0),
        h if h < 120.0 => (x, c, 0.0),
        h if h < 180.0 => (0.0, c, x),
        h if h < 240.0 => (0.0, x, c),
        h if h < 300.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (
        (r1 + m) * PCT_SCALE,
        (g1 + m) * PCT_SCALE,
        (b1 + m) * PCT_SCALE,
    )
}

/// 格式化 alpha 值。
pub(crate) fn format_alpha(a: f64) -> String {
    match a.is_nan() {
        true => return "none".to_string(),
        false => {}
    }
    match a.fract() == 0.0 {
        true => format!("{}", a as i64),
        false => format!("{a}"),
    }
}

/// HWB → HSL 转换（内联实现，供 display.rs 序列化使用）。
/// 基于 CSS Color 4 规范的 HWB→HSL 算法。
pub(crate) fn hwb_to_hsl_inline(h: f64, w: f64, b: f64) -> (f64, f64, f64) {
    let h_norm = h.rem_euclid(360.0) / 360.0;
    let (w, b) = if w + b > 1.0 {
        (w / (w + b), b / (w + b))
    } else {
        (w, b)
    };
    let factor = 1.0 - w - b;

    let hue_to_rgb = |hue: f64| -> f64 {
        let mut hue = hue;
        if hue < 0.0 {
            hue += 1.0;
        }
        if hue > 1.0 {
            hue -= 1.0;
        }
        match hue {
            h if h < 1.0 / 6.0 => w + factor * hue * 6.0,
            h if h < 0.5 => w + factor,
            h if h < 2.0 / 3.0 => w + factor * (2.0 / 3.0 - hue) * 6.0,
            _ => w,
        }
    };
    let r = hue_to_rgb(h_norm + 1.0 / 3.0);
    let g = hue_to_rgb(h_norm);
    let bl = hue_to_rgb(h_norm - 1.0 / 3.0);

    let max = r.max(g).max(bl);
    let min = r.min(g).min(bl);
    let l = f64::midpoint(max, min);
    let delta = max - min;
    let s = if delta < 1e-12 {
        0.0
    } else {
        delta / (1.0 - (2.0 * l - 1.0).abs())
    };
    (h, s, l)
}
