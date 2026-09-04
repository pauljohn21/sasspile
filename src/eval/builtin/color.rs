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
use crate::parse::ast::{Color, ColorOutput, Separator, Value};
use std::collections::HashMap;

/// 展开空格分隔的 List 参数——用于 color.hsl(0 100% 50%) 等 CSS Level 4 语法。
/// 当参数只有一个且为 space-separated list 时，展开为独立参数。
/// 也支持 List + alpha 参数的情况（如 hsl(0 100% 50% / 0.5)）。
/// 同时处理 `SlashLiteral` 分隔（声明值中 / 被解析为 `SlashLiteral`）。
fn flatten_space_list(args: &[Value]) -> Vec<Value> {
    if let Some(Value::List(items, Separator::Space, false)) = args.first() {
        if args.len() == 1 {
            return items.clone();
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
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            match color_arg {
                Some(Value::Color(c)) => {
                    let (h, s, l) =
                        Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
                    let new_h = (h + 180.0).rem_euclid(360.0);
                    let new_c = Evaluator::hsl_to_rgb(new_h, s, l);
                    Ok(Some(Value::Color(Color::with_hsl(
                        new_h,
                        s,
                        l,
                        c.a,
                        ColorOutput::RgbPercent,
                        new_c.legacy_rgb,
                    ))))
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
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            match color_arg {
                Some(Value::Color(c)) => {
                    let (h, _s, l) =
                        Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
                    let new_c = Evaluator::hsl_to_rgb(h, 0.0, l);
                    Ok(Some(Value::Color(Color::with_hsl(
                        h,
                        0.0,
                        l,
                        c.a,
                        ColorOutput::RgbPercent,
                        new_c.legacy_rgb,
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
            // 展开空格分隔的 List（CSS hwb() 语法：hwb(0deg 30% 40%)）
            let flat = flatten_space_list(args);
            match &flat[..] {
                [
                    Value::Number(h, _),
                    Value::Number(w, wu),
                    Value::Number(b, bu),
                ] => {
                    if wu.as_deref() != Some("%") {
                        return Err(SassError::Eval(format!(
                            "Expected whiteness to have unit \"%\", was {w}"
                        )));
                    }
                    if bu.as_deref() != Some("%") {
                        return Err(SassError::Eval(format!(
                            "Expected blackness to have unit \"%\", was {b}"
                        )));
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
                [
                    Value::Number(h, _),
                    Value::Number(w, wu),
                    Value::Number(b, bu),
                    Value::Number(a, au),
                ] => {
                    if wu.as_deref() != Some("%") {
                        return Err(SassError::Eval(format!(
                            "Expected whiteness to have unit \"%\", was {w}"
                        )));
                    }
                    if bu.as_deref() != Some("%") {
                        return Err(SassError::Eval(format!(
                            "Expected blackness to have unit \"%\", was {b}"
                        )));
                    }
                    if au.is_some() && au.as_deref() != Some("%") {
                        return Err(SassError::Eval(format!(
                            "Expected alpha to have unit \"%\" or no units, was {a}"
                        )));
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
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
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
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
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
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            match color_arg {
                Some(Value::Color(c)) => {
                    let (h, s, l) =
                        Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
                    let new_h = (h + 180.0).rem_euclid(360.0);
                    let new_c = Evaluator::hsl_to_rgb(new_h, s, l);
                    Ok(Some(Value::Color(Color::with_hsl(
                        new_h,
                        s,
                        l,
                        c.a,
                        ColorOutput::RgbPercent,
                        new_c.legacy_rgb,
                    ))))
                }
                _ => Err(SassError::Eval(
                    "complement requires 1 color argument".into(),
                )),
            }
        }
        "hsl" => {
            let is_space = matches!(args.first(), Some(Value::List(_, Separator::Space, false)));
            let flat = flatten_space_list(args);
            // 检测是否有 none 参数
            let has_none = flat
                .iter()
                .any(|a| matches!(a, Value::String(s, false) if s == "none"));
            match &flat[..] {
                [
                    Value::Number(h, _),
                    Value::Number(s, _),
                    Value::Number(l, _),
                ] => {
                    let c = Evaluator::hsl_to_rgb(*h, *s / 100.0, *l / 100.0);
                    Ok(Some(Value::Color(Color::with_hsl(
                        *h,
                        *s / 100.0,
                        *l / 100.0,
                        1.0,
                        ColorOutput::Auto,
                        c.legacy_rgb,
                    ))))
                }
                [
                    Value::Number(h, _),
                    Value::Number(s, _),
                    Value::Number(l, _),
                    Value::Number(a, _),
                ] => {
                    let c = Evaluator::hsl_to_rgb(*h, *s / 100.0, *l / 100.0);
                    Ok(Some(Value::Color(Color::with_hsl(
                        *h,
                        *s / 100.0,
                        *l / 100.0,
                        *a,
                        ColorOutput::Auto,
                        c.legacy_rgb,
                    ))))
                }
                // CSS 透传：参数包含 none/var()/calc() 等非数值时，原样输出
                _ if has_none || flat.iter().any(|a| !matches!(a, Value::Number(_, _))) => {
                    let sep = if is_space { " " } else { ", " };
                    let arg_strs: Vec<String> = flat
                        .iter()
                        .enumerate()
                        .map(|(i, a)| if i == 0 { format_hue(a) } else { a.to_string() })
                        .collect();
                    Ok(Some(Value::String(
                        format!("hsl({})", arg_strs.join(sep)),
                        false,
                    )))
                }
                _ => Err(SassError::Eval("hsl requires 3-4 arguments".into())),
            }
        }
        "hsla" => {
            let is_space = matches!(args.first(), Some(Value::List(_, Separator::Space, false)));
            let flat = flatten_space_list(args);
            // 检测是否有 none 参数
            let has_none = flat
                .iter()
                .any(|a| matches!(a, Value::String(s, false) if s == "none"));
            match &flat[..] {
                [
                    Value::Number(h, _),
                    Value::Number(s, _),
                    Value::Number(l, _),
                    Value::Number(a, _),
                ] => {
                    let c = Evaluator::hsl_to_rgb(*h, *s / 100.0, *l / 100.0);
                    Ok(Some(Value::Color(Color::with_hsl(
                        *h,
                        *s / 100.0,
                        *l / 100.0,
                        *a,
                        ColorOutput::Auto,
                        c.legacy_rgb,
                    ))))
                }
                // CSS 透传
                _ if has_none || flat.iter().any(|a| !matches!(a, Value::Number(_, _))) => {
                    let sep = if is_space { " " } else { ", " };
                    let arg_strs: Vec<String> = flat
                        .iter()
                        .enumerate()
                        .map(|(i, a)| if i == 0 { format_hue(a) } else { a.to_string() })
                        .collect();
                    Ok(Some(Value::String(
                        format!("hsla({})", arg_strs.join(sep)),
                        false,
                    )))
                }
                _ => Err(SassError::Eval("hsla requires 4 arguments".into())),
            }
        }
        "adjust-hue" => {
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            let deg_arg = args
                .get(1)
                .or_else(|| kw_args.get("$degrees"))
                .or_else(|| kw_args.get("$hue"));
            match (color_arg, deg_arg) {
                (Some(Value::Color(c)), Some(Value::Number(deg, _))) => {
                    let (h, s, l) =
                        Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
                    let new_h = (h + *deg).rem_euclid(360.0);
                    let new_c = Evaluator::hsl_to_rgb(new_h, s, l);
                    Ok(Some(Value::Color(Color::with_hsl(
                        new_h,
                        s,
                        l,
                        c.a,
                        ColorOutput::RgbPercent,
                        new_c.legacy_rgb,
                    ))))
                }
                _ => Err(SassError::Eval(
                    "adjust-hue 需要 (color, degrees) 参数".into(),
                )),
            }
        }
        "saturate" => {
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            let amount_arg = args.get(1).or_else(|| kw_args.get("$amount"));
            match (color_arg, amount_arg) {
                (Some(Value::Color(c)), Some(Value::Number(amount, _))) => {
                    let (h, s, l) =
                        Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
                    let new_s = (s + *amount / 100.0).min(1.0);
                    let new_c = Evaluator::hsl_to_rgb(h, new_s, l);
                    Ok(Some(Value::Color(Color::with_hsl(
                        h,
                        new_s,
                        l,
                        c.a,
                        ColorOutput::RgbPercent,
                        new_c.legacy_rgb,
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
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            let amount_arg = args.get(1).or_else(|| kw_args.get("$amount"));
            match (color_arg, amount_arg) {
                (Some(Value::Color(c)), Some(Value::Number(amount, _))) => {
                    let (h, s, l) =
                        Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
                    let new_s = (s - *amount / 100.0).max(0.0);
                    let new_c = Evaluator::hsl_to_rgb(h, new_s, l);
                    Ok(Some(Value::Color(Color::with_hsl(
                        h,
                        new_s,
                        l,
                        c.a,
                        ColorOutput::RgbPercent,
                        new_c.legacy_rgb,
                    ))))
                }
                _ => Err(SassError::Eval(
                    "desaturate requires (color, amount) arguments".into(),
                )),
            }
        }
        "transparentize" | "fade-out" => {
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            let amount_arg = args.get(1).or_else(|| kw_args.get("$amount"));
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
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            let amount_arg = args.get(1).or_else(|| kw_args.get("$amount"));
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
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
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
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            match color_arg {
                Some(Value::Color(c)) => Ok(Some(Value::Number(c.legacy_rgb[0], None))),
                _ => Err(SassError::Eval("red requires 1 color argument".into())),
            }
        }
        "green" => {
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            match color_arg {
                Some(Value::Color(c)) => Ok(Some(Value::Number(c.legacy_rgb[1], None))),
                _ => Err(SassError::Eval("green requires 1 color argument".into())),
            }
        }
        "blue" => {
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            match color_arg {
                Some(Value::Color(c)) => Ok(Some(Value::Number(c.legacy_rgb[2], None))),
                _ => Err(SassError::Eval("blue requires 1 color argument".into())),
            }
        }
        "hue" => {
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
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
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
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
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
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
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
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
