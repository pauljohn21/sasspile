#![allow(
    clippy::many_single_char_names,
    clippy::single_char_pattern,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
//! Value::Color 的 Display 实现入口。
//!
//! 从 display.rs 提取以避免单文件超限。
//! Auto 模式按色彩空间的序列化已迁移到 `display_color_spaces.rs`。

use super::*;
use crate::consts::{ALPHA_TOLERANCE, FLOAT_NOISE_THRESHOLD};

/// 格式化 RGB 百分比输出（alpha 判断 + 命名色 + hex + percent/rgba 格式分派）。
fn fmt_rgb_percent_output(
    c: &Color,
    rp: f64,
    gp: f64,
    bp: f64,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    // legacy_rgb 整数检查——基于 0-255 尺度，避免百分比尺度精度损失
    let legacy_int = (c.legacy_rgb[0] - c.legacy_rgb[0].round()).abs() < FLOAT_NOISE_THRESHOLD
        && (c.legacy_rgb[1] - c.legacy_rgb[1].round()).abs() < FLOAT_NOISE_THRESHOLD
        && (c.legacy_rgb[2] - c.legacy_rgb[2].round()).abs() < FLOAT_NOISE_THRESHOLD;
    let rp_r = c.legacy_rgb[0].round() as u8;
    let gp_r = c.legacy_rgb[1].round() as u8;
    let bp_r = c.legacy_rgb[2].round() as u8;
    let alpha_ok = (c.a - 1.0).abs() < ALPHA_TOLERANCE;
    let named = crate::eval::Evaluator::reverse_lookup_named_color(c);
    match (alpha_ok, named) {
        (true, Some(name)) => write!(f, "{name}"),
        (true, None) => match legacy_int {
            true => write!(f, "#{rp_r:02x}{gp_r:02x}{bp_r:02x}"),
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
                        format!("{rounded}")
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
            fmt_rgb_percent_output(c, rp, gp, bp, f)
        }
        ColorOutput::Auto => super::display_color_spaces::fmt_color_auto(c, f),
    }
}
