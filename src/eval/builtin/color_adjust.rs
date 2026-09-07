#![allow(
    clippy::many_single_char_names,
    clippy::single_char_pattern,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
//! `color.adjust` / `color.change` / `color.scale` 实现。
//!
//! 支持所有 CSS Color 4 颜色空间：
//! - Legacy: RGB, HSL, HWB
//! - Modern: Lab, Lch, Oklab, Oklch, `DisplayP3`, sRGB, sRGB-Linear, etc.
//!
//! 三种语义统一用 `apply_channel` 表达：
//! - adjust: `|v, d| v + d` — 增量
//! - change: `|_v, d| d` — 赋值
//! - scale: `scale_channel` — 按比例
//!
//! 每种语义一个 extractor 决定单位转换策略：
//! - `raw_value` — 字面数值（RGB、chroma 等）
//! - `percentage` — n/100 (HSL saturation/lightness)
//! - `angle_deg` — 角度统一到度

use crate::error::{Result, SassError};
use crate::eval::Evaluator;
use crate::parse::ast::{Color, ColorOutput, ColorSpace, Value};
use imbl::HashMap;

use super::color_parse::extract_calc_f64;

/// 提取 calc() 数值，但忽略 NaN（`calc(NaN)` 视为无操作）。
/// 仅用于 change 通道：`calc(±infinity)` → ±inf，其他 → None。
fn extract_calc_f64_nonan(s: &str) -> Option<f64> {
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
        match (pct >= 0.0, max == f64::MAX) {
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
fn extract_color<'a>(args: &'a [Value], kw_args: &'a HashMap<String, Value>) -> Result<&'a Color> {
    match args.first().or_else(|| kw_args.get("color")) {
        Some(Value::Color(c)) => Ok(c),
        Some(v) => Err(SassError::Eval(format!("$color: {v} is not a color."))),
        None => Err(SassError::Eval("Missing argument $color.".into())),
    }
}

/// 根据颜色空间的通道语义，将 X/Y/Z 关键词映射到 channels[0..2]。
fn modern_channel_keys(space: ColorSpace) -> [&'static str; 3] {
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

// ── Modern RGB 空间 (DisplayP3, sRGB, A98, ProPhoto, Rec2020) ─────────────
// 内部尺度 0-1。percent → n/100（percentage points），unitless → raw，none → NaN

fn adjust_modern_rgb_space(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let [c0, c1, c2] = modern_channel_keys(c.space);
    let r = apply_cie_channel(c.channels[0], kw_args, c0, 1.0, |v, d| v + d);
    let g = apply_cie_channel(c.channels[1], kw_args, c1, 1.0, |v, d| v + d);
    let b = apply_cie_channel(c.channels[2], kw_args, c2, 1.0, |v, d| v + d);
    let a = apply_channel(c.a, kw_args, "alpha", alpha_value, |v, d| (v + d).clamp(0.0, 1.0));

    Ok(Value::Color(Color::with_space(
        c.space,
        [r, g, b],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

fn change_modern_rgb_space(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let [c0, c1, c2] = modern_channel_keys(c.space);
    let r = apply_cie_channel(c.channels[0], kw_args, c0, 1.0, |_v, d| d);
    let g = apply_cie_channel(c.channels[1], kw_args, c1, 1.0, |_v, d| d);
    let b = apply_cie_channel(c.channels[2], kw_args, c2, 1.0, |_v, d| d);
    let a = apply_channel(c.a, kw_args, "alpha", alpha_value, |_v, d| d.clamp(0.0, 1.0));

    Ok(Value::Color(Color::with_space(
        c.space,
        [r, g, b],
        a,
        c.output,
        c.legacy_rgb,
    )))
}

fn scale_modern_rgb_space(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let [c0, c1, c2] = modern_channel_keys(c.space);
    // 现代 RGB 空间：线性插值 val + (max - val)*pct，max=1.0
    let r = scale_channel(c.channels[0], 1.0, kw_args, c0);
    let g = scale_channel(c.channels[1], 1.0, kw_args, c1);
    let b = scale_channel(c.channels[2], 1.0, kw_args, c2);
    let a = scale_channel(c.a, 1.0, kw_args, "alpha").clamp(0.0, 1.0);

    Ok(Value::Color(Color::with_space(
        c.space, [r, g, b], a, c.output, c.legacy_rgb,
    )))
}

// ── Legacy (RGB/HSL/HWB) ──────────────────────────────────────────────
// 修改哪个通道集决定计算路径+输出格式（哪个 space 被改，输出就是哪个）

#[derive(Clone, Copy, PartialEq, Eq)]
enum ModifiedSpace { Rgb, Hsl, Hwb }

fn classify_modified(kw: &HashMap<String, Value>) -> ModifiedSpace {
    if kw.contains_key("whiteness") || kw.contains_key("blackness") { return ModifiedSpace::Hwb; }
    if kw.contains_key("hue") || kw.contains_key("saturation") || kw.contains_key("lightness") { return ModifiedSpace::Hsl; }
    ModifiedSpace::Rgb
}

// 根据被修改的色彩空间构造 Color：Rgb→保原空间（超范围→HSL回退）, Hsl/Hwb→build_channel_modified_color
fn build_legacy_color(
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
                let rgb_in_range = r >= 0.0 && r <= 255.0 && g >= 0.0 && g <= 255.0 && b >= 0.0 && b <= 255.0;
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
fn build_channel_modified_color(original: ColorSpace, r: f64, g: f64, b: f64, alpha: f64) -> Color {
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

/// 钳制 legacy_rgb 到有效范围——仅在最终输出时调用。
fn clamp_rgb(v: f64) -> f64 {
    v.clamp(0.0, 255.0)
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
    // 当原始空间是 HSL 时，直接保持 HSL channels（不通过 RGB 转换）
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
            // 获取 HSL 初始值：已有 HSL 数据或从 RGB 推导
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

fn change_legacy(c: &Color, kw_args: &HashMap<String, Value>) -> Result<Value> {
    let modified = classify_modified(kw_args);

    // RGB 通道设置（change 不 clamp——规范是绝对赋值，允许超出范围）
    let r = apply_channel(c.legacy_rgb[0], kw_args, "red", raw_value, |_v, d| d);
    let g = apply_channel(c.legacy_rgb[1], kw_args, "green", raw_value, |_v, d| d);
    let b = apply_channel(c.legacy_rgb[2], kw_args, "blue", raw_value, |_v, d| d);
    // alpha：change 允许 none（NaN），但非 NaN 值需 clamp 到 [0, 1]
let alpha = apply_channel(c.a, kw_args, "alpha", alpha_value, |_v, d| {
    if d.is_nan() { d } else { d.clamp(0.0, 1.0) }
});

    // HSL/HWB 通道设置
    let rgb_result = match modified {
        ModifiedSpace::Hwb => {
            let (h_init, hw_init, hb_init) =
                Evaluator::rgb_to_hwb(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
            let h =
                apply_channel(h_init, kw_args, "hue", angle_deg, |_v, d| d.rem_euclid(360.0));
            let hw = apply_channel(hw_init, kw_args, "whiteness", percentage, |_v, d| d);
            let hb = apply_channel(hb_init, kw_args, "blackness", percentage, |_v, d| d);
            // HWB 归一化：hw + hb > 1 时按比例缩放
            let sum = hw + hb;
            let (hw_n, hb_n) = match sum.is_nan() || sum.is_infinite() {
                true => (hw, hb),
                false => match sum > 1.0 {
                    true => (hw / sum, hb / sum),
                    false => (hw, hb),
                },
            };
            // 有 NaN → HWB(none) 格式；RGB 有效值 [0,255] → RGB 输出（hex/named/rgb%）；超范围 → HWB 格式
            let has_nan = h.is_nan() || hw.is_nan() || hb.is_nan();
            let clean_ch = |v: f64| if v.abs() < 1e-6 { 0.0 } else { v };
            let (nr, ng, nb, _) = hwb_to_rgb_channels(h, hw_n, hb_n, 1.0);
            let (nr_c, ng_c, nb_c) = (clean_ch(nr), clean_ch(ng), clean_ch(nb));
            let rgb_in_range = !has_nan
                && nr_c >= 0.0 && nr_c <= 255.0
                && ng_c >= 0.0 && ng_c <= 255.0
                && nb_c >= 0.0 && nb_c <= 255.0;
            return match rgb_in_range {
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
            };
        }
        ModifiedSpace::Hsl => {
            // 获取 HSL 初始通道：从已有 HSL 数据或从 RGB 推导
            let (h_init, s_init, l_init) = match c.space == ColorSpace::Hsl {
                true => (c.channels[0], c.channels[1], c.channels[2]),
                false => {
                    Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2])
                }
            };
            let h = apply_channel(h_init, kw_args, "hue", angle_deg, |_v, d| {
                d.rem_euclid(360.0)
            });
            // change 不 clamp——允许超出范围值
            let s = apply_channel(s_init, kw_args, "saturation", percentage, |_v, d| d);
            let l = apply_channel(l_init, kw_args, "lightness", percentage, |_v, d| d);
            // 清理浮点噪声：极小值归零（避免 -1e-15 导致 rgb_valid=false）
            let clean_rgb_ch = |v: f64| if v.abs() < 1e-6 { 0.0 } else { v };
            // 序列化规则：有 NaN → HSL(none)；RGB 有效值 [0,255] → RGB 输出（hex/named/rgb%）；超范围 → HSL 格式
            let rgb = hsl_to_rgb_channels(h, s, l);
            let rgb_clean = (clean_rgb_ch(rgb.0), clean_rgb_ch(rgb.1), clean_rgb_ch(rgb.2));
            let rgb_in_range = rgb_clean.0 >= 0.0 && rgb_clean.0 <= 255.0
                && rgb_clean.1 >= 0.0 && rgb_clean.1 <= 255.0
                && rgb_clean.2 >= 0.0 && rgb_clean.2 <= 255.0;
            return match (h.is_nan() || s.is_nan() || l.is_nan(), rgb_in_range) {
                (true, _) | (false, false) => Ok(Value::Color(build_channel_modified_color(
                    ColorSpace::Hsl, h, s, l, alpha,
                ))),
                (false, true) => Ok(Value::Color(Color::with_hsl(
                    h, s, l, alpha, ColorOutput::RgbPercent, [rgb_clean.0, rgb_clean.1, rgb_clean.2],
                ))),
            };
        }
        ModifiedSpace::Rgb => (r, g, b),
    };

    // change 不 clamp——保留 NaN/超出范围值以便序列化
    let color = build_legacy_color(
        modified,
        c.space,
        rgb_result.0,
        rgb_result.1,
        rgb_result.2,
        alpha,
    );
    Ok(Value::Color(color))
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
            let (nr, ng, nb, _) = hwb_to_rgb_channels(h, w_n, bk_n, alpha);
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
            let (nr, ng, nb) = hsl_to_rgb_channels(h, s, l);
            (nr, ng, nb)
        }
        ModifiedSpace::Rgb => (r, g, b),
    };

    // scale-color HSL/HWB-modified: 总是以 rgb(r%, g%, b%) 格式输出
    // scale-color RGB-modified: hex/rgb 格式（与 change-color 相同）
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

// ── 颜色空间转换辅助 ──────────────────────────────────────────────────────

/// HSL → RGB，返回 (r, g, b) 0-255 范围。
fn hsl_to_rgb_channels(h: f64, s: f64, l: f64) -> (f64, f64, f64) {
    use crate::eval::Evaluator;
    let c = Evaluator::hsl_to_rgb(h, s, l);
    (c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2])
}

/// HWB → RGB，返回 (r, g, b, alpha) 0-255 范围。
fn hwb_to_rgb_channels(h: f64, w: f64, bk: f64, a: f64) -> (f64, f64, f64, f64) {
    use crate::eval::Evaluator;
    let c = Evaluator::hwb_to_rgb(h, w, bk, a);
    (c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2], c.a)
}
