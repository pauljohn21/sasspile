#![allow(
    clippy::many_single_char_names,
    clippy::single_char_pattern,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
//! Color 序列化——CSS Color 4 色彩空间 Auto 输出。
//!
//! 从 `display_color.rs` 提取以避免单文件超限。

use super::*;
use crate::consts::{ALPHA_TOLERANCE, DEG_UNIT, FLOAT_NOISE_THRESHOLD, FLOAT_PRECISION_INV, PCT_SCALE};

/// 清理颜色分量的浮点噪声——将极小值归零。
fn clean_num(v: f64) -> f64 {
    match v.abs() < FLOAT_NOISE_THRESHOLD {
        true => 0.0,
        false => v,
    }
}

/// 格式化百分比值（输入已 0-100 范围）——NaN 输出为 "none"，正常值输出为 "N%"。
fn fmt_pct_with_pctval(v: f64) -> String {
    match v.is_nan() {
        true => "none".to_string(),
        false => {
            let pct = (v * FLOAT_PRECISION_INV).round() / FLOAT_PRECISION_INV;
            match pct.fract() == 0.0 {
                true => format!("{}%", pct as i64),
                false => format!("{pct}%"),
            }
        }
    }
}

/// 格式化百分比值（输入为 0-1 范围）——NaN 输出为 "none"，正常值输出为 "N%"。
fn fmt_pct_with_01(v: f64) -> String {
    match v.is_nan() {
        true => "none".to_string(),
        false => {
            let pct = (v * PCT_SCALE * FLOAT_PRECISION_INV).round() / FLOAT_PRECISION_INV;
            match pct.fract() == 0.0 {
                true => format!("{}%", pct as i64),
                false => format!("{pct}%"),
            }
        }
    }
}

/// 格式化浮点数——截断到 10 位小数（与 SCSS 规范一致）。
/// NaN → "none"，±inf → "calc(±infinity)"（Sass 语法）。
fn format_num(n: f64) -> String {
    match (n.is_nan(), n.is_infinite()) {
        (true, _) => "none".to_string(),
        (_, true) => match n > 0.0 {
            true => "calc(infinity)".to_string(),
            false => "calc(-infinity)".to_string(),
        },
        _ => {
            let n = (n * FLOAT_PRECISION_INV).round() / FLOAT_PRECISION_INV;
            match n.fract() == 0.0 {
                true => format!("{n:.0}"),
                false => format!("{n}"),
            }
        }
    }
}

/// alpha 通道是否近似 1.0（视为不透明）。
fn alpha_is_opaque(a: f64) -> bool {
    (a - 1.0).abs() < ALPHA_TOLERANCE
}

/// 格式化 color() 函数输出——辅助闭包，处理通用的 channels + alpha 分派。
fn fmt_color_fn(
    name: &str,
    r: f64,
    g: f64,
    b: f64,
    c: &Color,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    match alpha_is_opaque(c.a) {
        true => write!(f, "{name} {} {} {})", format_num(r), format_num(g), format_num(b)),
        false => write!(
            f,
            "{name} {} {} {} / {})",
            format_num(r),
            format_num(g),
            format_num(b),
            format_alpha(c.a)
        ),
    }
}

/// 格式化 Color 值为 CSS 字符串（Auto 模式，按色彩空间分派）。
pub(super) fn fmt_color_auto(
    c: &Color,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    match c.space {
        ColorSpace::Hsl => {
            let (h, s, l) = (c.channels[0], c.channels[1], c.channels[2]);
            // CSS Color 4 missing 通道: 空格分隔，数字加后缀（h→deg, s/l→%），NaN→none
            match h.is_nan() || s.is_nan() || l.is_nan() {
                true => {
                    let h_str = match h.is_nan() {
                        true => "none".to_string(),
                        false => format_hue(h) + DEG_UNIT,
                    };
                    let s_str = match s.is_nan() {
                        true => "none".to_string(),
                        false => format!("{}%", format_pct(s)),
                    };
                    let l_str = match l.is_nan() {
                        true => "none".to_string(),
                        false => format!("{}%", format_pct(l)),
                    };
                    match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                        true => write!(f, "hsl({h_str} {s_str} {l_str})"),
                        false => write!(
                            f,
                            "hsl({h_str} {s_str} {l_str} / {})",
                            format_alpha(c.a)
                        ),
                    }
                }
                false => {
                    let hue_str = format_hue(h);
                    match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                        true => write!(
                            f,
                            "hsl({hue_str}, {}%, {}%)",
                            format_pct(s),
                            format_pct(l)
                        ),
                        false => write!(
                            f,
                            "hsla({hue_str}, {}%, {}%, {})",
                            format_pct(s),
                            format_pct(l),
                            format_alpha(c.a)
                        ),
                    }
                }
            }
        }
        ColorSpace::Hwb => {
            let (h, w, bk) = (c.channels[0], c.channels[1], c.channels[2]);
            // SCSS 规范：HWB 全有效值时 Auto 输出规范化为 HSL；有 NaN 时保留 hwb() 格式
            match h.is_nan() || w.is_nan() || bk.is_nan() || c.a.is_nan() {
                true => {
                    let h_str = match h.is_nan() {
                        true => "none".to_string(),
                        false => format_hue(h) + DEG_UNIT,
                    };
                    let w_str = match w.is_nan() {
                        true => "none".to_string(),
                        false => format!("{}%", format_pct(w)),
                    };
                    let bk_str = match bk.is_nan() {
                        true => "none".to_string(),
                        false => format!("{}%", format_pct(bk)),
                    };
                    match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                        true => write!(f, "hwb({h_str} {w_str} {bk_str})"),
                        false => write!(
                            f,
                            "hwb({h_str} {w_str} {bk_str} / {})",
                            format_alpha(c.a)
                        ),
                    }
                }
                false => {
                    let (hsl_h, hsl_s, hsl_l) = hwb_to_hsl_inline(h, w, bk);
                    let hue_str = format_hue(hsl_h);
                    let alpha_ok = (c.a - 1.0).abs() < ALPHA_TOLERANCE;
                    // 全有效值 HWB 序列化为 HSL 时，alpha=1 则查找命名色
                    match (alpha_ok, crate::eval::Evaluator::reverse_lookup_named_color(c)) {
                        (true, Some(name)) => write!(f, "{name}"),
                        _ => match alpha_ok {
                            true => write!(
                                f,
                                "hsl({}, {}%, {}%)",
                                hue_str,
                                format_pct(hsl_s),
                                format_pct(hsl_l)
                            ),
                            false => write!(
                                f,
                                "hsla({}, {}%, {}%, {})",
                                hue_str,
                                format_pct(hsl_s),
                                format_pct(hsl_l),
                                format_alpha(c.a)
                            ),
                        },
                    }
                }
            }
        }
        ColorSpace::Lab => {
            let (l, a, b) = (c.channels[0], c.channels[1], c.channels[2]);
            let l_str = fmt_pct_with_pctval(l);
            let a_clean = clean_num(a);
            let b_clean = clean_num(b);
            match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                true => write!(
                    f,
                    "lab({} {} {})",
                    l_str,
                    format_num(a_clean),
                    format_num(b_clean)
                ),
                false => write!(
                    f,
                    "lab({} {} {} / {})",
                    l_str,
                    format_num(a_clean),
                    format_num(b_clean),
                    format_alpha(c.a)
                ),
            }
        }
        ColorSpace::Lch => {
            let (l, ch, h) = (c.channels[0], c.channels[1], c.channels[2]);
            let l_str = fmt_pct_with_pctval(l);
            let ch_str = format_num(ch);
            let ch_is_zero = !ch.is_nan() && ch.abs() < FLOAT_NOISE_THRESHOLD;
            let h_str = match ch_is_zero || h.is_nan() {
                true => "none".to_string(),
                false => format!("{}{}", format_hue(h), DEG_UNIT),
            };
            match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                true => write!(f, "lch({l_str} {ch_str} {h_str})"),
                false => write!(
                    f,
                    "lch({} {} {} / {})",
                    l_str,
                    ch_str,
                    h_str,
                    format_alpha(c.a)
                ),
            }
        }
        ColorSpace::Oklab => {
            let (l, a, b) = (c.channels[0], c.channels[1], c.channels[2]);
            let l_str = fmt_pct_with_01(l);
            let a_clean = clean_num(a);
            let b_clean = clean_num(b);
            match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                true => write!(
                    f,
                    "oklab({} {} {})",
                    l_str,
                    format_num(a_clean),
                    format_num(b_clean)
                ),
                false => write!(
                    f,
                    "oklab({} {} {} / {})",
                    l_str,
                    format_num(a_clean),
                    format_num(b_clean),
                    format_alpha(c.a)
                ),
            }
        }
        ColorSpace::Oklch => {
            let (l, ch, h) = (c.channels[0], c.channels[1], c.channels[2]);
            let l_str = fmt_pct_with_01(l);
            let ch_str = format_num(ch);
            let ch_is_zero = !ch.is_nan() && ch.abs() < FLOAT_NOISE_THRESHOLD;
            let h_str = match ch_is_zero || h.is_nan() {
                true => "none".to_string(),
                false => format!("{}{}", format_hue(h), DEG_UNIT),
            };
            match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                true => write!(f, "oklch({l_str} {ch_str} {h_str})"),
                false => write!(
                    f,
                    "oklch({} {} {} / {})",
                    l_str,
                    ch_str,
                    h_str,
                    format_alpha(c.a)
                ),
            }
        }
        ColorSpace::DisplayP3 => {
            fmt_color_fn("color(display-p3", c.channels[0], c.channels[1], c.channels[2], c, f)
        }
        ColorSpace::DisplayP3Linear => {
            fmt_color_fn("color(display-p3-linear", c.channels[0], c.channels[1], c.channels[2], c, f)
        }
        ColorSpace::Srgb => {
            fmt_color_fn("color(srgb", c.channels[0], c.channels[1], c.channels[2], c, f)
        }
        ColorSpace::SrgbLinear => {
            fmt_color_fn("color(srgb-linear", c.channels[0], c.channels[1], c.channels[2], c, f)
        }
        ColorSpace::A98Rgb => {
            fmt_color_fn("color(a98-rgb", c.channels[0], c.channels[1], c.channels[2], c, f)
        }
        ColorSpace::ProphotoRgb => {
            fmt_color_fn("color(prophoto-rgb", c.channels[0], c.channels[1], c.channels[2], c, f)
        }
        ColorSpace::Rec2020 => {
            fmt_color_fn("color(rec2020", c.channels[0], c.channels[1], c.channels[2], c, f)
        }
        ColorSpace::XyzD65 => {
            fmt_color_fn("color(xyz", c.channels[0], c.channels[1], c.channels[2], c, f)
        }
        ColorSpace::XyzD50 => {
            fmt_color_fn("color(xyz-d50", c.channels[0], c.channels[1], c.channels[2], c, f)
        }
        ColorSpace::Rgb => {
            // Auto + Rgb = hex / 命名色 / rgba
            match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                true => match crate::eval::Evaluator::reverse_lookup_named_color(c) {
                    Some(name) => write!(f, "{name}"),
                    None => write!(
                        f,
                        "#{:02x}{:02x}{:02x}",
                        c.legacy_rgb[0].round() as u8,
                        c.legacy_rgb[1].round() as u8,
                        c.legacy_rgb[2].round() as u8
                    ),
                },
                false => write!(
                    f,
                    "rgba({}, {}, {}, {})",
                    c.legacy_rgb[0].round() as u8,
                    c.legacy_rgb[1].round() as u8,
                    c.legacy_rgb[2].round() as u8,
                    format_alpha(c.a)
                ),
            }
        }
    }
}
