#![allow(
    clippy::many_single_char_names,
    clippy::single_char_pattern,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
//! Value::Color 的 Display 实现。
//!
//! 从 display.rs 提取以避免单文件超限。
//! 支持所有 CSS Color 4 颜色空间的序列化输出。

use super::*;
use crate::consts::{ALPHA_TOLERANCE, DEG_UNIT, FLOAT_NOISE_THRESHOLD, FLOAT_PRECISION_INV, PCT_ROUND_THRESHOLD, PCT_SCALE};

/// 清理颜色分量的浮点噪声——将极小值归零。
fn clean_num(v: f64) -> f64 {
    match v.abs() < FLOAT_NOISE_THRESHOLD {
        true => 0.0,
        false => v,
    }
}

/// 清理百分比分量——接近 0 或 100 时归整。
fn clean_pct(v: f64) -> f64 {
    match v {
        v if v.abs() < FLOAT_NOISE_THRESHOLD => 0.0,
        v if (v - PCT_SCALE).abs() < PCT_ROUND_THRESHOLD => PCT_SCALE,
        _ => v,
    }
}

/// 格式化浮点数——截断到 10 位小数（与 SCSS 规范一致）。
/// NaN 输出为 `none`（CSS Color 4 missing 通道）。
fn format_num(n: f64) -> String {
    match n.is_nan() {
        true => return "none".to_string(),
        false => {}
    }
    let n = (n * FLOAT_PRECISION_INV).round() / FLOAT_PRECISION_INV;
    match n.fract() == 0.0 {
        true => format!("{}", n as i64),
        false => format!("{n}"),
    }
}

/// 格式化 Color 值为 CSS 字符串。
pub(super) fn fmt_color(
    c: &Color,
    output: ColorOutput,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    match output {
        ColorOutput::RgbExplicit => {
            match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                true => write!(
                    f,
                    "rgb({}, {}, {})",
                    c.legacy_rgb[0].round() as u8,
                    c.legacy_rgb[1].round() as u8,
                    c.legacy_rgb[2].round() as u8
                ),
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
        ColorOutput::RgbModern => {
            // 现代语法：rgb(R G B / A)，空格分隔，NaN → "none"
            let fmt_ch = |v: f64| -> String {
                match v.is_nan() {
                    true => "none".to_string(),
                    false => {
                        let rounded = v.round() as i64;
                        format!("{}", rounded)
                    }
                }
            };
            let rs = fmt_ch(c.legacy_rgb[0]);
            let gs = fmt_ch(c.legacy_rgb[1]);
            let bs = fmt_ch(c.legacy_rgb[2]);
            match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                true => write!(f, "rgb({rs} {gs} {bs})"),
                false => {
                    let astr = match c.a.is_nan() {
                        true => "none".to_string(),
                        false => format_alpha(c.a),
                    };
                    write!(f, "rgb({rs} {gs} {bs} / {astr})")
                }
            }
        }
        ColorOutput::RgbPercent => {
            // 根据色彩空间选择 RGB 百分比来源
            let (rp, gp, bp) = match c.space {
                ColorSpace::Hsl => {
                    // channels 存储 HSL 值 (h, s, l)，需转换
                    let (h, s, l) = (c.channels[0], c.channels[1], c.channels[2]);
                    hsl_to_rgb_percent(h, s, l)
                }
                _ => {
                    // 其他空间（如 change-color HWB）：legacy_rgb 已是 0-255 RGB
                    (c.legacy_rgb[0] / 2.55, c.legacy_rgb[1] / 2.55, c.legacy_rgb[2] / 2.55)
                }
            };
            // legacy_rgb 整数检查——基于 0-255 尺度，避免百分比尺度精度损失
            let legacy_int = (c.legacy_rgb[0] - c.legacy_rgb[0].round()).abs() < FLOAT_NOISE_THRESHOLD
                && (c.legacy_rgb[1] - c.legacy_rgb[1].round()).abs() < FLOAT_NOISE_THRESHOLD
                && (c.legacy_rgb[2] - c.legacy_rgb[2].round()).abs() < FLOAT_NOISE_THRESHOLD;
            let rp_r = c.legacy_rgb[0].round() as u8;
            let gp_r = c.legacy_rgb[1].round() as u8;
            let bp_r = c.legacy_rgb[2].round() as u8;
            // 检查是否匹配命名颜色，优先输出名称
            let alpha_ok = (c.a - 1.0).abs() < ALPHA_TOLERANCE;
            let named = crate::eval::Evaluator::reverse_lookup_named_color(c);
            match (alpha_ok, named) {
                (true, Some(name)) => write!(f, "{name}"),
                (true, None) => match legacy_int {
                    true => write!(f, "#{:02x}{:02x}{:02x}", rp_r, gp_r, bp_r),
                    false => write!(
                        f,
                        "rgb({}%, {}%, {}%)",
                        format_pct_val(rp),
                        format_pct_val(gp),
                        format_pct_val(bp)
                    ),
                },
                (false, _) => write!(
                    f,
                    "rgba({}%, {}%, {}%, {})",
                    format_pct_val(rp),
                    format_pct_val(gp),
                    format_pct_val(bp),
                    format_alpha(c.a)
                ),
            }
        }
        ColorOutput::Auto => match c.space {
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
                // SCSS 规范：HWB 全有效值（无 NaN，包括 alpha）时 Auto 输出规范化为 HSL；有 NaN 时保留 hwb() 格式
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
                let l_clean = clean_pct(l);
                let a_clean = clean_num(a);
                let b_clean = clean_num(b);
                match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                    true => write!(
                        f,
                        "lab({}% {} {})",
                        format_num(l_clean),
                        format_num(a_clean),
                        format_num(b_clean)
                    ),
                    false => write!(
                        f,
                        "lab({}% {} {} / {})",
                        format_num(l_clean),
                        format_num(a_clean),
                        format_num(b_clean),
                        format_alpha(c.a)
                    ),
                }
            }
            ColorSpace::Lch => {
                let (l, ch, h) = (c.channels[0], c.channels[1], c.channels[2]);
                let l_clean = clean_pct(l);
                let ch_clean = clean_num(ch);
                // CSS Color 4: chroma=0 或 hue=NaN 时输出 none
                let h_str = match ch_clean == 0.0 || h.is_nan() {
                    true => "none".to_string(),
                    false => format!("{}{}", format_hue(h), DEG_UNIT),
                };
                match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                    true => write!(
                        f,
                        "lch({}% {} {})",
                        format_num(l_clean),
                        format_num(ch_clean),
                        h_str
                    ),
                    false => write!(
                        f,
                        "lch({}% {} {} / {})",
                        format_num(l_clean),
                        format_num(ch_clean),
                        h_str,
                        format_alpha(c.a)
                    ),
                }
            }
            ColorSpace::Oklab => {
                let (l, a, b) = (c.channels[0], c.channels[1], c.channels[2]);
                let l_pct = clean_pct(l * PCT_SCALE);
                let a_clean = clean_num(a);
                let b_clean = clean_num(b);
                match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                    true => write!(
                        f,
                        "oklab({}% {} {})",
                        format_num(l_pct),
                        format_num(a_clean),
                        format_num(b_clean)
                    ),
                    false => write!(
                        f,
                        "oklab({}% {} {} / {})",
                        format_num(l_pct),
                        format_num(a_clean),
                        format_num(b_clean),
                        format_alpha(c.a)
                    ),
                }
            }
            ColorSpace::Oklch => {
                let (l, ch, h) = (c.channels[0], c.channels[1], c.channels[2]);
                let l_pct = clean_pct(l * PCT_SCALE);
                let ch_clean = clean_num(ch);
                // CSS Color 4: chroma=0 或 hue=NaN 时输出 none
                let h_str = match ch_clean == 0.0 || h.is_nan() {
                    true => "none".to_string(),
                    false => format!("{}{}", format_hue(h), DEG_UNIT),
                };
                match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                    true => write!(
                        f,
                        "oklch({}% {} {})",
                        format_num(l_pct),
                        format_num(ch_clean),
                        h_str
                    ),
                    false => write!(
                        f,
                        "oklch({}% {} {} / {})",
                        format_num(l_pct),
                        format_num(ch_clean),
                        h_str,
                        format_alpha(c.a)
                    ),
                }
            }
            ColorSpace::DisplayP3 => {
                let (r, g, b) = (c.channels[0], c.channels[1], c.channels[2]);
                match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                    true => write!(
                        f,
                        "color(display-p3 {} {} {})",
                        format_num(r),
                        format_num(g),
                        format_num(b)
                    ),
                    false => write!(
                        f,
                        "color(display-p3 {} {} {} / {})",
                        format_num(r),
                        format_num(g),
                        format_num(b),
                        format_alpha(c.a)
                    ),
                }
            }
            ColorSpace::DisplayP3Linear => {
                let (r, g, b) = (c.channels[0], c.channels[1], c.channels[2]);
                match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                    true => write!(
                        f,
                        "color(display-p3-linear {} {} {})",
                        format_num(r),
                        format_num(g),
                        format_num(b)
                    ),
                    false => write!(
                        f,
                        "color(display-p3-linear {} {} {} / {})",
                        format_num(r),
                        format_num(g),
                        format_num(b),
                        format_alpha(c.a)
                    ),
                }
            }
            ColorSpace::Srgb => {
                let (r, g, b) = (c.channels[0], c.channels[1], c.channels[2]);
                match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                    true => write!(
                        f,
                        "color(srgb {} {} {})",
                        format_num(r),
                        format_num(g),
                        format_num(b)
                    ),
                    false => write!(
                        f,
                        "color(srgb {} {} {} / {})",
                        format_num(r),
                        format_num(g),
                        format_num(b),
                        format_alpha(c.a)
                    ),
                }
            }
            ColorSpace::SrgbLinear => {
                let (r, g, b) = (c.channels[0], c.channels[1], c.channels[2]);
                match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                    true => write!(
                        f,
                        "color(srgb-linear {} {} {})",
                        format_num(r),
                        format_num(g),
                        format_num(b)
                    ),
                    false => write!(
                        f,
                        "color(srgb-linear {} {} {} / {})",
                        format_num(r),
                        format_num(g),
                        format_num(b),
                        format_alpha(c.a)
                    ),
                }
            }
            ColorSpace::A98Rgb => {
                let (r, g, b) = (c.channels[0], c.channels[1], c.channels[2]);
                match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                    true => write!(
                        f,
                        "color(a98-rgb {} {} {})",
                        format_num(r),
                        format_num(g),
                        format_num(b)
                    ),
                    false => write!(
                        f,
                        "color(a98-rgb {} {} {} / {})",
                        format_num(r),
                        format_num(g),
                        format_num(b),
                        format_alpha(c.a)
                    ),
                }
            }
            ColorSpace::ProphotoRgb => {
                let (r, g, b) = (c.channels[0], c.channels[1], c.channels[2]);
                match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                    true => write!(
                        f,
                        "color(prophoto-rgb {} {} {})",
                        format_num(r),
                        format_num(g),
                        format_num(b)
                    ),
                    false => write!(
                        f,
                        "color(prophoto-rgb {} {} {} / {})",
                        format_num(r),
                        format_num(g),
                        format_num(b),
                        format_alpha(c.a)
                    ),
                }
            }
            ColorSpace::Rec2020 => {
                let (r, g, b) = (c.channels[0], c.channels[1], c.channels[2]);
                match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                    true => write!(
                        f,
                        "color(rec2020 {} {} {})",
                        format_num(r),
                        format_num(g),
                        format_num(b)
                    ),
                    false => write!(
                        f,
                        "color(rec2020 {} {} {} / {})",
                        format_num(r),
                        format_num(g),
                        format_num(b),
                        format_alpha(c.a)
                    ),
                }
            }
            ColorSpace::XyzD65 => {
                let (x, y, z) = (c.channels[0], c.channels[1], c.channels[2]);
                match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                    true => write!(
                        f,
                        "color(xyz {} {} {})",
                        format_num(x),
                        format_num(y),
                        format_num(z)
                    ),
                    false => write!(
                        f,
                        "color(xyz {} {} {} / {})",
                        format_num(x),
                        format_num(y),
                        format_num(z),
                        format_alpha(c.a)
                    ),
                }
            }
            ColorSpace::XyzD50 => {
                let (x, y, z) = (c.channels[0], c.channels[1], c.channels[2]);
                match (c.a - 1.0).abs() < ALPHA_TOLERANCE {
                    true => write!(
                        f,
                        "color(xyz-d50 {} {} {})",
                        format_num(x),
                        format_num(y),
                        format_num(z)
                    ),
                    false => write!(
                        f,
                        "color(xyz-d50 {} {} {} / {})",
                        format_num(x),
                        format_num(y),
                        format_num(z),
                        format_alpha(c.a)
                    ),
                }
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
        },
    }
}
