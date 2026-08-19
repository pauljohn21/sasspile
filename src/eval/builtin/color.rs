//! Color 内建函数（match arms 提取）。
//!
//! 包含 invert/grayscale/color-channel/hwb/complement/hsl/hsla/adjust-hue/
//! saturate/desaturate/transparentize/opacify/alpha/red/green/blue/hue/saturation/lightness。
//! 注意：rgba/rgb/darken/lighten/mix 仍在 builtin.rs 中（调用 Self::builtin_*）。

use super::super::Evaluator;
use crate::error::{Result, SassError};
use crate::parse::ast::*;
use im::HashMap;

/// 展开空格分隔的 List 参数——用于 color.hsl(0 100% 50%) 等 CSS Level 4 语法。
/// 当参数只有一个且为 space-separated list 时，展开为独立参数。
fn flatten_space_list(args: &[Value]) -> Vec<Value> {
    if args.len() == 1
        && let Value::List(items, Separator::Space, false) = &args[0] {
            return items.clone();
        }
    args.to_vec()
}

pub fn call(name: &str, args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Option<Value>> {
    match name {
        "invert" => {
let color_arg = args.first().or_else(|| kw_args.get("$color"));
match color_arg {
Some(Value::Color(c)) => {
                    let (h, s, l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);
                    let new_h = (h + 180.0).rem_euclid(360.0);
                    let new_c = Evaluator::hsl_to_rgb(new_h, s, l);
                    Ok(Some(Value::Color(Color::rgba_fmt(new_c.r, new_c.g, new_c.b, c.a, ColorFormat::RgbPercent(new_h, s, l)))))
                },
// CSS 滤镜函数透传：invert(number) 非颜色参数
_ if !args.is_empty() => {
let arg_str = args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ");
Ok(Some(Value::String(format!("invert({arg_str})"), false)))
},
_ => Err(SassError::Eval("invert requires 1 argument".into())),
}
},
"grayscale" => {
let color_arg = args.first().or_else(|| kw_args.get("$color"));
match color_arg {
Some(Value::Color(c)) => {
                    let (h, _s, l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);
                    let new_c = Evaluator::hsl_to_rgb(h, 0.0, l);
                    Ok(Some(Value::Color(Color::rgba_fmt(new_c.r, new_c.g, new_c.b, c.a, ColorFormat::RgbPercent(h, 0.0, l)))))
                }
_ if !args.is_empty() => {
let arg_str = args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ");
Ok(Some(Value::String(format!("grayscale({arg_str})"), false)))
},
_ => Err(SassError::Eval("grayscale requires 1 argument".into())),
}
},
        "color-channel" => match args {
            [Value::Color(c), Value::String(ch, _)] => match ch.as_str() {
                "red" => Ok(Some(Value::Number(c.r as f64, None))),
                "green" => Ok(Some(Value::Number(c.g as f64, None))),
                "blue" => Ok(Some(Value::Number(c.b as f64, None))),
                "alpha" => Ok(Some(Value::Number(c.a, None))),
                _ => Err(SassError::Eval(format!("Unknown color channel: {ch}"))),
            },
            _ => Err(SassError::Eval(
                "color-channel 需要 (color, channel) 参数".into(),
            )),
        },
        "adjust-color" => {
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            let c = match color_arg {
                Some(Value::Color(c)) => c.clone(),
                _ => return Err(SassError::Eval("adjust-color requires 1 color argument".into())),
            };
            let mut r = c.r as f64;
            let mut g = c.g as f64;
            let mut b = c.b as f64;
            let mut a = c.a;
            let mut has_hsl = false;
            let (mut h, mut s, mut l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);
            let mut has_hwb = false;
            let (mut hw, mut hb) = {
                let r = c.r as f64 / 255.0;
                let g = c.g as f64 / 255.0;
                let b = c.b as f64 / 255.0;
                (r.min(g).min(b), 1.0 - r.max(g).max(b))
            };

            if let Some(v) = kw_args.get("red") {
                if let Value::Number(n, _) = v { r += *n; } else { return Err(SassError::Eval("red requires a number".into())); }
            }
            if let Some(v) = kw_args.get("green") {
                if let Value::Number(n, _) = v { g += *n; } else { return Err(SassError::Eval("green requires a number".into())); }
            }
            if let Some(v) = kw_args.get("blue") {
                if let Value::Number(n, _) = v { b += *n; } else { return Err(SassError::Eval("blue requires a number".into())); }
            }
            if let Some(v) = kw_args.get("alpha") {
                if let Value::Number(n, _) = v { a += *n; } else { return Err(SassError::Eval("alpha requires a number".into())); }
            }
            if let Some(v) = kw_args.get("hue")
                && let Value::Number(n, _) = v { h = (h + *n).rem_euclid(360.0); has_hsl = true; has_hwb = true; }
            if let Some(v) = kw_args.get("saturation")
                && let Value::Number(n, _) = v { s = (s + *n / 100.0).clamp(0.0, 1.0); has_hsl = true; }
            if let Some(v) = kw_args.get("lightness")
                && let Value::Number(n, _) = v { l = (l + *n / 100.0).clamp(0.0, 1.0); has_hsl = true; }
            if let Some(v) = kw_args.get("whiteness")
                && let Value::Number(n, _) = v { hw = (hw + *n / 100.0).clamp(0.0, 1.0); has_hwb = true; }
            if let Some(v) = kw_args.get("blackness")
                && let Value::Number(n, _) = v { hb = (hb + *n / 100.0).clamp(0.0, 1.0); has_hwb = true; }
            if has_hwb {
                let new_c = Evaluator::hwb_to_rgb(h, hw, hb, 1.0);
                r = new_c.r as f64; g = new_c.g as f64; b = new_c.b as f64;
            } else if has_hsl {
                let new_c = Evaluator::hsl_to_rgb(h, s, l);
                r = new_c.r as f64; g = new_c.g as f64; b = new_c.b as f64;
            }
            // HSL/HWB 转换后用百分比输出，纯 RGB 调整保持原格式
            let fmt = if has_hsl || has_hwb { ColorFormat::RgbPercent(h, s, l) } else { ColorFormat::Auto };
            Ok(Some(Value::Color(Color::rgba_fmt(
                r.round().clamp(0.0, 255.0) as u8,
                g.round().clamp(0.0, 255.0) as u8,
                b.round().clamp(0.0, 255.0) as u8,
                a.clamp(0.0, 1.0),
                fmt,
            ))))
        }
        "change-color" => {
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            let c = match color_arg {
                Some(Value::Color(c)) => c.clone(),
                _ => return Err(SassError::Eval("change-color requires 1 color argument".into())),
            };
            let mut r = c.r as f64;
            let mut g = c.g as f64;
            let mut b = c.b as f64;
            let mut a = c.a;
            let mut has_hsl = false;
            let (mut h, mut s, mut l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);
            let mut has_hwb = false;
            let (mut hw, mut hb) = {
                let r = c.r as f64 / 255.0;
                let g = c.g as f64 / 255.0;
                let b = c.b as f64 / 255.0;
                (r.min(g).min(b), 1.0 - r.max(g).max(b))
            };

            if let Some(v) = kw_args.get("red")
                && let Value::Number(n, _) = v { r = *n; }
            if let Some(v) = kw_args.get("green")
                && let Value::Number(n, _) = v { g = *n; }
            if let Some(v) = kw_args.get("blue")
                && let Value::Number(n, _) = v { b = *n; }
            if let Some(v) = kw_args.get("alpha")
                && let Value::Number(n, _) = v { a = *n; }
            if let Some(v) = kw_args.get("hue")
                && let Value::Number(n, _) = v { h = (*n).rem_euclid(360.0); has_hsl = true; has_hwb = true; }
            if let Some(v) = kw_args.get("saturation")
                && let Value::Number(n, _) = v { s = (*n / 100.0).clamp(0.0, 1.0); has_hsl = true; }
            if let Some(v) = kw_args.get("lightness")
                && let Value::Number(n, _) = v { l = (*n / 100.0).clamp(0.0, 1.0); has_hsl = true; }
            if let Some(v) = kw_args.get("whiteness")
                && let Value::Number(n, _) = v { hw = (*n / 100.0).clamp(0.0, 1.0); has_hwb = true; }
            if let Some(v) = kw_args.get("blackness")
                && let Value::Number(n, _) = v { hb = (*n / 100.0).clamp(0.0, 1.0); has_hwb = true; }
            if has_hwb {
                let new_c = Evaluator::hwb_to_rgb(h, hw, hb, 1.0);
                r = new_c.r as f64; g = new_c.g as f64; b = new_c.b as f64;
            } else if has_hsl {
                let new_c = Evaluator::hsl_to_rgb(h, s, l);
                r = new_c.r as f64; g = new_c.g as f64; b = new_c.b as f64;
            }
            // HSL/HWB 转换后用百分比输出，纯 RGB 调整保持原格式
            let fmt = if has_hsl || has_hwb { ColorFormat::RgbPercent(h, s, l) } else { ColorFormat::Auto };
            Ok(Some(Value::Color(Color::rgba_fmt(
                r.round().clamp(0.0, 255.0) as u8,
                g.round().clamp(0.0, 255.0) as u8,
                b.round().clamp(0.0, 255.0) as u8,
                a.clamp(0.0, 1.0),
                fmt,
            ))))
        }
        "scale-color" => {
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            let c = match color_arg {
                Some(Value::Color(c)) => c.clone(),
                _ => return Err(SassError::Eval("scale-color requires 1 color argument".into())),
            };
            let mut r = c.r as f64;
            let mut g = c.g as f64;
            let mut b = c.b as f64;
            let mut a = c.a;
            let mut has_hsl = false;
            let (h, mut s, mut l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);

            let scale_fn = |val: f64, max: f64, kw: &str, kw_args: &HashMap<String, Value>| -> Result<f64> {
                if let Some(Value::Number(n, _)) = kw_args.get(kw) {
                    let pct = *n / 100.0;
                    if pct >= 0.0 {
                        Ok(val + (max - val) * pct)
                    } else {
                        Ok(val + val * pct)
                    }
                } else {
                    Ok(val)
                }
            };
            r = scale_fn(r, 255.0, "red", kw_args)?;
            g = scale_fn(g, 255.0, "green", kw_args)?;
            b = scale_fn(b, 255.0, "blue", kw_args)?;
            a = scale_fn(a, 1.0, "alpha", kw_args)?;
            if kw_args.contains_key("hue") {
                // hue 的 scale 是调整角度，但 Sass 中 hue 不支持 scale（会报错）
                // 这里简单处理：忽略 hue scale
            }
            if kw_args.contains_key("saturation") {
                s = scale_fn(s * 100.0, 100.0, "saturation", kw_args)? / 100.0;
                s = s.clamp(0.0, 1.0);
                has_hsl = true;
            }
            if kw_args.contains_key("lightness") {
                l = scale_fn(l * 100.0, 100.0, "lightness", kw_args)? / 100.0;
                l = l.clamp(0.0, 1.0);
                has_hsl = true;
            }
            if has_hsl {
                let new_c = Evaluator::hsl_to_rgb(h, s, l);
                r = new_c.r as f64; g = new_c.g as f64; b = new_c.b as f64;
            }
            let fmt = if has_hsl { ColorFormat::RgbPercent(h, s, l) } else { ColorFormat::Auto };
            Ok(Some(Value::Color(Color::rgba_fmt(
                r.round().clamp(0.0, 255.0) as u8,
                g.round().clamp(0.0, 255.0) as u8,
                b.round().clamp(0.0, 255.0) as u8,
                a.clamp(0.0, 1.0),
                fmt,
            ))))
        }
        "hwb" => {
            // 展开空格分隔的 List（CSS hwb() 语法：hwb(0deg 30% 40%)）
            let flat = flatten_space_list(args);
            match &flat[..] {
                [Value::Number(h, _), Value::Number(w, wu), Value::Number(b, bu)] => {
                    if wu.as_deref() != Some("%") {
                        return Err(SassError::Eval(
                            format!("Expected whiteness to have unit \"%\", was {}", w),
                        ));
                    }
                    if bu.as_deref() != Some("%") {
                        return Err(SassError::Eval(
                            format!("Expected blackness to have unit \"%\", was {}", b),
                        ));
                    }
                    let c = Evaluator::hwb_to_rgb(*h, *w / 100.0, *b / 100.0, 1.0);
                    Ok(Some(Value::Color(Color::rgba_fmt(c.r, c.g, c.b, 1.0, ColorFormat::Hwb(*h, *w / 100.0, *b / 100.0)))))
                }
                [Value::Number(h, _), Value::Number(w, wu), Value::Number(b, bu), Value::Number(a, au)] => {
                    if wu.as_deref() != Some("%") {
                        return Err(SassError::Eval(
                            format!("Expected whiteness to have unit \"%\", was {}", w),
                        ));
                    }
                    if bu.as_deref() != Some("%") {
                        return Err(SassError::Eval(
                            format!("Expected blackness to have unit \"%\", was {}", b),
                        ));
                    }
                    if au.is_some() && au.as_deref() != Some("%") {
                        return Err(SassError::Eval(
                            format!("Expected alpha to have unit \"%\" or no units, was {}", a),
                        ));
                    }
                    let c = Evaluator::hwb_to_rgb(*h, *w / 100.0, *b / 100.0, *a);
                    Ok(Some(Value::Color(Color::rgba_fmt(c.r, c.g, c.b, *a, ColorFormat::Hwb(*h, *w / 100.0, *b / 100.0)))))
                }
                // CSS 透传：参数包含 var()/calc() 等非数值时，原样输出
                _ if flat.iter().any(|a| !matches!(a, Value::Number(_, _))) => {
                    let arg_str = flat.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(" ");
                    Ok(Some(Value::String(format!("hwb({arg_str})"), false)))
                }
                _ => Err(SassError::Eval("hwb requires 3-4 arguments".into())),
            }
        }
        "whiteness" => {
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            match color_arg {
                Some(Value::Color(c)) => {
                    let w = c.r.min(c.g).min(c.b) as f64 / 255.0 * 100.0;
                    Ok(Some(Value::Number(w, Some("%".to_string()))))
                }
                _ => Err(SassError::Eval("whiteness requires 1 color argument".into())),
            }
        }
        "blackness" => {
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            match color_arg {
                Some(Value::Color(c)) => {
                    let b = (1.0 - c.r.max(c.g).max(c.b) as f64 / 255.0) * 100.0;
                    Ok(Some(Value::Number(b, Some("%".to_string()))))
                }
                _ => Err(SassError::Eval("blackness requires 1 color argument".into())),
            }
        }
        "complement" => {
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            match color_arg {
                Some(Value::Color(c)) => {
                    let (h, s, l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);
                    let new_h = (h + 180.0).rem_euclid(360.0);
                    let new_c = Evaluator::hsl_to_rgb(new_h, s, l);
                    Ok(Some(Value::Color(Color::rgba_fmt(new_c.r, new_c.g, new_c.b, c.a, ColorFormat::RgbPercent(new_h, s, l)))))
                }
                _ => Err(SassError::Eval("complement requires 1 color argument".into())),
            }
        }
        "hsl" => {
            let flat = flatten_space_list(args);
            match &flat[..] {
            [
                Value::Number(h, _),
                Value::Number(s, _),
                Value::Number(l, _),
            ] => {
                let c = Evaluator::hsl_to_rgb(*h, *s / 100.0, *l / 100.0);
                Ok(Some(Value::Color(Color::rgba_fmt(c.r, c.g, c.b, 1.0, ColorFormat::Hsl(*h, *s / 100.0, *l / 100.0)))))
            }
            [
                Value::Number(h, _),
                Value::Number(s, _),
                Value::Number(l, _),
                Value::Number(a, _),
            ] => {
                let c = Evaluator::hsl_to_rgb(*h, *s / 100.0, *l / 100.0);
                Ok(Some(Value::Color(Color::rgba_fmt(c.r, c.g, c.b, *a, ColorFormat::Hsl(*h, *s / 100.0, *l / 100.0)))))
            }
            // CSS 透传：参数包含 var()/calc() 等非数值时，原样输出
            _ if flat.iter().any(|a| !matches!(a, Value::Number(_, _))) => {
                let arg_str = flat.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ");
                Ok(Some(Value::String(format!("hsl({arg_str})"), false)))
            }
            _ => Err(SassError::Eval("hsl requires 3-4 arguments".into())),
            }
        },
        "hsla" => {
            let flat = flatten_space_list(args);
            match &flat[..] {
            [
                Value::Number(h, _),
                Value::Number(s, _),
                Value::Number(l, _),
                Value::Number(a, _),
            ] => {
                let c = Evaluator::hsl_to_rgb(*h, *s / 100.0, *l / 100.0);
                Ok(Some(Value::Color(Color::rgba_fmt(c.r, c.g, c.b, *a, ColorFormat::Hsl(*h, *s / 100.0, *l / 100.0)))))
            }
            // CSS 透传
            _ if flat.iter().any(|a| !matches!(a, Value::Number(_, _))) => {
                let arg_str = flat.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ");
                Ok(Some(Value::String(format!("hsla({arg_str})"), false)))
            }
            _ => Err(SassError::Eval("hsla requires 4 arguments".into())),
            }
        },
        "adjust-hue" => {
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            let deg_arg = args.get(1).or_else(|| kw_args.get("$degrees")).or_else(|| kw_args.get("$hue"));
            match (color_arg, deg_arg) {
                (Some(Value::Color(c)), Some(Value::Number(deg, _))) => {
                    let (h, s, l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);
                    let new_h = (h + *deg).rem_euclid(360.0);
                    let new_c = Evaluator::hsl_to_rgb(new_h, s, l);
                    Ok(Some(Value::Color(Color::rgba_fmt(new_c.r, new_c.g, new_c.b, c.a, ColorFormat::RgbPercent(new_h, s, l)))))
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
                    let (h, s, l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);
                    let new_s = (s + *amount / 100.0).min(1.0);
                    let new_c = Evaluator::hsl_to_rgb(h, new_s, l);
                    Ok(Some(Value::Color(Color::rgba_fmt(new_c.r, new_c.g, new_c.b, c.a, ColorFormat::RgbPercent(h, new_s, l)))))
                }
                // CSS 滤镜函数透传：saturate(number)
                (Some(Value::Number(n, _)), None) => Ok(Some(Value::String(format!("saturate({n})"), false))),
                _ => Err(SassError::Eval("saturate requires (color, amount) arguments".into())),
            }
        }
        "desaturate" => {
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            let amount_arg = args.get(1).or_else(|| kw_args.get("$amount"));
            match (color_arg, amount_arg) {
                (Some(Value::Color(c)), Some(Value::Number(amount, _))) => {
                    let (h, s, l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);
                    let new_s = (s - *amount / 100.0).max(0.0);
                    let new_c = Evaluator::hsl_to_rgb(h, new_s, l);
                    Ok(Some(Value::Color(Color::rgba_fmt(new_c.r, new_c.g, new_c.b, c.a, ColorFormat::RgbPercent(h, new_s, l)))))
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
                (Some(Value::Color(c)), Some(Value::Number(amount, _))) => Ok(Some(Value::Color(Color::rgba(
                    c.r,
                    c.g,
                    c.b,
                    (c.a - *amount).max(0.0),
                )))),
                _ => Err(SassError::Eval(
                    "transparentize requires (color, amount) arguments".into(),
                )),
            }
        }
        "opacify" | "fade-in" => {
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            let amount_arg = args.get(1).or_else(|| kw_args.get("$amount"));
            match (color_arg, amount_arg) {
                (Some(Value::Color(c)), Some(Value::Number(amount, _))) => Ok(Some(Value::Color(Color::rgba(
                    c.r,
                    c.g,
                    c.b,
                    (c.a + *amount).min(1.0),
                )))),
                _ => Err(SassError::Eval("opacify requires (color, amount) arguments".into())),
            }
        }
        "alpha" | "opacity" => {
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            match color_arg {
                Some(Value::Color(c)) => Ok(Some(Value::Number(c.a, None))),
                _ => {
                    // CSS 透传：旧 IE filter 语法 alpha(opacity=0) — 关键字参数直接透传
                    if !kw_args.is_empty() {
                        let kw_str = kw_args.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(", ");
                        Ok(Some(Value::String(format!("{name}({kw_str})"), false)))
                    } else if !args.is_empty() {
                        // CSS 透传：非颜色位置参数原样输出（如 alpha(var(--x))）
                        let arg_str = args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ");
                        Ok(Some(Value::String(format!("{name}({arg_str})"), false)))
                    } else {
                        Err(SassError::Eval("alpha requires 1 color argument".into()))
                    }
                }
            }
        }
        "red" => {
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            match color_arg {
                Some(Value::Color(c)) => Ok(Some(Value::Number(c.r as f64, None))),
                _ => Err(SassError::Eval("red requires 1 color argument".into())),
            }
        }
        "green" => {
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            match color_arg {
                Some(Value::Color(c)) => Ok(Some(Value::Number(c.g as f64, None))),
                _ => Err(SassError::Eval("green requires 1 color argument".into())),
            }
        }
        "blue" => {
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            match color_arg {
                Some(Value::Color(c)) => Ok(Some(Value::Number(c.b as f64, None))),
                _ => Err(SassError::Eval("blue requires 1 color argument".into())),
            }
        }
        "hue" => {
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            match color_arg {
                Some(Value::Color(c)) => {
                    let (h, _, _) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);
                    Ok(Some(Value::Number(h, Some("deg".into()))))
                }
                _ => Err(SassError::Eval("hue requires 1 color argument".into())),
            }
        }
        "saturation" => {
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            match color_arg {
                Some(Value::Color(c)) => {
                    let (_, s, _) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);
                    Ok(Some(Value::Number(s * 100.0, Some("%".into()))))
                }
                _ => Err(SassError::Eval("saturation requires 1 color argument".into())),
            }
        }
        "lightness" => {
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            match color_arg {
                Some(Value::Color(c)) => {
                    let (_, _, l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);
                    Ok(Some(Value::Number(l * 100.0, Some("%".into()))))
                }
                _ => Err(SassError::Eval("lightness requires 1 color argument".into())),
            }
        }
        // ── sass:color 模块函数（Level 4 颜色空间支持）──
        "is-powerless" => {
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            let channel = args.get(1).or_else(|| kw_args.get("$channel"));
            match (color_arg, channel) {
                (Some(Value::Color(c)), Some(Value::String(ch, _))) => {
                    let powerless = is_channel_powerless(c, ch);
                    Ok(Some(Value::Bool(powerless)))
                }
                // 处理 oklch()/oklab()/lch()/lab() 字符串形式
                (Some(Value::String(s, _)), Some(Value::String(ch, _))) => {
                    let powerless = is_channel_powerless_str(s, ch);
                    match powerless {
                        Some(b) => Ok(Some(Value::Bool(b))),
                        None => Err(SassError::Eval(
                            "is-powerless requires $color and $channel arguments".into(),
                        )),
                    }
                }
                _ => Err(SassError::Eval(
                    "is-powerless requires $color and $channel arguments".into(),
                )),
            }
        }
        "is-missing" => {
            // sasspile 目前不支持 none/missing 通道，所以总是返回 false。
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            let channel_arg = args.get(1).or_else(|| kw_args.get("$channel"));
            match (color_arg, channel_arg) {
                (Some(Value::Color(_)), Some(Value::String(_, _))) => Ok(Some(Value::Bool(false))),
                _ => Err(SassError::Eval("is-missing requires $color and $channel arguments".into())),
            }
        }
        "is-in-gamut" => {
            // sasspile 存储的 sRGB 颜色始终在色域内（u8 范围 + alpha 0-1）
            let color_arg = args.first().or_else(|| kw_args.get("$color"));
            match color_arg {
                Some(Value::Color(_c)) => Ok(Some(Value::Bool(true))),
                _ => Err(SassError::Eval("is-in-gamut requires $color argument".into())),
            }
        }
        "is-legacy" => {
            // sasspile 所有颜色都是 sRGB（legacy 空间）
            Ok(Some(Value::Bool(true)))
        }
        "channel" => super::color_space::channel(args, kw_args),
        "to-space" => super::color_space::to_space(args, kw_args),
        "space" => super::color_space::space(args, kw_args),
        "same" => super::color_space::same(args, kw_args),
        _ => Ok(None),
    }
}

/// 检查颜色通道是否"无效"（powerless）。
/// 参考 dart-sass 实现：
/// - HSL: hue 在 saturation ≈ 0 时无效；saturation 在 lightness = 0%/100% 时无效
///   （注意：CSS 规范已更新，lightness 极端值不再使 hue/saturation 无效）
/// - HWB: hue 在 whiteness + blackness >= 100%（归一化后）时无效
/// - LCH/OKLCH: hue 在 chroma ≈ 0 时无效
fn is_channel_powerless(c: &Color, channel: &str) -> bool {
    let (_h, s, l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);
    let max = c.r.max(c.g).max(c.b) as f64 / 255.0;
    let min = c.r.min(c.g).min(c.b) as f64 / 255.0;
    let w = min;
    let b = 1.0 - max;
    let w_b_sum = (w + b).min(1.0);
    let eps = 0.0001;
    match channel {
        "hue" => s < eps || w_b_sum >= 1.0 - eps,
        "saturation" => l < eps || (1.0 - l) < eps,
        "lightness" => false,
        "whiteness" => false,
        "blackness" => false,
        "a" | "b" => false,
        _ => false,
    }
}

/// 从字符串形式的 Level 4 颜色（如 `oklch(50% 0.1 0deg)`）判断通道是否 powerless。
///
/// 返回 `Some(bool)` 表示成功判断，`None` 表示无法识别。
fn is_channel_powerless_str(color_str: &str, channel: &str) -> Option<bool> {
    let s = color_str.trim();
    let eps = 0.0001;

    // 提取函数名和参数
    let paren_start = s.find('(')?;
    let func_name = s[..paren_start].trim();
    let paren_end = s.rfind(')')?;
    let inner = &s[paren_start + 1..paren_end];

    // 解析空格分隔的参数
    let parts: Vec<&str> = inner.split_whitespace().collect();

    match func_name {
        "oklch" | "lch" => {
            // lch(L C H) / oklch(L C H)
            // hue powerless 当 chroma ≈ 0
            // chroma/lightness 永不 powerless
            match channel {
                "hue" => {
                    // chroma 是第二个参数
                    let chroma_str = parts.get(1)?;
                    let chroma = parse_percent_or_number(chroma_str)?;
                    Some(chroma.abs() < eps)
                }
                "chroma" | "lightness" => Some(false),
                _ => Some(false),
            }
        }
        "oklab" | "lab" => {
            // lab(L A B) / oklab(L A B)
            // a/b 永不 powerless
            match channel {
                "a" | "b" | "lightness" => Some(false),
                _ => Some(false),
            }
        }
        _ => None,
    }
}

/// 解析 `50%` 或 `0.1` 形式的数值。
fn parse_percent_or_number(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.ends_with('%') {
        s[..s.len() - 1].parse::<f64>().ok().map(|v| v / 100.0)
    } else {
        s.parse::<f64>().ok()
    }
}
