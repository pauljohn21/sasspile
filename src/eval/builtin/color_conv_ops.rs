#![allow(
    clippy::many_single_char_names,
    clippy::single_char_pattern,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
//! 颜色空间转换工具函数。
//!
//! 包含 `ColorSpace` 空间判断、空间转换、空间→sRGB 转换、
//! HSL/HWB f64 `精度转换、make_color` 等。

use crate::consts::RGB_MAX;
use crate::error::{Result, SassError};
use crate::parse::ast::{Color, ColorOutput, ColorSpace, Value};

use super::super::Evaluator;

/// 判断 `ColorSpace` 是否与目标空间名称匹配。
pub(crate) fn is_same_space(space: ColorSpace, target: &str) -> bool {
    space.as_str() == target
        || (space == ColorSpace::XyzD65 && (target == "xyz" || target == "xyz-d65"))
}

/// 转换颜色到目标空间，用 f64 精度算法。
/// CSS Color 4 规范：NaN 通道在计算中取 0，输出时恢复为 NaN。
pub(crate) fn convert_space(c: &Color, target_space: &str) -> Result<Value> {
    use super::color_conv;

    // 同空间转换——直接返回原始值，避免精度损失
    // 创建函数的 legacy 规范化由 display.rs 的 Auto 输出处理
    match is_same_space(c.space, target_space) {
        true => return Ok(Value::Color(c.clone())),
        false => {}
    }

    // 获取源 sRGB (0-1) 值，记录 NaN 位置后替换为 0 进行计算
    let (r, g, b) = space_to_srgb_f64(c.space, c.channels, c.legacy_rgb);
    let nan_mask = [r.is_nan(), g.is_nan(), b.is_nan()];
    // HWB 的 hue 为 NaN 时，目标色彩空间的 hue 通道也应为 NaN
    let src_hwb_nan_hue = c.space == ColorSpace::Hwb && c.channels[0].is_nan();
    let r = match nan_mask[0] {
        true => 0.0,
        false => r,
    };
    let g = match nan_mask[1] {
        true => 0.0,
        false => g,
    };
    let b = match nan_mask[2] {
        true => 0.0,
        false => b,
    };

    // NaN 恢复：输出通道中，输入为 NaN 的位置重新设为 NaN
    let restore_nan = |vals: [f64; 3]| -> [f64; 3] {
        [
            match nan_mask[0] {
                true => f64::NAN,
                false => vals[0],
            },
            match nan_mask[1] {
                true => f64::NAN,
                false => vals[1],
            },
            match nan_mask[2] {
                true => f64::NAN,
                false => vals[2],
            },
        ]
    };

    match target_space {
        "rgb" => {
            let r255 = c.legacy_rgb[0];
            let g255 = c.legacy_rgb[1];
            let b255 = c.legacy_rgb[2];
            Ok(Value::Color(Color::with_rgb(
                r255,
                g255,
                b255,
                c.a,
                ColorSpace::Rgb,
                ColorOutput::Auto,
            )))
        }
        "srgb" => Ok(make_color(
            ColorSpace::Srgb,
            [r, g, b],
            c.a,
            ColorOutput::Auto,
        )),
        "srgb-linear" => {
            let (rl, gl, bl) = color_conv::srgb_to_linear_srgb(r, g, b);
            Ok(make_color(
                ColorSpace::SrgbLinear,
                restore_nan([rl, gl, bl]),
                c.a,
                ColorOutput::Auto,
            ))
        }
        "hsl" => {
            let (h, s, l) = match c.space {
                ColorSpace::Hsl => (c.channels[0], c.channels[1], c.channels[2]),
                ColorSpace::Hwb => {
                    hwb_to_hsl_via_color(c.channels[0], c.channels[1], c.channels[2])
                }
                _ => Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]),
            };
            Ok(Value::Color(Color::with_hsl(
                h,
                s,
                l,
                c.a,
                ColorOutput::Auto,
                c.legacy_rgb,
            )))
        }
        "hwb" => {
            let (h, w, bk) = if c.space == ColorSpace::Hwb {
                (c.channels[0], c.channels[1], c.channels[2])
            } else {
                let (r_f, g_f, b_f) = space_to_srgb_f64(c.space, c.channels, c.legacy_rgb);
                let nan_r = r_f.is_nan();
                let nan_g = g_f.is_nan();
                let nan_b = b_f.is_nan();
                let r_f = match nan_r {
                    true => 0.0,
                    false => r_f,
                };
                let g_f = match nan_g {
                    true => 0.0,
                    false => g_f,
                };
                let b_f = match nan_b {
                    true => 0.0,
                    false => b_f,
                };
                let (h, _s, _l) =
                    Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
                let w = r_f.min(g_f).min(b_f);
                let bk = 1.0 - r_f.max(g_f).max(b_f);
                let (w, bk) = match nan_r || nan_g || nan_b {
                    true => (f64::NAN, f64::NAN),
                    false => (w, bk),
                };
                (h, w, bk)
            };
            Ok(Value::Color(Color::with_hwb(h, w, bk, c.a, c.legacy_rgb)))
        }
        "lab" => {
            let (l, a, b_lab) = color_conv::srgb_to_lab(r, g, b);
            Ok(make_color(
                ColorSpace::Lab,
                restore_nan([l, a, b_lab]),
                c.a,
                ColorOutput::Auto,
            ))
        }
        "lch" => {
            let (l, c_lch, h) = color_conv::srgb_to_lch(r, g, b);
            // CSS Color 4: 源 HWB hue 为 NaN 时，目标 LCH hue 也为 NaN
            // 注意：LCH 的 hue 独立于 sRGB 通道，不使用 restore_nan
            let h = match src_hwb_nan_hue {
                true => f64::NAN,
                false => h,
            };
            Ok(make_color(
                ColorSpace::Lch,
                [
                    match nan_mask[0] {
                        true => f64::NAN,
                        false => l,
                    },
                    match nan_mask[1] {
                        true => f64::NAN,
                        false => c_lch,
                    },
                    h,
                ],
                c.a,
                ColorOutput::Auto,
            ))
        }
        "oklab" => {
            let (l, a, b_ok) = color_conv::srgb_to_oklab(r, g, b);
            Ok(make_color(
                ColorSpace::Oklab,
                restore_nan([l, a, b_ok]),
                c.a,
                ColorOutput::Auto,
            ))
        }
        "oklch" => {
            let (l, c_ok, h) = color_conv::srgb_to_oklch(r, g, b);
            // CSS Color 4: 源 HWB hue 为 NaN 时，目标 OKLCH hue 也为 NaN
            // 注意：OKLCH 的 hue 独立于 sRGB 通道，不使用 restore_nan
            let h = match src_hwb_nan_hue {
                true => f64::NAN,
                false => h,
            };
            Ok(make_color(
                ColorSpace::Oklch,
                [
                    match nan_mask[0] {
                        true => f64::NAN,
                        false => l,
                    },
                    match nan_mask[1] {
                        true => f64::NAN,
                        false => c_ok,
                    },
                    h,
                ],
                c.a,
                ColorOutput::Auto,
            ))
        }
        "display-p3" => {
            let (rp, gp, bp) = color_conv::srgb_to_display_p3(r, g, b);
            Ok(make_color(
                ColorSpace::DisplayP3,
                restore_nan([rp, gp, bp]),
                c.a,
                ColorOutput::Auto,
            ))
        }
        "display-p3-linear" => {
            let (rl, gl, bl) = color_conv::srgb_to_linear_srgb(r, g, b);
            Ok(make_color(
                ColorSpace::DisplayP3Linear,
                restore_nan([rl, gl, bl]),
                c.a,
                ColorOutput::Auto,
            ))
        }
        "a98-rgb" => {
            let (rp, gp, bp) = color_conv::srgb_to_a98_rgb(r, g, b);
            Ok(make_color(
                ColorSpace::A98Rgb,
                restore_nan([rp, gp, bp]),
                c.a,
                ColorOutput::Auto,
            ))
        }
        "prophoto-rgb" => {
            let (rp, gp, bp) = color_conv::srgb_to_prophoto(r, g, b);
            Ok(make_color(
                ColorSpace::ProphotoRgb,
                restore_nan([rp, gp, bp]),
                c.a,
                ColorOutput::Auto,
            ))
        }
        "rec2020" => {
            let (rp, gp, bp) = color_conv::srgb_to_rec2020(r, g, b);
            Ok(make_color(
                ColorSpace::Rec2020,
                restore_nan([rp, gp, bp]),
                c.a,
                ColorOutput::Auto,
            ))
        }
        "xyz" | "xyz-d65" => {
            let (x, y, z) = color_conv::srgb_to_xyz_d65(r, g, b);
            Ok(make_color(
                ColorSpace::XyzD65,
                restore_nan([x, y, z]),
                c.a,
                ColorOutput::Auto,
            ))
        }
        "xyz-d50" => {
            let (x_d65, y_d65, z_d65) = color_conv::srgb_to_xyz_d65(r, g, b);
            let (x, y, z) = color_conv::xyz_d65_to_xyz_d50(x_d65, y_d65, z_d65);
            Ok(make_color(
                ColorSpace::XyzD50,
                restore_nan([x, y, z]),
                c.a,
                ColorOutput::Auto,
            ))
        }
        _ => Err(SassError::Eval(format!(
            "Unknown color space: {target_space}"
        ))),
    }
}

/// 从 `ColorSpace` + channels 获取 sRGB (0-1) f64 值。
pub(crate) fn space_to_srgb_f64(
    space: ColorSpace,
    channels: [f64; 3],
    legacy_rgb: [f64; 3],
) -> (f64, f64, f64) {
    use super::color_conv;
    match space {
        ColorSpace::Rgb => (
            legacy_rgb[0] / RGB_MAX,
            legacy_rgb[1] / RGB_MAX,
            legacy_rgb[2] / RGB_MAX,
        ),
        ColorSpace::Srgb => (channels[0], channels[1], channels[2]),
        ColorSpace::SrgbLinear => {
            color_conv::linear_srgb_to_srgb(channels[0], channels[1], channels[2])
        }
        ColorSpace::DisplayP3 => {
            color_conv::display_p3_to_srgb(channels[0], channels[1], channels[2])
        }
        ColorSpace::DisplayP3Linear => {
            color_conv::linear_srgb_to_srgb(channels[0], channels[1], channels[2])
        }
        ColorSpace::A98Rgb => color_conv::a98_rgb_to_srgb(channels[0], channels[1], channels[2]),
        ColorSpace::ProphotoRgb => {
            color_conv::prophoto_to_srgb(channels[0], channels[1], channels[2])
        }
        ColorSpace::Rec2020 => color_conv::rec2020_to_srgb(channels[0], channels[1], channels[2]),
        ColorSpace::XyzD65 => color_conv::xyz_d65_to_srgb(channels[0], channels[1], channels[2]),
        ColorSpace::XyzD50 => {
            let (x_d65, y_d65, z_d65) =
                color_conv::xyz_d50_to_xyz_d65(channels[0], channels[1], channels[2]);
            color_conv::xyz_d65_to_srgb(x_d65, y_d65, z_d65)
        }
        ColorSpace::Hsl => hsl_to_srgb_f64(channels[0], channels[1], channels[2]),
        ColorSpace::Hwb => hwb_to_srgb_f64(channels[0], channels[1], channels[2]),
        ColorSpace::Lab => color_conv::lab_to_srgb(channels[0], channels[1], channels[2]),
        ColorSpace::Lch => color_conv::lch_to_srgb(channels[0], channels[1], channels[2]),
        ColorSpace::Oklab => color_conv::oklab_to_srgb(channels[0], channels[1], channels[2]),
        ColorSpace::Oklch => color_conv::oklch_to_srgb(channels[0], channels[1], channels[2]),
    }
}

/// HWB→HSL 转换，直接在 f64 上计算，避免 u8 量化精度损失。
/// CSS Color 4: NaN 通道在计算中取 0，hue 为 NaN 时基于 w/b 推导 saturation/lightness。
pub(crate) fn hwb_to_hsl_via_color(h: f64, w: f64, b: f64) -> (f64, f64, f64) {
    // w/b 为 NaN 时替换为 0 进行计算（CSS Color 4 missing 通道规范）
    let w = match w.is_nan() {
        true => 0.0,
        false => w,
    };
    let b = match b.is_nan() {
        true => 0.0,
        false => b,
    };
    // hue 为 NaN 时，HSL 的 saturation/lightness 基于 w/b 推导
    match h.is_nan() {
        true => {
            let l = 0.5 * (1.0 - b + w);
            let max = 1.0 - b;
            let min = w;
            let delta = (max - min).abs();
            let s = match delta < 1e-12 {
                true => 0.0,
                false => delta / (1.0 - (2.0 * l - 1.0).abs()),
            };
            (f64::NAN, s, l)
        }
        false => {
            let h_norm = h.rem_euclid(360.0) / 360.0;
            let (w, b) = if w + b > 1.0 {
                (w / (w + b), b / (w + b))
            } else {
                (w, b)
            };
            let factor = 1.0 - w - b;

            let hue_to_rgb = |hue: f64| -> f64 {
                let mut hue = hue;
                match hue < 0.0 {
                    true => hue += 1.0,
                    false => {}
                }
                match hue > 1.0 {
                    true => hue -= 1.0,
                    false => {}
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
    }
}

/// HSL → sRGB (0-1) f64 精度转换，不经过 u8 量化。
///
/// 使用 CSS Color 4 规范的 HSL→RGB 算法。
pub(crate) fn hsl_to_srgb_f64(h: f64, s: f64, l: f64) -> (f64, f64, f64) {
    let h = h.rem_euclid(360.0);
    match s == 0.0 {
        true => return (l, l, l),
        false => {}
    }
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
    (r1 + m, g1 + m, b1 + m)
}

/// HWB → sRGB (0-1) f64 精度转换，不经过 u8 量化。
///
/// 基于 CSS Color 4 规范的 HWB→RGB 算法。
/// NaN hue 时基于 w/bk 推导 saturation 和 lightness，hue=0 计算 RGB。
/// CSS Color 4: 所有 HWB 通道为 NaN 时，所有 sRGB 通道也为 NaN。
/// 单个通道为 NaN 时，替换为 0 计算（非全 NaN 不传播）。
pub(crate) fn hwb_to_srgb_f64(h: f64, w: f64, bk: f64) -> (f64, f64, f64) {
    let nan_h = h.is_nan();
    let nan_w = w.is_nan();
    let nan_bk = bk.is_nan();
    // 所有通道为 NaN 时，所有输出为 NaN
    let all_nan = nan_h && nan_w && nan_bk;
    // 单个通道为 NaN 时替换为 0 进行计算
    let h = match nan_h {
        true => 0.0,
        false => h,
    };
    let w = match nan_w {
        true => 0.0,
        false => w,
    };
    let bk = match nan_bk {
        true => 0.0,
        false => bk,
    };
    let (r, g, b) = match all_nan {
        true => (f64::NAN, f64::NAN, f64::NAN),
        false => {
            let factor = 1.0 - w - bk;
            let (hr, hg, hb) = hsl_to_srgb_f64(h, 1.0, 0.5);
            (hr * factor + w, hg * factor + w, hb * factor + w)
        }
    };
    (r, g, b)
}

/// 生成颜色的显示名称（用于错误消息）。
pub(crate) fn color_name(c: &Color) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        c.legacy_rgb[0].round() as u8,
        c.legacy_rgb[1].round() as u8,
        c.legacy_rgb[2].round() as u8
    )
}

/// 从空间和通道创建 Color（含 sRGB 近似值）。
pub(crate) fn make_color(
    space: ColorSpace,
    channels: [f64; 3],
    alpha: f64,
    output: ColorOutput,
) -> Value {
    let (r, g, b) = space_to_srgb_f64(space, channels, [0.0, 0.0, 0.0]);
    let r255 = (r * 255.0).clamp(0.0, 255.0);
    let g255 = (g * 255.0).clamp(0.0, 255.0);
    let b255 = (b * 255.0).clamp(0.0, 255.0);
    Value::Color(Color {
        space,
        channels,
        a: alpha,
        output,
        legacy_rgb: [r255, g255, b255],
    })
}
