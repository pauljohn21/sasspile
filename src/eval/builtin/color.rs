#![allow(
    clippy::many_single_char_names,
    clippy::single_char_pattern,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
//! Color 内建函数（match arms 提取）。
//!
//! 包含 invert/grayscale/color-channel/hwb/complement/hsl/hsla/adjust-hue/
//! saturate/desaturate/transparentize/opacify/alpha/red/green/blue/hue/saturation/lightness。
//! 注意：rgba/rgb/darken/lighten/mix 仍在 builtin.rs 中（调用 `Self::builtin`_*）。

use super::super::Evaluator;
use crate::error::{Result, SassError};
use crate::parse::ast::{Color, ColorOutput, ColorSpace, Separator, Value};
use std::collections::HashMap;

/// 判断 Value 是否为 `none` 关键字。
fn is_none_str(v: &Value) -> bool {
    matches!(v, Value::String(s, false) if s == "none")
}

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

/// 格式化 hue 值——无单位数字添加 deg 后缀（CSS 规范化）。
fn format_hue(v: &Value) -> String {
    match v {
        Value::Number(n, None) => format!("{n}deg"),
        _ => v.to_string(),
    }
}

pub fn call(name: &str, args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Option<Value>> {
    match name {
        "invert" => {
            let color_arg = args.first().or_else(|| kw_args.get("color"));
            let space_arg = kw_args.get("space");
            match color_arg {
                Some(Value::Color(c)) => {
                    match c.space.is_legacy() {
                        true => {
                            // Legacy invert：RGB 反转（255 - channel），输出 Auto（查找命名色）
                            let r = 255.0 - c.legacy_rgb[0];
                            let g = 255.0 - c.legacy_rgb[1];
                            let b = 255.0 - c.legacy_rgb[2];
                            Ok(Some(Value::Color(Color::with_rgb(
                                r, g, b, c.a,
                                ColorSpace::Rgb,
                                ColorOutput::Auto,
                            ))))
                        }
                        false => {
                            // 现代空间 invert：各通道 1 - channel
                            // 如果指定了 $space，先转到该空间
                            let (r, g, b) = match space_arg {
                                Some(Value::String(s, _)) => {
                                    let converted = super::color_conv_ops::convert_space(c, s)?;
                                    match converted {
                                        Value::Color(cc) => (cc.channels[0], cc.channels[1], cc.channels[2]),
                                        _ => (c.channels[0], c.channels[1], c.channels[2]),
                                    }
                                }
                                _ => (c.channels[0], c.channels[1], c.channels[2]),
                            };
                            let target_space = match space_arg {
                                Some(Value::String(s, _)) => {
                                    ColorSpace::from_str(s).unwrap_or(c.space)
                                }
                                _ => c.space,
                            };
                            Ok(Some(Value::Color(Color::with_space(
                                target_space,
                                [1.0 - r, 1.0 - g, 1.0 - b],
                                c.a,
                                c.output,
                                [0.0, 0.0, 0.0],
                            ))))
                        }
                    }
                }
                // CSS 滤镜函数透传：invert(number) 非颜色参数
                _ if !args.is_empty() => {
                    let arg_str = args
                        .iter()
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ");
                    Ok(Some(Value::String(format!("invert({arg_str})"), false)))
                }
                _ => Err(SassError::Eval("invert requires 1 argument".into())),
            }
        }
        "grayscale" => {
            let color_arg = args.first().or_else(|| kw_args.get("color"));
            match color_arg {
                Some(Value::Color(c)) => {
                    let (h, _s, l) =
                        Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
                    let new_c = Evaluator::hsl_to_rgb(h, 0.0, l);
                    Ok(Some(Value::Color(Color::with_rgb(
                        new_c.legacy_rgb[0],
                        new_c.legacy_rgb[1],
                        new_c.legacy_rgb[2],
                        c.a,
                        ColorSpace::Rgb,
                        ColorOutput::Auto,
                    ))))
                }
                _ if !args.is_empty() => {
                    let arg_str = args
                        .iter()
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ");
                    Ok(Some(Value::String(format!("grayscale({arg_str})"), false)))
                }
                _ => Err(SassError::Eval("grayscale requires 1 argument".into())),
            }
        }
        "color-channel" => match args {
            [Value::Color(c), Value::String(ch, _)] => match ch.as_str() {
                "red" => Ok(Some(Value::Number(c.legacy_rgb[0], None))),
                "green" => Ok(Some(Value::Number(c.legacy_rgb[1], None))),
                "blue" => Ok(Some(Value::Number(c.legacy_rgb[2], None))),
                "alpha" => Ok(Some(Value::Number(c.a, None))),
                _ => Err(SassError::Eval(format!("Unknown color channel: {ch}"))),
            },
            _ => Err(SassError::Eval(
                "color-channel 需要 (color, channel) 参数".into(),
            )),
        },
        "adjust-color" => super::color_adjust::adjust_color(args, kw_args).map(Some),
        "change-color" => super::color_adjust::change_color(args, kw_args).map(Some),
        "scale-color" => super::color_adjust::scale_color(args, kw_args).map(Some),
        "hwb" => {
            // 合并命名参数到位置参数
            let merged = merge_named_color_args(args, kw_args, &["hue", "whiteness", "blackness", "alpha"]);
            // 展开空格分隔的 List（CSS hwb() 语法：hwb(0deg 30% 40%)）
            let flat = flatten_space_list(&merged);
            match &flat[..] {
                [
                    Value::Number(h, _),
                    Value::Number(w, wu),
                    Value::Number(b, bu),
                ] => {
    match wu.as_deref() != Some("%") {
        true => return Err(SassError::Eval(format!(
            "Expected whiteness to have unit \"%\", was {w}"
        ))),
        false => {}
    }
    match bu.as_deref() != Some("%") {
        true => return Err(SassError::Eval(format!(
            "Expected blackness to have unit \"%\", was {b}"
        ))),
        false => {}
    }
                    let c = Evaluator::hwb_to_rgb(*h, *w / 100.0, *b / 100.0, 1.0);
                    Ok(Some(Value::Color(Color::with_hwb(
                        *h,
                        *w / 100.0,
                        *b / 100.0,
                        1.0,
                        c.legacy_rgb,
                    ))))
                }
                // none 参数处理：hwb(none none none) → 创建带 NaN 的颜色
                [a, b, c] if is_none_str(a) || is_none_str(b) || is_none_str(c) => {
                    let h = match a {
                        Value::Number(n, _) => *n,
                        _ => f64::NAN,
                    };
                    let w = match b {
                        Value::Number(n, Some(u)) if u == "%" => n / 100.0,
                        Value::Number(n, _) => *n,
                        _ => f64::NAN,
                    };
                    let bk = match c {
                        Value::Number(n, Some(u)) if u == "%" => n / 100.0,
                        Value::Number(n, _) => *n,
                        _ => f64::NAN,
                    };
                    Ok(Some(Value::Color(Color::with_hwb(
                        h,
                        w,
                        bk,
                        1.0,
                        [0.0, 0.0, 0.0],
                    ))))
                }
                [
                    Value::Number(h, _),
                    Value::Number(w, wu),
                    Value::Number(b, bu),
                    Value::Number(a, au),
                ] => {
    match wu.as_deref() != Some("%") {
        true => return Err(SassError::Eval(format!(
            "Expected whiteness to have unit \"%\", was {w}"
        ))),
        false => {}
    }
    match bu.as_deref() != Some("%") {
        true => return Err(SassError::Eval(format!(
            "Expected blackness to have unit \"%\", was {b}"
        ))),
        false => {}
    }
                    match au.is_some() && au.as_deref() != Some("%") {
                        true => return Err(SassError::Eval(format!(
                            "Expected alpha to have unit \"%\" or no units, was {a}"
                        ))),
                        false => {}
                    }
                    let c = Evaluator::hwb_to_rgb(*h, *w / 100.0, *b / 100.0, *a);
                    Ok(Some(Value::Color(Color::with_hwb(
                        *h,
                        *w / 100.0,
                        *b / 100.0,
                        *a,
                        c.legacy_rgb,
                    ))))
                }
                // CSS 透传：参数包含 none/var()/calc() 等非数值时，原样输出
                _ if flat.iter().any(|a| !matches!(a, Value::Number(_, _))) => {
                    let arg_strs: Vec<String> = flat
                        .iter()
                        .enumerate()
                        .map(|(i, a)| if i == 0 { format_hue(a) } else { a.to_string() })
                        .collect();
                    Ok(Some(Value::String(
                        format!("hwb({})", arg_strs.join(" ")),
                        false,
                    )))
                }
                _ => Err(SassError::Eval("hwb requires 3-4 arguments".into())),
            }
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
            let is_space = matches!(args.first(), Some(Value::List(_, Separator::Space, false)));
            let flat = flatten_space_list(&merge_named_color_args(args, kw_args, &["hue", "saturation", "lightness", "alpha"]));
            match &flat[..] {
                [Value::Number(h, _), Value::Number(s, _), Value::Number(l, _)] => {
                    let c = Evaluator::hsl_to_rgb(*h, *s / 100.0, *l / 100.0);
                    Ok(Some(Value::Color(Color::with_hsl(*h, *s / 100.0, *l / 100.0, 1.0, ColorOutput::Auto, c.legacy_rgb))))
                }
                [Value::Number(h, _), Value::Number(s, _), Value::Number(l, _), Value::Number(a, _)] => {
                    let c = Evaluator::hsl_to_rgb(*h, *s / 100.0, *l / 100.0);
                    Ok(Some(Value::Color(Color::with_hsl(*h, *s / 100.0, *l / 100.0, *a, ColorOutput::Auto, c.legacy_rgb))))
                }
                // CSS Color 4 missing channels: hsl(none 50% 50%) → Color with NaN channels
                chans if chans.iter().any(|a| matches!(a, Value::String(s, false) if s == "none")) => {
                    let c: Vec<f64> = chans.iter().map(|v| extract_none_num(v).unwrap_or(f64::NAN)).collect();
                    let rgb = Evaluator::hsl_to_rgb(c[0], c[1] / 100.0, c[2] / 100.0);
                    Ok(Some(Value::Color(Color::with_hsl(c[0], c[1] / 100.0, c[2] / 100.0, c.get(3).copied().unwrap_or(100.0) / 100.0, ColorOutput::Auto, rgb.legacy_rgb))))
                }
                // CSS 透传：参数包含 var()/calc() 等非数值时，原样输出字符串
                chans if chans.iter().any(|a| !matches!(a, Value::Number(_, _))) => {
                    let sep = if is_space { " " } else { ", " };
                    let arg_strs: Vec<String> = chans.iter().enumerate().map(|(i, a)| if i == 0 { format_hue(a) } else { a.to_string() }).collect();
                    Ok(Some(Value::String(format!("hsl({})", arg_strs.join(sep)), false)))
                }
                _ => Err(SassError::Eval("hsl requires 3-4 arguments".into())),
            }
        }
        "hsla" => {
            let is_space = matches!(args.first(), Some(Value::List(_, Separator::Space, false)));
            let flat = flatten_space_list(&merge_named_color_args(args, kw_args, &["hue", "saturation", "lightness", "alpha"]));
            match &flat[..] {
                [Value::Number(h, _), Value::Number(s, _), Value::Number(l, _), Value::Number(a, _)] => {
                    let c = Evaluator::hsl_to_rgb(*h, *s / 100.0, *l / 100.0);
                    Ok(Some(Value::Color(Color::with_hsl(*h, *s / 100.0, *l / 100.0, *a, ColorOutput::Auto, c.legacy_rgb))))
                }
                // CSS Color 4 missing channels: hsla(none 50% 50% / 0.5) → Color with NaN
                chans if chans.iter().any(|a| matches!(a, Value::String(s, false) if s == "none")) => {
                    let c: Vec<f64> = chans.iter().map(|v| extract_none_num(v).unwrap_or(f64::NAN)).collect();
                    let rgb = Evaluator::hsl_to_rgb(c[0], c[1] / 100.0, c[2] / 100.0);
                    Ok(Some(Value::Color(Color::with_hsl(c[0], c[1] / 100.0, c[2] / 100.0, c.get(3).copied().unwrap_or(f64::NAN) / 100.0, ColorOutput::Auto, rgb.legacy_rgb))))
                }
                // CSS 透传
                chans if chans.iter().any(|a| !matches!(a, Value::Number(_, _))) => {
                    let sep = if is_space { " " } else { ", " };
                    let arg_strs: Vec<String> = chans.iter().enumerate().map(|(i, a)| if i == 0 { format_hue(a) } else { a.to_string() }).collect();
                    Ok(Some(Value::String(format!("hsla({})", arg_strs.join(sep)), false)))
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
        // ── sass:color 模块函数（Level 4 颜色空间支持）──
        "is-powerless" | "is-missing" | "is-in-gamut" | "is-legacy" => {
            super::color_inspect::call(name, args, kw_args)
        }
        "channel" => super::color_space::channel(args, kw_args),
        "to-space" => super::color_space::to_space(args, kw_args),
        "to-gamut" => super::color_gamut::to_gamut(args, kw_args),
        "space" => super::color_space::space(args, kw_args),
        "same" => super::color_space::same(args, kw_args),
        "ie-hex-str" => {
            let color_arg = args.first().or_else(|| kw_args.get("color"));
            match color_arg {
                Some(Value::Color(c)) => {
                    let alpha = (c.a * 255.0).round() as u8;
                    let r = c.legacy_rgb[0] as u8;
                    let g = c.legacy_rgb[1] as u8;
                    let b = c.legacy_rgb[2] as u8;
                    Ok(Some(Value::String(
                        format!("#{alpha:02X}{r:02X}{g:02X}{b:02X}"),
                        false,
                    )))
                }
                Some(v) => Err(SassError::Eval(format!("$color: {v} is not a color."))),
                None => Err(SassError::Eval("Missing argument $color.".into())),
            }
        }
        _ => Ok(None),
    }
}
