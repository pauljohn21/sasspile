//! 颜色格式化辅助函数。
//!
//! hue/百分比/alpha 值的浮点精度截断和格式化，
//! HSL → RGB 百分比转换。

use crate::consts::{FLOAT_PRECISION_INV, HUE_MAX, PCT_SCALE};

/// 格式化 hue 值——截断到 10 位小数。
pub(crate) fn format_hue(h: f64) -> String {
    let h = (h * FLOAT_PRECISION_INV).round() / FLOAT_PRECISION_INV;
    if h.fract() == 0.0 {
        format!("{}", h as i64)
    } else {
        format!("{h}")
    }
}

/// 格式化百分比值（0.0-1.0 → 0%-100%），浮点精度截断。
pub(crate) fn format_pct(v: f64) -> String {
    let pct = v * PCT_SCALE;
    let pct = (pct * FLOAT_PRECISION_INV).round() / FLOAT_PRECISION_INV;
    if pct.fract() == 0.0 {
        format!("{}", pct as i64)
    } else {
        format!("{pct}")
    }
}

/// 格式化百分比值（0.0-100.0 → 0%-100%），用于 rgb(%) 输出。
pub(crate) fn format_pct_val(v: f64) -> String {
    let v = (v * FLOAT_PRECISION_INV).round() / FLOAT_PRECISION_INV;
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

/// HSL → RGB 百分比转换（用于百分比输出）。
/// 返回 (r%, g%, b%)，范围 0.0-100.0。
pub(crate) fn hsl_to_rgb_percent(h: f64, s: f64, l: f64) -> (f64, f64, f64) {
    let h = h.rem_euclid(HUE_MAX);
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r1, g1, b1) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    ((r1 + m) * PCT_SCALE, (g1 + m) * PCT_SCALE, (b1 + m) * PCT_SCALE)
}

/// 格式化 alpha 值。
pub(crate) fn format_alpha(a: f64) -> String {
    if a.fract() == 0.0 {
        format!("{}", a as i64)
    } else {
        format!("{a}")
    }
}
