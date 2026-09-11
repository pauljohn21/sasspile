#![allow(
    clippy::many_single_char_names,
    clippy::single_char_pattern,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
//! `color.scale` 实现——按比例缩放颜色通道。

use crate::error::Result;
use crate::eval::Evaluator;
use crate::parse::ast::{Color, ColorOutput, ColorSpace, Value};
use imbl::HashMap;

use super::color_adjust::{
    build_channel_modified_color, build_legacy_color, classify_modified, extract_color,
    scale_channel, ModifiedSpace,
};

/// `color.scale($color, $kwargs)` — 按比例缩放颜色通道。
pub fn scale_color(args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Value> {
    let c = extract_color(args, kw_args)?;

    match c.space {
        ColorSpace::Oklch => super::color_adjust_cie::scale_oklch(c, kw_args),
        ColorSpace::Oklab => super::color_adjust_cie::scale_oklab(c, kw_args),
        ColorSpace::Lch => super::color_adjust_cie::scale_lch(c, kw_args),
        ColorSpace::Lab => super::color_adjust_cie::scale_lab(c, kw_args),
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

fn scale_modern_rgb_space(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    // 现代 RGB 空间：线性插值 val + (max - val)*pct，max=1.0
    let r = scale_channel(c.channels[0], 1.0, kw_args, "red");
    let g = scale_channel(c.channels[1], 1.0, kw_args, "green");
    let b = scale_channel(c.channels[2], 1.0, kw_args, "blue");
    let a = scale_channel(c.a, 1.0, kw_args, "alpha").clamp(0.0, 1.0);

    Ok(Value::Color(Color::with_space(
        c.space, [r, g, b], a, c.output, c.legacy_rgb,
    )))
}

fn scale_legacy(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let modified = classify_modified(kw_args);

    // RGB 通道缩放
    let r = scale_channel(c.legacy_rgb[0], 255.0, kw_args, "red");
    let g = scale_channel(c.legacy_rgb[1], 255.0, kw_args, "green");
    let b = scale_channel(c.legacy_rgb[2], 255.0, kw_args, "blue");
    let alpha = scale_channel(c.a, 1.0, kw_args, "alpha").clamp(0.0, 1.0);

    let rgb_result = match modified {
        ModifiedSpace::Hwb => {
            let (h, w_init, bk_init) =
                Evaluator::rgb_to_hwb(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
            let w = scale_channel(w_init, 1.0, kw_args, "whiteness");
            let bk = scale_channel(bk_init, 1.0, kw_args, "blackness");
            let (w_n, bk_n) = match w + bk > 1.0 {
                true => (w / (w + bk), bk / (w + bk)),
                false => (w, bk),
            };
            let (nr, ng, nb, _) = super::color_adjust::hwb_to_rgb_channels(h, w_n, bk_n, alpha);
            (nr, ng, nb)
        }
        ModifiedSpace::Hsl => {
            let (h_init, s_init, l_init) = match c.space == ColorSpace::Hsl {
                true => (c.channels[0], c.channels[1], c.channels[2]),
                false => {
                    let (h, s, l) = Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
                    (h, s, l)
                }
            };
            let h = scale_channel(h_init, 360.0, kw_args, "hue");
            let s = scale_channel(s_init, 1.0, kw_args, "saturation").clamp(0.0, 1.0);
            let l = scale_channel(l_init, 1.0, kw_args, "lightness").clamp(0.0, 1.0);
            if c.space == ColorSpace::Hsl {
                return Ok(Value::Color(build_channel_modified_color(
                    c.space, h, s, l, alpha,
                )));
            }
            let (nr, ng, nb) = super::color_adjust::hsl_to_rgb_channels(h, s, l);
            (nr, ng, nb)
        }
        ModifiedSpace::Rgb => (r, g, b),
    };

    // scale-color HSL/HWB-modified: 总是以 rgb(r%, g%, b%) 格式输出
    // scale-color RGB-modified: hex/rgb 格式（与 change-color 相同）
    let clamp_rgb = |v: f64| v.clamp(0.0, 255.0);
    let color = match (modified, c.space) {
        (ModifiedSpace::Hsl | ModifiedSpace::Hwb, _) => {
            let has_nan = rgb_result.0.is_nan() || rgb_result.1.is_nan() || rgb_result.2.is_nan() || alpha.is_nan();
            match has_nan {
                true => Color::with_hsl(
                    rgb_result.0, rgb_result.1, rgb_result.2, alpha,
                    ColorOutput::Auto, [0.0; 3],
                ),
                false => Color::with_rgb(
                    clamp_rgb(rgb_result.0),
                    clamp_rgb(rgb_result.1),
                    clamp_rgb(rgb_result.2),
                    alpha,
                    ColorSpace::Rgb,
                    ColorOutput::RgbPercent,
                ),
            }
        }
        _ => build_legacy_color(
            modified,
            c.space,
            clamp_rgb(rgb_result.0),
            clamp_rgb(rgb_result.1),
            clamp_rgb(rgb_result.2),
            alpha,
        ),
    };
    Ok(Value::Color(color))
}
