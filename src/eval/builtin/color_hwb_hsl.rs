#![allow(
    clippy::many_single_char_names,
    clippy::single_char_pattern,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
//! Color 内建函数 — HSL/HWB 通道操作。
//!
//! 包含 hwb/whiteness/blackness/complement/hsl/hsla/adjust-hue/
//! saturate/desaturate/transparentize/opacify/alpha/red/green/blue/hue/saturation/lightness。
//! 注意：invert/grayscale/color-channel 仍在 color.rs 中。

use super::super::Evaluator;
use super::color_parse::extract_calc_f64;
use crate::error::{Result, SassError};
use crate::parse::ast::{Color, ColorOutput, ColorSpace, Separator, Value};
use std::collections::HashMap;

/// 从 Value 提取数值或 NaN（用于 none 通道处理）。
pub(crate) fn extract_none_num(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n, _) => Some(*n),
        Value::String(s, false) if s == "none" => Some(f64::NAN),
        _ => None,
    }
}

/// 合并位置参数和命名参数——用于 hsl($hue: 0, $saturation: 100%, ...) 等。
/// 按 names 顺序从 kw_args 提取参数补充到 args 中。
fn merge_named_color_args(
    args: &[Value],
    kw_args: &HashMap<String, Value>,
    names: &[&str],
) -> Vec<Value> {
    let mut result = args.to_vec();
    for name in names {
        match kw_args.get(*name).or_else(|| kw_args.get(&format!("${name}"))) {
            Some(v) => {
                // 只在位置参数不足时补充
                let idx = names.iter().position(|n| *n == *name).unwrap_or(0);
                if idx >= result.len() {
                    result.push(v.clone());
                }
            }
            None => {}
        }
    }
    result
}

/// 展开空格分隔的 List 参数——用于 color.hsl(0 100% 50%) 等 CSS Level 4 语法。
/// 当参数只有一个且为 space-separated list 时，展开为独立参数。
/// 也支持 List + alpha 参数的情况（如 hsl(0 100% 50% / 0.5)）。
/// 同时处理 `SlashLiteral` 分隔（声明值中 / 被解析为 `SlashLiteral`）。
fn flatten_space_list(args: &[Value]) -> Vec<Value> {
    if let Some(Value::List(items, Separator::Space, false)) = args.first() {
        match args.len() {
            1 => return items.clone(),
            _ => {}
        }
        // List + alpha 参数：展开列表并追加额外参数
        let mut flat = items.clone();
        flat.extend(args[1..].iter().cloned());
        return flat;
    }
    // SlashLiteral 分隔的列表：hsl(H S L / A) → [Space[H,S,L], A]
    if args.len() == 1
        && let Some(Value::List(items, Separator::SlashLiteral | Separator::Slash, false)) =
            args.first()
        && items.len() == 2
    {
        let mut flat = Vec::new();
        if let Some(Value::List(hsl_items, Separator::Space, false)) = items.first() {
            flat.extend(hsl_items.iter().cloned());
        } else {
            flat.push(items[0].clone());
        }
        flat.push(items[1].clone());
        return flat;
    }
    args.to_vec()
}

/// 格式化 hue 值——passthrough 序列化时保持原始表示。
/// Sass spec 要求 hsl(1, ...) 输出 `hsl(1, ...)` 而非 `hsl(1deg, ...)`。
fn format_hue(v: &Value) -> String {
    v.to_string()
}

/// 将角度单位转换为 deg（CSS 基础单位），用于 HSL/HWB hue 规范化。
/// Sass spec: deg=as-is, rad=to_degrees, grad=n*360/400, turn=n*360, unitless=as-is。
fn angle_to_deg(n: f64, unit: Option<&String>) -> f64 {
    match unit.map(String::as_str) {
        Some("deg") | None => n,
        Some("rad") => n.to_degrees(),
        Some("grad") => n * 360.0 / 400.0,
        Some("turn") => n * 360.0,
        // 未知单位：按 Sass spec 保留原值（调用处应发出 deprecation warning）
        _ => n,
    }
}

/// 格式化 HWB 透传字符串输出。
fn format_hwb_passthrough(args: &[Value]) -> String {
    match args.len() {
        4 => {
            let h_deg = match &args[0] {
                Value::Number(n, _) if !n.is_nan() => format!("{}deg", n),
                _ => format_hue(&args[0]),
            };
            format!(
                "hwb({h_deg} {} {} / {})",
                args[1].to_string(),
                args[2].to_string(),
                args[3].to_string()
            )
        }
        _ => {
            let arg_strs: Vec<String> = args
                .iter()
                .enumerate()
                .map(|(i, a)| if i == 0 { format_hue(a) } else { a.to_string() })
                .collect();
            format!("hwb({})", arg_strs.join(" "))
        }
    }
}

pub fn call(name: &str, args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Option<Value>> {
    match name {
        "hwb" => {
            // 合并命名参数到位置参数
            let merged = merge_named_color_args(args, kw_args, &["hue", "whiteness", "blackness", "alpha"]);
            // 展开空格分隔的 List（CSS hwb() 语法：hwb(0deg 30% 40%)）
            let flat = flatten_space_list(&merged);

            /// 颜色通道值提取结果——区分数值、missing、不可解析。
            /// 用于 HWB whiteness/blackness/none 语义：`none` 表示通道缺失，
            /// `Number(NaN)` 表示计算得到的数值 NaN（传播计算）。
            #[derive(Clone, Copy)]
            enum ChannelVal {
                Number(f64),
                Missing,
            }

            /// 尝试从 Value 提取通道数值。
            /// Ok(Some(Number(n))) = 数值 n；Ok(Some(Missing)) = none 关键字；Ok(None) = 不可解析（透传）。
            fn extract_channel(v: &Value) -> Result<Option<ChannelVal>> {
                match v {
                    Value::String(s, false) if s == "none" => Ok(Some(ChannelVal::Missing)),
                    Value::Number(n, Some(u)) if u == "%" => Ok(Some(ChannelVal::Number(*n / 100.0))),
                    Value::Number(n, _) => Ok(Some(ChannelVal::Number(
                        // 统一 -0 → +0
                        if *n == 0.0 { 0.0 } else { *n },
                    ))),
                    // calc(infinity/-infinity/NaN) → 解析为对应 f64（数值，非 missing）
                    _ => match extract_calc_f64(&v.to_string()) {
                        Some(n) => Ok(Some(ChannelVal::Number(n))),
                        None => Ok(None),
                    },
                }
            }

            /// 从 Value 提取 alpha 值（支持 % 单位：100% → 1.0，以及特殊值 clamp）。
            fn extract_alpha(v: &Value) -> Result<Option<f64>> {
                match v {
                    Value::Number(n, Some(u)) if u == "%" => {
                        Ok(Some((*n / 100.0).clamp(0.0, 1.0)))
                    }
                    Value::Number(n, _) => Ok(Some(n.clamp(0.0, 1.0))),
                    _ => match extract_calc_f64(&v.to_string()) {
                        Some(f64::INFINITY) => Ok(Some(1.0)),
                        Some(f64::NEG_INFINITY) => Ok(Some(0.0)),
                        Some(n) if n.is_nan() => Ok(Some(0.0)),
                        Some(n) => Ok(Some(n.clamp(0.0, 1.0))),
                        None => Ok(None),
                    },
                }
            }

            /// 构建 HWB Color（计算 RGB + 存储通道值）。
            /// whiteness/blackness 退化值处理：
            /// - ±Infinity: +inf → 纯 hue (w=0, b=0)；-inf → 纯黑 (w=0, b=1)
            /// - Missing (none 关键字): 保留为 NaN（序列化保持 HWB 格式）
            /// - 数值 NaN (calc(NaN)): 传播为 0 参与计算（结果序列化为 HSL）
            fn build_hwb(h: f64, h_is_nan: bool, w_ch: ChannelVal, bk_ch: ChannelVal, a: f64) -> Value {
                // 用于计算的 w/bk（数值 NaN 传播为 0）
                let (w_compute, bk_compute) = match (w_ch, bk_ch) {
                    // 任何 +infinity → 纯 hue
                    (ChannelVal::Number(w), _) if w.is_infinite() && w > 0.0 => (0.0, 0.0),
                    (_, ChannelVal::Number(bk)) if bk.is_infinite() && bk > 0.0 => (0.0, 0.0),
                    // 任何 -infinity → 纯黑
                    (ChannelVal::Number(w), _) if w.is_infinite() && w < 0.0 => (0.0, 1.0),
                    (_, ChannelVal::Number(bk)) if bk.is_infinite() && bk < 0.0 => (0.0, 1.0),
                    // 数值 NaN 传播为 0；none → 保留 NaN (missing)
                    (ChannelVal::Number(w), ChannelVal::Number(bk)) => {
                        (if w.is_nan() { 0.0 } else { w }, if bk.is_nan() { 0.0 } else { bk })
                    }
                    (w, bk) => {
                        let w = match w {
                            ChannelVal::Number(n) if n.is_nan() => 0.0,
                            ChannelVal::Number(n) => n,
                            ChannelVal::Missing => 0.0,
                        };
                        let bk = match bk {
                            ChannelVal::Number(n) if n.is_nan() => 0.0,
                            ChannelVal::Number(n) => n,
                            ChannelVal::Missing => 0.0,
                        };
                        (w, bk)
                    }
                };
                // 用于存储的 w/bk（missing 用 NaN 保留以维持 HWB 格式）
                let (w_store, bk_store) = match (w_ch, bk_ch) {
                    (ChannelVal::Missing, ChannelVal::Missing) => (f64::NAN, f64::NAN),
                    (ChannelVal::Missing, _) => (f64::NAN, bk_compute),
                    (_, ChannelVal::Missing) => (w_compute, f64::NAN),
                    _ => (w_compute, bk_compute),
                };
                // h：如果是 none/calc(NaN)，存储为 NaN 以保持 HWB 格式
                let h_store = if h_is_nan { f64::NAN } else { h };
                let c = Evaluator::hwb_to_rgb(h, w_compute, bk_compute, a);
                Value::Color(Color::with_hwb(h_store, w_store, bk_store, a, c.legacy_rgb))
            }

            /// 解析 hue，不可解析时返回 None（调用方决定透传）。
            /// 返回 (值, 单位, 是否是none关键字)。
            fn extract_hue(v: &Value) -> Result<Option<(f64, Option<String>, bool)>> {
                match v {
                    Value::String(s, false) if s == "none" => Ok(Some((f64::NAN, None, true))),
                    Value::Number(n, unit) => Ok(Some((*n, unit.clone(), false))),
                    _ => match extract_calc_f64(&v.to_string()) {
                        Some(n) => Ok(Some((n, None, false))),
                        None => Ok(None),
                    },
                }
            }

            /// hue: 非有限值 normalize 到 0。
            fn normalize_hue(h: f64, unit: Option<&String>) -> f64 {
                match h.is_finite() {
                    true => angle_to_deg(h, unit),
                    false => 0.0,
                }
            }

            // 解析所有通道：参数不足时透传
            if flat.len() < 3 {
                return Ok(Some(Value::String(format_hwb_passthrough(&flat), false)));
            }
            let (h_parsed, h_unit, h_is_none_kw) = match extract_hue(&flat[0]) {
                Ok(Some((n, unit, is_none))) => (n, unit, is_none),
                Ok(None) => return Ok(Some(Value::String(format_hwb_passthrough(&flat), false))),
                Err(e) => return Err(e),
            };
            let w_parsed = match extract_channel(&flat[1]) {
                Ok(Some(c)) => c,
                Ok(None) => return Ok(Some(Value::String(format_hwb_passthrough(&flat), false))),
                Err(e) => return Err(e),
            };
            let bk_parsed = match extract_channel(&flat[2]) {
                Ok(Some(c)) => c,
                Ok(None) => return Ok(Some(Value::String(format_hwb_passthrough(&flat), false))),
                Err(e) => return Err(e),
            };

            // 如果有任何通道是 Missing（none 关键字），需构建特殊颜色
            let _colors = [w_parsed, bk_parsed];
            let a_parsed = match flat.len() {
                4 => match extract_alpha(&flat[3]) {
                    Ok(Some(n)) => n,
                    Ok(None) => return Ok(Some(Value::String(format_hwb_passthrough(&flat), false))),
                    Err(e) => return Err(e),
                },
                _ => 1.0,
            };

            let h = normalize_hue(h_parsed, h_unit.as_ref());
            // h_is_nan 存储为 NaN 的条件：只有 none 关键字才保持 HWB 格式
            // calc(NaN) 应该转为 0 并序列化为 HSL
            let h_store_nan = h_is_none_kw && h_parsed.is_nan();
            Ok(Some(build_hwb(h, h_store_nan, w_parsed, bk_parsed, a_parsed)))
        }
        "whiteness" => {
            let color_arg = args.first().or_else(|| kw_args.get("color"));
            match color_arg {
                Some(Value::Color(c)) => {
                    let w =
                        c.legacy_rgb[0].min(c.legacy_rgb[1]).min(c.legacy_rgb[2]) / 255.0 * 100.0;
                    Ok(Some(Value::Number(w, Some("%".to_string()))))
                }
                _ => Err(SassError::Eval(
                    "whiteness requires 1 color argument".into(),
                )),
            }
        }
        "blackness" => {
            let color_arg = args.first().or_else(|| kw_args.get("color"));
            match color_arg {
                Some(Value::Color(c)) => {
                    let b = (1.0
                        - c.legacy_rgb[0].max(c.legacy_rgb[1]).max(c.legacy_rgb[2]) / 255.0)
                        * 100.0;
                    Ok(Some(Value::Number(b, Some("%".to_string()))))
                }
                _ => Err(SassError::Eval(
                    "blackness requires 1 color argument".into(),
                )),
            }
        }
        "complement" => {
            let color_arg = args.first().or_else(|| kw_args.get("color"));
            match color_arg {
                Some(Value::Color(c)) => {
                    let (h, s, l) =
                        Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
                    let new_h = (h + 180.0).rem_euclid(360.0);
                    let new_c = Evaluator::hsl_to_rgb(new_h, s, l);
                    Ok(Some(Value::Color(Color::with_rgb(
                        new_c.legacy_rgb[0],
                        new_c.legacy_rgb[1],
                        new_c.legacy_rgb[2],
                        c.a,
                        ColorSpace::Rgb,
                        ColorOutput::Auto,
                    ))))
                }
                _ => Err(SassError::Eval(
                    "complement requires 1 color argument".into(),
                )),
            }
        }
        "hsl" => {
            // Sass spec: HSL 输出始终用逗号分隔。
            let flat = flatten_space_list(&merge_named_color_args(args, kw_args, &["hue", "saturation", "lightness", "alpha"]));
            match &flat[..] {
                [Value::Number(h, hu), Value::Number(s, _), Value::Number(l, _)] => {
                    // Sass spec: saturation 负值 clamp 到 0，hue/lightness 不 clamp
                    // hue 先转为 deg（处理 rad/grad/turn 单位）
                    let h = angle_to_deg(*h, hu.as_ref());
                    let s = s.max(0.0);
                    let c = Evaluator::hsl_to_rgb(h, s / 100.0, *l / 100.0);
                    Ok(Some(Value::Color(Color::with_hsl(h, s / 100.0, *l / 100.0, 1.0, ColorOutput::Auto, c.legacy_rgb))))
                }
                [Value::Number(h, hu), Value::Number(s, _), Value::Number(l, _), Value::Number(a, _)] => {
                    let h = angle_to_deg(*h, hu.as_ref());
                    let s = s.max(0.0);
                    let c = Evaluator::hsl_to_rgb(h, s / 100.0, *l / 100.0);
                    Ok(Some(Value::Color(Color::with_hsl(h, s / 100.0, *l / 100.0, *a, ColorOutput::Auto, c.legacy_rgb))))
                }
                // CSS Color 4 missing channels: hsl(none 50% 50%) → Color with NaN channels
                chans if chans.iter().any(|a| matches!(a, Value::String(s, false) if s == "none")) => {
                    let c: Vec<f64> = chans.iter().map(|v| extract_none_num(v).unwrap_or(f64::NAN)).collect();
                    let rgb = Evaluator::hsl_to_rgb(c[0], c[1] / 100.0, c[2] / 100.0);
                    Ok(Some(Value::Color(Color::with_hsl(c[0], c[1] / 100.0, c[2] / 100.0, c.get(3).copied().unwrap_or(100.0) / 100.0, ColorOutput::Auto, rgb.legacy_rgb))))
                }
                // CSS 透传：参数包含 var()/calc() 等非数值时，原样输出逗号分隔字符串
                chans if chans.iter().any(|a| !matches!(a, Value::Number(_, _))) => {
                    let sep = ", ";
                    let arg_strs: Vec<String> = chans.iter().enumerate().map(|(i, a)| if i == 0 { format_hue(a) } else { a.to_string() }).collect();
                    Ok(Some(Value::String(format!("hsl({})", arg_strs.join(sep)), false)))
                }
                _ => Err(SassError::Eval("hsl requires 3-4 arguments".into())),
            }
        }
        "hsla" => {
            let flat = flatten_space_list(&merge_named_color_args(args, kw_args, &["hue", "saturation", "lightness", "alpha"]));
            match &flat[..] {
                [Value::Number(h, hu), Value::Number(s, _), Value::Number(l, _), Value::Number(a, _)] => {
                    let h = angle_to_deg(*h, hu.as_ref());
                    let s = s.max(0.0);
                    let c = Evaluator::hsl_to_rgb(h, s / 100.0, *l / 100.0);
                    Ok(Some(Value::Color(Color::with_hsl(h, s / 100.0, *l / 100.0, *a, ColorOutput::Auto, c.legacy_rgb))))
                }
                // CSS Color 4 missing channels: hsla(none 50% 50% / 0.5) → Color with NaN
                chans if chans.iter().any(|a| matches!(a, Value::String(s, false) if s == "none")) => {
                    let c: Vec<f64> = chans.iter().map(|v| extract_none_num(v).unwrap_or(f64::NAN)).collect();
                    let rgb = Evaluator::hsl_to_rgb(c[0], c[1] / 100.0, c[2] / 100.0);
                    Ok(Some(Value::Color(Color::with_hsl(c[0], c[1] / 100.0, c[2] / 100.0, c.get(3).copied().unwrap_or(f64::NAN) / 100.0, ColorOutput::Auto, rgb.legacy_rgb))))
                }
                // CSS 透传：始终用逗号分隔
                chans if chans.iter().any(|a| !matches!(a, Value::Number(_, _))) => {
                    let arg_strs: Vec<String> = chans.iter().enumerate().map(|(i, a)| if i == 0 { format_hue(a) } else { a.to_string() }).collect();
                    Ok(Some(Value::String(format!("hsla({})", arg_strs.join(", ")), false)))
                }
                _ => Err(SassError::Eval("hsla requires 4 arguments".into())),
            }
        }
        "adjust-hue" => {
            let color_arg = args.first().or_else(|| kw_args.get("color"));
            let deg_arg = args
                .get(1)
                .or_else(|| kw_args.get("degrees"))
                .or_else(|| kw_args.get("hue"));
            match (color_arg, deg_arg) {
                (Some(Value::Color(c)), Some(Value::Number(deg, _))) => {
                    let (h, s, l) =
                        Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
                    let new_h = (h + *deg).rem_euclid(360.0);
                    let new_c = Evaluator::hsl_to_rgb(new_h, s, l);
                    Ok(Some(Value::Color(Color::with_rgb(
                        new_c.legacy_rgb[0],
                        new_c.legacy_rgb[1],
                        new_c.legacy_rgb[2],
                        c.a,
                        ColorSpace::Rgb,
                        ColorOutput::Auto,
                    ))))
                }
                _ => Err(SassError::Eval(
                    "adjust-hue 需要 (color, degrees) 参数".into(),
                )),
            }
        }
        "saturate" => {
            let color_arg = args.first().or_else(|| kw_args.get("color"));
            let amount_arg = args.get(1).or_else(|| kw_args.get("amount"));
            match (color_arg, amount_arg) {
                (Some(Value::Color(c)), Some(Value::Number(amount, _))) => {
                    let (h, s, l) =
                        Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
                    let new_s = (s + *amount / 100.0).min(1.0);
                    let new_c = Evaluator::hsl_to_rgb(h, new_s, l);
                    Ok(Some(Value::Color(Color::with_rgb(
                        new_c.legacy_rgb[0],
                        new_c.legacy_rgb[1],
                        new_c.legacy_rgb[2],
                        c.a,
                        ColorSpace::Rgb,
                        ColorOutput::Auto,
                    ))))
                }
                // CSS 滤镜函数透传：saturate(number)
                (Some(Value::Number(n, _)), None) => {
                    Ok(Some(Value::String(format!("saturate({n})"), false)))
                }
                _ => Err(SassError::Eval(
                    "saturate requires (color, amount) arguments".into(),
                )),
            }
        }
        "desaturate" => {
            let color_arg = args.first().or_else(|| kw_args.get("color"));
            let amount_arg = args.get(1).or_else(|| kw_args.get("amount"));
            match (color_arg, amount_arg) {
                (Some(Value::Color(c)), Some(Value::Number(amount, _))) => {
                    let (h, s, l) =
                        Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
                    let new_s = (s - *amount / 100.0).max(0.0);
                    let new_c = Evaluator::hsl_to_rgb(h, new_s, l);
                    Ok(Some(Value::Color(Color::with_rgb(
                        new_c.legacy_rgb[0],
                        new_c.legacy_rgb[1],
                        new_c.legacy_rgb[2],
                        c.a,
                        ColorSpace::Rgb,
                        ColorOutput::Auto,
                    ))))
                }
                _ => Err(SassError::Eval(
                    "desaturate requires (color, amount) arguments".into(),
                )),
            }
        }
        "transparentize" | "fade-out" => {
            let color_arg = args.first().or_else(|| kw_args.get("color"));
            let amount_arg = args.get(1).or_else(|| kw_args.get("amount"));
            match (color_arg, amount_arg) {
                (Some(Value::Color(c)), Some(Value::Number(amount, _))) => {
                    Ok(Some(Value::Color(Color::rgba(
                        c.legacy_rgb[0],
                        c.legacy_rgb[1],
                        c.legacy_rgb[2],
                        (c.a - *amount).max(0.0),
                    ))))
                }
                _ => Err(SassError::Eval(
                    "transparentize requires (color, amount) arguments".into(),
                )),
            }
        }
        "opacify" | "fade-in" => {
            let color_arg = args.first().or_else(|| kw_args.get("color"));
            let amount_arg = args.get(1).or_else(|| kw_args.get("amount"));
            match (color_arg, amount_arg) {
                (Some(Value::Color(c)), Some(Value::Number(amount, _))) => {
                    Ok(Some(Value::Color(Color::rgba(
                        c.legacy_rgb[0],
                        c.legacy_rgb[1],
                        c.legacy_rgb[2],
                        (c.a + *amount).min(1.0),
                    ))))
                }
                _ => Err(SassError::Eval(
                    "opacify requires (color, amount) arguments".into(),
                )),
            }
        }
        "alpha" | "opacity" => {
            let color_arg = args.first().or_else(|| kw_args.get("color"));
            match color_arg {
                Some(Value::Color(c)) => Ok(Some(Value::Number(c.a, None))),
                _ => {
                    // CSS 透传：旧 IE filter 语法 alpha(opacity=0) — 关键字参数直接透传
                    match (!kw_args.is_empty(), !args.is_empty()) {
                        (true, _) => {
                            let kw_str = kw_args
                                .iter()
                                .map(|(k, v)| format!("{k}={v}"))
                                .collect::<Vec<_>>()
                                .join(", ");
                            Ok(Some(Value::String(format!("{name}({kw_str})"), false)))
                        }
                        (false, true) => {
                            // CSS 透传：非颜色位置参数原样输出（如 alpha(var(--x))）
                            let arg_str = args
                                .iter()
                                .map(std::string::ToString::to_string)
                                .collect::<Vec<_>>()
                                .join(", ");
                            Ok(Some(Value::String(format!("{name}({arg_str})"), false)))
                        }
                        (false, false) => Err(SassError::Eval("alpha requires 1 color argument".into())),
                    }
                }
            }
        }
        "red" => {
            let color_arg = args.first().or_else(|| kw_args.get("color"));
            match color_arg {
                Some(Value::Color(c)) => Ok(Some(Value::Number(c.legacy_rgb[0], None))),
                _ => Err(SassError::Eval("red requires 1 color argument".into())),
            }
        }
        "green" => {
            let color_arg = args.first().or_else(|| kw_args.get("color"));
            match color_arg {
                Some(Value::Color(c)) => Ok(Some(Value::Number(c.legacy_rgb[1], None))),
                _ => Err(SassError::Eval("green requires 1 color argument".into())),
            }
        }
        "blue" => {
            let color_arg = args.first().or_else(|| kw_args.get("color"));
            match color_arg {
                Some(Value::Color(c)) => Ok(Some(Value::Number(c.legacy_rgb[2], None))),
                _ => Err(SassError::Eval("blue requires 1 color argument".into())),
            }
        }
        "hue" => {
            let color_arg = args.first().or_else(|| kw_args.get("color"));
            match color_arg {
                Some(Value::Color(c)) => {
                    let (h, _, _) =
                        Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
                    Ok(Some(Value::Number(h, Some("deg".into()))))
                }
                _ => Err(SassError::Eval("hue requires 1 color argument".into())),
            }
        }
        "saturation" => {
            let color_arg = args.first().or_else(|| kw_args.get("color"));
            match color_arg {
                Some(Value::Color(c)) => {
                    let (_, s, _) =
                        Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
                    Ok(Some(Value::Number(s * 100.0, Some("%".into()))))
                }
                _ => Err(SassError::Eval(
                    "saturation requires 1 color argument".into(),
                )),
            }
        }
        "lightness" => {
            let color_arg = args.first().or_else(|| kw_args.get("color"));
            match color_arg {
                Some(Value::Color(c)) => {
                    let (_, _, l) =
                        Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
                    Ok(Some(Value::Number(l * 100.0, Some("%".into()))))
                }
                _ => Err(SassError::Eval(
                    "lightness requires 1 color argument".into(),
                )),
            }
        }
        _ => Ok(None),
    }
}
