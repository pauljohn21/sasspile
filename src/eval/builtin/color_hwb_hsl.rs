#![allow(
    clippy::many_single_char_names,
    clippy::single_char_pattern,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::items_after_statements
)]
//! Color 内建函数 — HSL 通道操作 + 共享辅助函数。
//!
//! 包含 complement/hsl/hsla/adjust-hue/saturate/desaturate/transparentize/opacify/
//! alpha/red/green/blue/hue/saturation/lightness。
//! HWB 相关函数已迁移到 `color_hwb.rs`。
//! 注意：invert/grayscale/color-channel 仍在 color.rs 中。

use super::super::Evaluator;
use crate::error::{Result, SassError};
use crate::parse::ast::{Color, ColorOutput, ColorSpace, Separator, Value};
use imbl::HashMap;

// ── 共享辅助函数（pub(super) 供 color_hwb.rs 使用）──────────────────────

/// 从 Value 提取数值或 NaN（用于 none 通道处理）。
pub(crate) fn extract_none_num(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n, _) => Some(*n),
        Value::String(s, false) if s == "none" => Some(f64::NAN),
        _ => None,
    }
}

/// 合并位置参数和命名参数——用于 hsl($hue: 0, $saturation: 100%, ...) 等。
pub(super) fn merge_named_color_args(
    args: &[Value],
    kw_args: &HashMap<String, Value>,
    names: &[&str],
) -> Vec<Value> {
    let extra: Vec<Value> = names
        .iter()
        .enumerate()
        .filter_map(|(i, name)| {
            // 跳过已由位置参数覆盖的命名参数
            if i < args.len() {
                return None;
            }
            kw_args
                .get(*name)
                .or_else(|| kw_args.get(&format!("${name}")))
                .cloned()
        })
        .collect();
    args.iter().cloned().chain(extra).collect()
}

/// 展开空格分隔的 List 参数——用于 color.hsl(0 100% 50%) 等 CSS Level 4 语法。
pub(super) fn flatten_space_list(args: &[Value]) -> Vec<Value> {
    if let Some(Value::List(items, Separator::Space, false)) = args.first() {
        match args.len() {
            1 => return items.clone(),
            _ => {}
        }
        let mut flat = items.clone();
        flat.extend(args[1..].iter().cloned());
        return flat;
    }
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
pub(super) fn format_hue(v: &Value) -> String {
    v.to_string()
}

/// 将角度单位转换为 deg（CSS 基础单位），用于 HSL/HWB hue 规范化。
pub(super) fn angle_to_deg(n: f64, unit: Option<&String>) -> f64 {
    match unit.map(String::as_str) {
        Some("deg") | None => n,
        Some("rad") => n.to_degrees(),
        Some("grad") => n * 360.0 / 400.0,
        Some("turn") => n * 360.0,
        _ => n,
    }
}

/// 格式化 HWB 透传字符串输出。
pub(super) fn format_hwb_passthrough(args: &[Value]) -> String {
    match args.len() {
        4 => {
            let h_deg = match &args[0] {
                Value::Number(n, _) if !n.is_nan() => format!("{n}deg"),
                _ => format_hue(&args[0]),
            };
            format!("hwb({h_deg} {} {} / {})", args[1], args[2], args[3])
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

// ── HSL/HWB 通道操作入口 ─────────────────────────────────────────────

pub fn call(name: &str, args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Option<Value>> {
    match name {
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
                chans if chans.iter().any(|a| matches!(a, Value::String(s, false) if s == "none")) => {
                    let c: Vec<f64> = chans.iter().map(|v| extract_none_num(v).unwrap_or(f64::NAN)).collect();
                    let rgb = Evaluator::hsl_to_rgb(c[0], c[1] / 100.0, c[2] / 100.0);
                    Ok(Some(Value::Color(Color::with_hsl(c[0], c[1] / 100.0, c[2] / 100.0, c.get(3).copied().unwrap_or(100.0) / 100.0, ColorOutput::Auto, rgb.legacy_rgb))))
                }
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
                chans if chans.iter().any(|a| matches!(a, Value::String(s, false) if s == "none")) => {
                    let c: Vec<f64> = chans.iter().map(|v| extract_none_num(v).unwrap_or(f64::NAN)).collect();
                    let rgb = Evaluator::hsl_to_rgb(c[0], c[1] / 100.0, c[2] / 100.0);
                    Ok(Some(Value::Color(Color::with_hsl(c[0], c[1] / 100.0, c[2] / 100.0, c.get(3).copied().unwrap_or(f64::NAN) / 100.0, ColorOutput::Auto, rgb.legacy_rgb))))
                }
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
                    // CSS 透传：旧 IE filter 语法 alpha(opacity=0)
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
