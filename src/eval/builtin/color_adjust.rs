#![allow(
    clippy::many_single_char_names,
    clippy::single_char_pattern,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
//! `color.adjust` 实现 + change/scale 共享基础设施。
//!
//! 支持所有 CSS Color 4 颜色空间：
//! - Legacy: RGB, HSL, HWB
//! - Modern: Lab, Lch, Oklab, Oklch, `DisplayP3`, sRGB, sRGB-Linear, etc.
//!
//! `change-color` 和 `scale-color` 分别在 `color_change.rs` 和 `color_scale.rs` 中，
//! 共享本模块的 Extractor / apply_channel / scale_channel 等基础设施。

use crate::error::{Result, SassError};
use crate::eval::Evaluator;
use crate::parse::ast::{Color, ColorOutput, ColorSpace, Value};
use imbl::HashMap;

use super::color_parse::extract_calc_f64;

/// 提取 calc() 数值，但忽略 NaN（`calc(NaN)` 视为无操作）。
/// 仅用于 change 通道：`calc(±infinity)` → ±inf，其他 → None。
pub(super) fn extract_calc_f64_nonan(s: &str) -> Option<f64> {
    extract_calc_f64(s).filter(|v| !v.is_nan())
}

// ── Channel Extractor + 统一 apply ───────────────────────────────────────

/// 通道提取器：`(&HashMap, &str) -> Option<f64>`。封装「单位转换 + 关键字查找」。
pub(super) type Extractor = fn(&HashMap<String, Value>, &str) -> Option<f64>;

/// 字面数值：任何单位视为字面数值；`none` → NaN；`calc(±infinity)` → ±inf；未提供 → None。
pub(super) fn raw_value(kw: &HashMap<String, Value>, key: &str) -> Option<f64> {
    match kw.get(key) {
        Some(Value::Number(n, _)) => Some(*n),
        Some(Value::String(s, false)) if s == "none" => Some(f64::NAN),
        Some(Value::Calc(s)) => extract_calc_f64(s),
        _ => None,
    }
}

/// 百分比：任何数字视为 n/100；`none` → NaN。
/// ⚠️ 此提取器对所有数值（包括无单位）均除以 100。适用于 where 内部值以 0-1 表示 0%-100% 的场景（HSL/Oklch/Oklab 的 lightness）。
pub(super) fn percentage(kw: &HashMap<String, Value>, key: &str) -> Option<f64> {
    match kw.get(key) {
        Some(Value::Number(n, _)) => Some(*n / 100.0),
        Some(Value::String(s, false)) if s == "none" => Some(f64::NAN),
        Some(Value::Calc(s)) => extract_calc_f64_nonan(s),
        _ => None,
    }
}

/// alpha 通道提取器：50% → 0.5，无单位 n → n，`none` → NaN。
/// 仅在单位是 "%" 时才除以 100。
pub(super) fn alpha_value(kw: &HashMap<String, Value>, key: &str) -> Option<f64> {
    match kw.get(key) {
        Some(Value::Number(n, u)) => match u.as_deref() {
            Some("%") => Some(*n / 100.0),
            _ => Some(*n),
        },
        Some(Value::String(s, false)) if s == "none" => Some(f64::NAN),
        Some(Value::Calc(s)) => extract_calc_f64_nonan(s),
        _ => None,
    }
}

/// CIE 通道提取器——区分数值单位语义：
/// - 有单位 `%` → 解释为 channel max 的百分比：n/100 * max
/// - 无单位 n → 直接使用 n（值已在内部尺度，如 0-100 for Lab）
/// - `none` → NaN
/// - `calc(±infinity)` → ±inf，`calc(NaN)` → NaN
pub(super) fn cie_channel(kw: &HashMap<String, Value>, key: &str, max: f64) -> Option<f64> {
    match kw.get(key) {
        Some(Value::Number(n, u)) => match u.as_deref() {
            Some("%") => Some(*n / 100.0 * max),
            _ => Some(*n),
        },
        Some(Value::String(s, false)) if s == "none" => Some(f64::NAN),
        Some(Value::Calc(s)) => extract_calc_f64_nonan(s),
        _ => None,
    }
}

/// 通用链式应用：`extract(key).map(|d| f(init, d)).unwrap_or(init)`
pub(super) fn apply_channel(
    init: f64,
    kw: &HashMap<String, Value>,
    key: &str,
    extract: Extractor,
    f: impl Fn(f64, f64) -> f64,
) -> f64 {
    extract(kw, key).map(|d| f(init, d)).unwrap_or(init)
}

/// 缩放通道：百分比缩放（正最大值方向，负最小值方向）。
///
/// - `pct > 0`：在当前值和 `max` 之间线性插值
/// - `pct < 0`：在当前值和 `min` 之间线性插值
/// - `max == MAX`（chroma 类无下界通道）：仅按百分比缩放绝对值（`val + val * pct`）
pub(super) fn scale_channel(val: f64, max: f64, kw: &HashMap<String, Value>, key: &str) -> f64 {
    scale_channel_min(val, max, 0.0, kw, key)
}

/// 带 min 边界的缩放通道。
///
/// 超出 [min, max] 的 val：在远离边界方向上的缩放无效（sRGB out_of_gamut 语义）。
pub(super) fn scale_channel_min(
    val: f64,
    max: f64,
    min: f64,
    kw: &HashMap<String, Value>,
    key: &str,
) -> f64 {
    raw_value(kw, key)
    .map(|n| {
        let pct = n / 100.0;
        match (pct >= 0.0, (max - f64::MAX).abs() < f64::EPSILON) {
            // val 超上界 + 扩向上：无效（保持原值）
            (true, false) if val > max => val,
            // val 超下界 + 扩向下：无效（保持原值）
            (false, false) if val < min => val,
            // chroma 类无下界：乘法缩放
            (_, true) => val + val * pct,
            // 通用：向 max 插值 / 向 min 插值
            (true, _) => val + (max - val) * pct,
            (false, _) => val + (val - min) * pct,
        }
    })
    .unwrap_or(val)
}

/// 从 args/kw_args 提取颜色参数。
pub(super) fn extract_color<'a>(args: &'a [Value], kw_args: &'a HashMap<String, Value>) -> Result<&'a Color> {
    match args.first().or_else(|| kw_args.get("color")) {
        Some(Value::Color(c)) => Ok(c),
        Some(v) => Err(SassError::Eval(format!("$color: {v} is not a color."))),
        None => Err(SassError::Eval("Missing argument $color.".into())),
    }
}

/// 根据颜色空间的通道语义，将 X/Y/Z 关键词映射到 channels[0..2]。
pub(super) fn modern_channel_keys(space: ColorSpace) -> [&'static str; 3] {
    match space {
        ColorSpace::XyzD65 | ColorSpace::XyzD50 => ["x", "y", "z"],
        _ => ["red", "green", "blue"],
    }
}

// ── 入口函数 ─────────────────────────────────────────────────────────────

/// `color.adjust($color, $kwargs)` — 调整颜色通道（增量）。
pub fn adjust_color(args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Value> {
    let c = extract_color(args, kw_args)?;

    let modern_channels = ["lightness", "chroma", "a", "b", "x", "y", "z"];
    let has_modern = modern_channels.iter().any(|ch| kw_args.contains_key(*ch));

    match c.space {
        ColorSpace::Oklch => super::color_adjust_cie::adjust_oklch(c, kw_args),
        ColorSpace::Oklab => super::color_adjust_cie::adjust_oklab(c, kw_args),
        ColorSpace::Lch => super::color_adjust_cie::adjust_lch(c, kw_args),
        ColorSpace::Lab => super::color_adjust_cie::adjust_lab(c, kw_args),
        ColorSpace::DisplayP3
        | ColorSpace::Srgb
        | ColorSpace::SrgbLinear
        | ColorSpace::DisplayP3Linear
        | ColorSpace::A98Rgb
        | ColorSpace::ProphotoRgb
        | ColorSpace::Rec2020
        | ColorSpace::XyzD65
        | ColorSpace::XyzD50 => adjust_modern_rgb_space(c, kw_args),
        _ if has_modern => adjust_legacy(c, kw_args),
        _ => adjust_legacy(c, kw_args),
    }
}

// ── Modern RGB 空间 (DisplayP3, sRGB, A98, ProPhoto, Rec2020) ─────────────

fn adjust_modern_rgb_space(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let [c0, c1, c2] = modern_channel_keys(c.space);
    let channels: Vec<f64> = [c0, c1, c2]
        .iter()
        .zip(c.channels.iter())
        .map(|(key, init)| apply_cie_channel(*init, kw_args, key, 1.0, |v, d| v + d))
        .collect();
    let a = apply_channel(c.a, kw_args, "alpha", alpha_value, |v, d| (v + d).clamp(0.0, 1.0));

    Ok(Value::Color(Color::with_space(
        c.space,
        [channels[0], channels[1], channels[2]],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

// ── Legacy (RGB/HSL/HWB) ──────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ModifiedSpace { Rgb, Hsl, Hwb }

pub(super) fn classify_modified(kw: &HashMap<String, Value>) -> ModifiedSpace {
    if kw.contains_key("whiteness") || kw.contains_key("blackness") { return ModifiedSpace::Hwb; }
    if kw.contains_key("hue") || kw.contains_key("saturation") || kw.contains_key("lightness") { return ModifiedSpace::Hsl; }
    ModifiedSpace::Rgb
}

// 根据被修改的色彩空间构造 Color：Rgb→保原空间（超范围→HSL回退）, Hsl/Hwb→build_channel_modified_color
pub(super) fn build_legacy_color(
    space: ModifiedSpace,
    original: ColorSpace,
    r: f64,
    g: f64,
    b: f64,
    alpha: f64,
) -> Color {
    match space {
        ModifiedSpace::Rgb => match original {
            ColorSpace::Hsl => { let (h, s, l) = Evaluator::rgb_to_hsl(r, g, b); Color::with_hsl(h, s, l, alpha, ColorOutput::Auto, [r, g, b]) }
            _ => {
                // 超范围 RGB（超出 0-255，如 change-color(black, $red: 500)）→ 转 HSL 扩展输出
                let rgb_in_range = (0.0..=255.0).contains(&r) && (0.0..=255.0).contains(&g) && (0.0..=255.0).contains(&b);
                let has_nan = r.is_nan() || g.is_nan() || b.is_nan() || alpha.is_nan();
                match (has_nan, rgb_in_range) {
                    (true, _) => Color::with_rgb(r, g, b, alpha, ColorSpace::Rgb, ColorOutput::RgbModern),
                    (false, true) => Color::with_rgb(r, g, b, alpha, ColorSpace::Rgb, ColorOutput::Auto),
                    (false, false) => {
                        // 超范围：计算 HSL（允许 S > 1.0），以 HSL 格式输出
                        let (h, s, l) = Evaluator::rgb_to_hsl(r, g, b);
                        Color::with_hsl(h, s, l, alpha, ColorOutput::Auto, [r, g, b])
                    }
                }
            }
        },
        ModifiedSpace::Hsl | ModifiedSpace::Hwb => build_channel_modified_color(original, r, g, b, alpha),
    }
}

// 构造 HSL/HWB 修改后的 legacy 颜色：同构→保持, 异构+NaN→HSL推导, 异构+整→hex, 异构+分→percent
pub(super) fn build_channel_modified_color(original: ColorSpace, r: f64, g: f64, b: f64, alpha: f64) -> Color {
    let rgb_int = [r.round(), g.round(), b.round()];
    let frac = (r - rgb_int[0]).abs() > 1e-9 || (g - rgb_int[1]).abs() > 1e-9 || (b - rgb_int[2]).abs() > 1e-9;
    let nan = r.is_nan() || g.is_nan() || b.is_nan();
    match (original, nan, frac) {
        (ColorSpace::Hsl | ColorSpace::Hwb, _, _) => build_from_input_space(original, r, g, b, alpha, rgb_int),
        (_, true, _) => { let (h, s, l) = Evaluator::rgb_to_hsl(r, g, b); Color::with_hsl(h, s, l, alpha, ColorOutput::Auto, rgb_int) }
        (_, false, false) => Color::with_rgb(rgb_int[0], rgb_int[1], rgb_int[2], alpha, ColorSpace::Rgb, ColorOutput::Auto),
        (_, false, true) => { let (h, s, l) = Evaluator::rgb_to_hsl(r, g, b); Color::with_space(ColorSpace::Hsl, [h, s, l], alpha, ColorOutput::RgbPercent, rgb_int) }
    }
}

fn build_from_input_space(sp: ColorSpace, c0: f64, c1: f64, c2: f64, a: f64, rgb: [f64; 3]) -> Color {
    match sp {
        ColorSpace::Hsl => Color::with_hsl(c0, c1, c2, a, ColorOutput::Auto, rgb),
        ColorSpace::Hwb => Color::with_space(ColorSpace::Hwb, [c0, c1, c2], a, ColorOutput::Auto, rgb),
        _ => Color::with_rgb(rgb[0], rgb[1], rgb[2], a, ColorSpace::Rgb, ColorOutput::Auto),
    }
}

fn adjust_legacy(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let modified = classify_modified(kw_args);

    // RGB 通道调整
    let r = apply_channel(c.legacy_rgb[0], kw_args, "red", raw_value, |v, d| v + d);
    let g = apply_channel(c.legacy_rgb[1], kw_args, "green", raw_value, |v, d| v + d);
    let b = apply_channel(c.legacy_rgb[2], kw_args, "blue", raw_value, |v, d| v + d);
    let alpha = apply_channel(c.a, kw_args, "alpha", alpha_value, |v, d| v + d)
        .clamp(0.0, 1.0);

    // HSL/HWB 通道调整——仅在对应模式时计算
    let (r_out, g_out, b_out) = match modified {
        ModifiedSpace::Hwb => {
            let (h_init, hw_init, hb_init) =
                Evaluator::rgb_to_hwb(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
            let h = apply_channel(h_init, kw_args, "hue", angle_deg, |v, d| {
                (v + d).rem_euclid(360.0)
            });
            let hw = apply_channel(hw_init, kw_args, "whiteness", percentage, |v, d| {
                (v + d).clamp(0.0, 1.0)
            });
            let hb = apply_channel(hb_init, kw_args, "blackness", percentage, |v, d| {
                (v + d).clamp(0.0, 1.0)
            });
            let (nr, ng, nb, _) = hwb_to_rgb_channels(h, hw, hb, 1.0);
            (nr, ng, nb)
        }
        ModifiedSpace::Hsl => {
            let (h_init, s_init, l_init) = match c.space == ColorSpace::Hsl {
                true => (c.channels[0], c.channels[1], c.channels[2]),
                false => {
                    Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2])
                }
            };
            let h = apply_channel(h_init, kw_args, "hue", angle_deg, |v, d| {
                (v + d).rem_euclid(360.0)
            });
            let s = apply_channel(s_init, kw_args, "saturation", percentage, |v, d| {
                (v + d).clamp(0.0, 1.0)
            });
            let l = apply_channel(l_init, kw_args, "lightness", percentage, |v, d| {
                (v + d).clamp(0.0, 1.0)
            });
            // 同构 HSL 输入 → 直接保持 HSL 输出
            if c.space == ColorSpace::Hsl {
                return Ok(Value::Color(build_channel_modified_color(
                    c.space, h, s, l, alpha,
                )));
            }
            // 异构输入 → hsl→RGB 转换
            let (nr, ng, nb) = hsl_to_rgb_channels(h, s, l);
            (nr, ng, nb)
        }
        ModifiedSpace::Rgb => (r, g, b),
    };

    let color = build_legacy_color(
        modified,
        c.space,
        clamp_rgb(r_out),
        clamp_rgb(g_out),
        clamp_rgb(b_out),
        alpha,
    );
    Ok(Value::Color(color))
}

/// 钳制 legacy_rgb 到有效范围——仅在最终输出时调用。
fn clamp_rgb(v: f64) -> f64 {
    v.clamp(0.0, 255.0)
}

// ── 角度提取器 + CIE 通道辅助 ──────────────────────────────────────────

/// 角度：rad/grad/turn/deg 统一转换为度；`none` → NaN；`calc(±infinity)` → ±inf。
pub(super) fn angle_deg(kw: &HashMap<String, Value>, key: &str) -> Option<f64> {
    match kw.get(key) {
        Some(Value::Number(n, None)) => Some(*n),
        Some(Value::Number(n, Some(unit))) => Some(match unit.as_str() {
            "deg" => *n,
            "rad" => n.to_degrees(),
            "grad" => *n * 0.9,
            "turn" => *n * 360.0,
            _ => *n,
        }),
        Some(Value::String(s, false)) if s == "none" => Some(f64::NAN),
        Some(Value::Calc(s)) => extract_calc_f64(s).map(|v| {
            // 无穷大角度无意义，规范化到 0（Dart Sass 行为）
            if v.is_infinite() { 0.0 } else { v }
        }),
        _ => None,
    }
}

/// CIE 通道调整/变化：应用 cie_channel 提取 + 自定义变换 f
pub(super) fn apply_cie_channel(
    init: f64,
    kw: &HashMap<String, Value>,
    key: &str,
    max: f64,
    f: impl Fn(f64, f64) -> f64,
) -> f64 {
    cie_channel(kw, key, max).map(|delta_or_val| f(init, delta_or_val)).unwrap_or(init)
}

// ── 颜色空间转换辅助 ──────────────────────────────────────────────────────

/// HSL → RGB，返回 (r, g, b) 0-255 范围。
pub(super) fn hsl_to_rgb_channels(h: f64, s: f64, l: f64) -> (f64, f64, f64) {
    let c = Evaluator::hsl_to_rgb(h, s, l);
    (c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2])
}

/// HWB → RGB，返回 (r, g, b, alpha) 0-255 范围。
pub(super) fn hwb_to_rgb_channels(h: f64, w: f64, bk: f64, a: f64) -> (f64, f64, f64, f64) {
    let c = Evaluator::hwb_to_rgb(h, w, bk, a);
    (c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2], c.a)
}
