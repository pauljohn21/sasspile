#![allow(
    clippy::many_single_char_names,
    clippy::single_char_pattern,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
//! `color.change` 实现——设置颜色通道（绝对值）。

use crate::error::Result;
use crate::eval::Evaluator;
use crate::parse::ast::{Color, ColorOutput, ColorSpace, Value};
use imbl::HashMap;

use super::color_adjust::{
    apply_channel, build_channel_modified_color, build_legacy_color, classify_modified,
    extract_color, modern_channel_keys, alpha_value, angle_deg, cie_channel, percentage,
    raw_value, ModifiedSpace,
};

/// `color.change($color, $kwargs)` — 设置颜色通道（绝对值）。
pub fn change_color(args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Value> {
    let c = extract_color(args, kw_args)?;

    match c.space {
        ColorSpace::Oklch => super::color_adjust_cie::change_oklch(c, kw_args),
        ColorSpace::Oklab => super::color_adjust_cie::change_oklab(c, kw_args),
        ColorSpace::Lch => super::color_adjust_cie::change_lch(c, kw_args),
        ColorSpace::Lab => super::color_adjust_cie::change_lab(c, kw_args),
        ColorSpace::DisplayP3
        | ColorSpace::Srgb
        | ColorSpace::SrgbLinear
        | ColorSpace::DisplayP3Linear
        | ColorSpace::A98Rgb
        | ColorSpace::ProphotoRgb
        | ColorSpace::Rec2020
        | ColorSpace::XyzD65
        | ColorSpace::XyzD50 => change_modern_rgb_space(c, kw_args),
        _ => change_legacy(c, kw_args),
    }
}

fn change_modern_rgb_space(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let [c0, c1, c2] = modern_channel_keys(c.space);
    let channels: Vec<f64> = [c0, c1, c2]
        .iter()
        .zip(c.channels.iter())
        .map(|(key, init)| cie_channel(kw_args, key, 1.0).unwrap_or(*init))
        .collect();
    let a = apply_channel(c.a, kw_args, "alpha", alpha_value, |_v, d| d.clamp(0.0, 1.0));

    Ok(Value::Color(Color::with_space(
        c.space,
        [channels[0], channels[1], channels[2]],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

fn change_legacy(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let modified = classify_modified(kw_args);

    match modified {
        ModifiedSpace::Hwb => change_hwb(c, kw_args),
        ModifiedSpace::Hsl => change_hsl(c, kw_args),
        ModifiedSpace::Rgb => change_rgb_legacy(c, kw_args),
    }
}

fn change_rgb_legacy(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    // RGB 通道设置（change 不 clamp——规范是绝对赋值，允许超出范围）
    let r = apply_channel(c.legacy_rgb[0], kw_args, "red", raw_value, |_v, d| d);
    let g = apply_channel(c.legacy_rgb[1], kw_args, "green", raw_value, |_v, d| d);
    let b = apply_channel(c.legacy_rgb[2], kw_args, "blue", raw_value, |_v, d| d);
    let alpha = apply_channel(c.a, kw_args, "alpha", alpha_value, |_v, d| {
        if d.is_nan() { d } else { d.clamp(0.0, 1.0) }
    });
    let color = build_legacy_color(
        ModifiedSpace::Rgb,
        c.space,
        r, g, b,
        alpha,
    );
    Ok(Value::Color(color))
}

fn change_hwb(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let alpha = apply_channel(c.a, kw_args, "alpha", alpha_value, |_v, d| {
        if d.is_nan() { d } else { d.clamp(0.0, 1.0) }
    });
    let (h_init, hw_init, hb_init) =
        Evaluator::rgb_to_hwb(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
    let h = apply_channel(h_init, kw_args, "hue", angle_deg, |_v, d| d.rem_euclid(360.0));
    let hw = apply_channel(hw_init, kw_args, "whiteness", percentage, |_v, d| d);
    let hb = apply_channel(hb_init, kw_args, "blackness", percentage, |_v, d| d);
    // HWB 归一化：hw + hb > 1 时按比例缩放
    let (hw_n, hb_n) = normalize_hwb_sum(hw, hb);
    // 序列化规则：有 NaN → HWB(none) 格式；RGB 有效值 [0,255] → RGB 输出；超范围 → HWB 格式
    let has_nan = h.is_nan() || hw.is_nan() || hb.is_nan();
    let (nr_c, ng_c, nb_c) = hwb_to_clean_rgb(h, hw_n, hb_n);
    let rgb_in_range = !has_nan
        && (0.0..=255.0).contains(&nr_c)
        && (0.0..=255.0).contains(&ng_c)
        && (0.0..=255.0).contains(&nb_c);
    match rgb_in_range {
        true => Ok(Value::Color(Color::with_space(
            ColorSpace::Hwb,
            [h, hw_n, hb_n],
            alpha,
            ColorOutput::RgbPercent,
            [nr_c, ng_c, nb_c],
        ))),
        false => Ok(Value::Color(build_channel_modified_color(
            ColorSpace::Hwb, h, hw_n, hb_n, alpha,
        ))),
    }
}

fn normalize_hwb_sum(hw: f64, hb: f64) -> (f64, f64) {
    let sum = hw + hb;
    match sum.is_nan() || sum.is_infinite() {
        true => (hw, hb),
        false => match sum > 1.0 {
            true => (hw / sum, hb / sum),
            false => (hw, hb),
        },
    }
}

fn hwb_to_clean_rgb(h: f64, hw: f64, hb: f64) -> (f64, f64, f64) {
    let clean_ch = |v: f64| if v.abs() < 1e-6 { 0.0 } else { v };
    let (nr, ng, nb, _) = super::color_adjust::hwb_to_rgb_channels(h, hw, hb, 1.0);
    (clean_ch(nr), clean_ch(ng), clean_ch(nb))
}

fn change_hsl(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let alpha = apply_channel(c.a, kw_args, "alpha", alpha_value, |_v, d| {
        if d.is_nan() { d } else { d.clamp(0.0, 1.0) }
    });
    // 获取 HSL 初始通道：从已有 HSL 数据或从 RGB 推导
    let (h_init, s_init, l_init) = match c.space == ColorSpace::Hsl {
        true => (c.channels[0], c.channels[1], c.channels[2]),
        false => {
            Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2])
        }
    };
    let h = apply_channel(h_init, kw_args, "hue", angle_deg, |_v, d| d.rem_euclid(360.0));
    // change 不 clamp——允许超出范围值
    let s = apply_channel(s_init, kw_args, "saturation", percentage, |_v, d| d);
    let l = apply_channel(l_init, kw_args, "lightness", percentage, |_v, d| d);
    // 清理浮点噪声：极小值归零（避免 -1e-15 导致 rgb_valid=false）
    let clean_rgb_ch = |v: f64| if v.abs() < 1e-6 { 0.0 } else { v };
    // 序列化规则：有 NaN → HSL(none)；RGB 有效值 [0,255] → RGB 输出（hex/named/rgb%）；超范围 → HSL 格式
    let rgb = super::color_adjust::hsl_to_rgb_channels(h, s, l);
    let rgb_clean = (clean_rgb_ch(rgb.0), clean_rgb_ch(rgb.1), clean_rgb_ch(rgb.2));
    let rgb_in_range = rgb_clean.0 >= 0.0 && rgb_clean.0 <= 255.0
        && rgb_clean.1 >= 0.0 && rgb_clean.1 <= 255.0
        && rgb_clean.2 >= 0.0 && rgb_clean.2 <= 255.0;
    match (h.is_nan() || s.is_nan() || l.is_nan(), rgb_in_range) {
        (true, _) | (false, false) => Ok(Value::Color(build_channel_modified_color(
            ColorSpace::Hsl, h, s, l, alpha,
        ))),
        (false, true) => Ok(Value::Color(Color::with_hsl(
            h, s, l, alpha, ColorOutput::RgbPercent, [rgb_clean.0, rgb_clean.1, rgb_clean.2],
        ))),
    }
}
