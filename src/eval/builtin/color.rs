//! Color 内建函数（match arms 提取）。
//!
//! 包含 invert/grayscale/color-channel/hwb/complement/hsl/hsla/adjust-hue/
//! saturate/desaturate/transparentize/opacify/alpha/red/green/blue/hue/saturation/lightness。
//! 注意：rgba/rgb/darken/lighten/mix 仍在 builtin.rs 中（调用 Self::builtin_*）。

use super::super::Evaluator;
use crate::error::{Result, SassError};
use crate::parse::ast::*;
use im::HashMap;

pub fn call(name: &str, args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Option<Value>> {
    match name {
"invert" => match args {
[Value::Color(c)] => Ok(Some(Value::Color(Color::rgb(
255 - c.r,
255 - c.g,
255 - c.b,
)))),
// CSS 滤镜函数透传：invert(number) 非颜色参数
_ if !args.is_empty() => {
let arg_str = args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ");
Ok(Some(Value::String(format!("invert({arg_str})"), false)))
},
_ => Err(SassError::Eval("invert 需要 1 个参数".into())),
},
"grayscale" => match args {
[Value::Color(c)] => {
let avg = ((c.r as u16 + c.g as u16 + c.b as u16) / 3) as u8;
                Ok(Some(Value::Color(Color::rgba(avg, avg, avg, c.a))))
            }
_ if !args.is_empty() => {
 let arg_str = args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ");
 Ok(Some(Value::String(format!("grayscale({arg_str})"), false)))
 },
 _ => Err(SassError::Eval("grayscale 需要 1 个参数".into())),
 },
        "color-channel" => match args {
            [Value::Color(c), Value::String(ch, _)] => match ch.as_str() {
                "red" => Ok(Some(Value::Number(c.r as f64, None))),
                "green" => Ok(Some(Value::Number(c.g as f64, None))),
                "blue" => Ok(Some(Value::Number(c.b as f64, None))),
                "alpha" => Ok(Some(Value::Number(c.a as f64, None))),
                _ => Err(SassError::Eval(format!("未知颜色通道: {ch}"))),
            },
            _ => Err(SassError::Eval(
                "color-channel 需要 (color, channel) 参数".into(),
            )),
        },
        "adjust-color" => {
            let c = match args.first() {
                Some(Value::Color(c)) => c.clone(),
                _ => return Err(SassError::Eval("adjust-color 需要 1 个颜色参数".into())),
            };
            let mut r = c.r as f64;
            let mut g = c.g as f64;
            let mut b = c.b as f64;
            let mut a = c.a as f64;
            let mut has_hsl = false;
            let (mut h, mut s, mut l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);

            if let Some(v) = kw_args.get("red") {
                if let Value::Number(n, _) = v { r += *n; } else { return Err(SassError::Eval("red 需要数字".into())); }
            }
            if let Some(v) = kw_args.get("green") {
                if let Value::Number(n, _) = v { g += *n; } else { return Err(SassError::Eval("green 需要数字".into())); }
            }
            if let Some(v) = kw_args.get("blue") {
                if let Value::Number(n, _) = v { b += *n; } else { return Err(SassError::Eval("blue 需要数字".into())); }
            }
            if let Some(v) = kw_args.get("alpha") {
                if let Value::Number(n, _) = v { a += *n; } else { return Err(SassError::Eval("alpha 需要数字".into())); }
            }
            if let Some(v) = kw_args.get("hue") {
                if let Value::Number(n, _) = v { h = (h + *n).rem_euclid(360.0); has_hsl = true; }
            }
            if let Some(v) = kw_args.get("saturation") {
                if let Value::Number(n, _) = v { s = (s + *n / 100.0).clamp(0.0, 1.0); has_hsl = true; }
            }
            if let Some(v) = kw_args.get("lightness") {
                if let Value::Number(n, _) = v { l = (l + *n / 100.0).clamp(0.0, 1.0); has_hsl = true; }
            }
            if has_hsl {
                let new_c = Evaluator::hsl_to_rgb(h, s, l);
                r = new_c.r as f64; g = new_c.g as f64; b = new_c.b as f64;
            }
            Ok(Some(Value::Color(Color::rgba(
                r.round().clamp(0.0, 255.0) as u8,
                g.round().clamp(0.0, 255.0) as u8,
                b.round().clamp(0.0, 255.0) as u8,
                a.clamp(0.0, 1.0) as f32,
            ))))
        }
        "change-color" => {
            let c = match args.first() {
                Some(Value::Color(c)) => c.clone(),
                _ => return Err(SassError::Eval("change-color 需要 1 个颜色参数".into())),
            };
            let mut r = c.r as f64;
            let mut g = c.g as f64;
            let mut b = c.b as f64;
            let mut a = c.a as f64;
            let mut has_hsl = false;
            let (mut h, mut s, mut l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);

            if let Some(v) = kw_args.get("red") {
                if let Value::Number(n, _) = v { r = *n; }
            }
            if let Some(v) = kw_args.get("green") {
                if let Value::Number(n, _) = v { g = *n; }
            }
            if let Some(v) = kw_args.get("blue") {
                if let Value::Number(n, _) = v { b = *n; }
            }
            if let Some(v) = kw_args.get("alpha") {
                if let Value::Number(n, _) = v { a = *n; }
            }
            if let Some(v) = kw_args.get("hue") {
                if let Value::Number(n, _) = v { h = (*n).rem_euclid(360.0); has_hsl = true; }
            }
            if let Some(v) = kw_args.get("saturation") {
                if let Value::Number(n, _) = v { s = (*n / 100.0).clamp(0.0, 1.0); has_hsl = true; }
            }
            if let Some(v) = kw_args.get("lightness") {
                if let Value::Number(n, _) = v { l = (*n / 100.0).clamp(0.0, 1.0); has_hsl = true; }
            }
            if has_hsl {
                let new_c = Evaluator::hsl_to_rgb(h, s, l);
                r = new_c.r as f64; g = new_c.g as f64; b = new_c.b as f64;
            }
            Ok(Some(Value::Color(Color::rgba(
                r.round().clamp(0.0, 255.0) as u8,
                g.round().clamp(0.0, 255.0) as u8,
                b.round().clamp(0.0, 255.0) as u8,
                a.clamp(0.0, 1.0) as f32,
            ))))
        }
        "scale-color" => {
            let c = match args.first() {
                Some(Value::Color(c)) => c.clone(),
                _ => return Err(SassError::Eval("scale-color 需要 1 个颜色参数".into())),
            };
            let mut r = c.r as f64;
            let mut g = c.g as f64;
            let mut b = c.b as f64;
            let mut a = c.a as f64;
            let mut has_hsl = false;
            let (mut h, mut s, mut l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);

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
            Ok(Some(Value::Color(Color::rgba(
                r.round().clamp(0.0, 255.0) as u8,
                g.round().clamp(0.0, 255.0) as u8,
                b.round().clamp(0.0, 255.0) as u8,
                a.clamp(0.0, 1.0) as f32,
            ))))
        }
        "hwb" => {
            // 展开空格分隔的 List（CSS hwb() 语法：hwb(0deg 30% 40%)）
            // 仅接受非 bracketed、恰好 3 元素的 space-separated list
            let flat = if args.len() == 1 {
                if let Value::List(items, Separator::Space, false) = &args[0] {
                    if items.len() == 3 {
                        items.clone()
                    } else {
                        return Err(SassError::Eval("hwb 需要 3-4 个参数".into()));
                    }
                } else {
                    return Err(SassError::Eval("hwb 需要 3-4 个参数".into()));
                }
            } else {
                args.to_vec()
            };
            match &flat[..] {
                [Value::Number(h, _), Value::Number(w, wu), Value::Number(b, bu)] => {
                    // whiteness 和 blackness 必须有 % 单位
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
                    Ok(Some(Value::Color(Evaluator::hwb_to_rgb(
                        *h,
                        *w / 100.0,
                        *b / 100.0,
                        1.0,
                    ))))
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
                    Ok(Some(Value::Color(Evaluator::hwb_to_rgb(
                        *h,
                        *w / 100.0,
                        *b / 100.0,
                        *a as f32,
                    ))))
                }
                _ => Err(SassError::Eval("hwb 需要 3-4 个参数".into())),
            }
        }
        "complement" => match args {
            [Value::Color(c)] => Ok(Some(Value::Color(Color::rgb(
                255 - c.r,
                255 - c.g,
                255 - c.b,
            )))),
            _ => Err(SassError::Eval("complement 需要 1 个颜色参数".into())),
        },
        "hsl" => match args {
            [
                Value::Number(h, _),
                Value::Number(s, _),
                Value::Number(l, _),
            ] => Ok(Some(Value::Color(Evaluator::hsl_to_rgb(
                *h,
                *s / 100.0,
                *l / 100.0,
            )))),
            [
                Value::Number(h, _),
                Value::Number(s, _),
                Value::Number(l, _),
                Value::Number(a, _),
            ] => {
                let mut c = Evaluator::hsl_to_rgb(*h, *s / 100.0, *l / 100.0);
                c.a = *a as f32;
                Ok(Some(Value::Color(c)))
            }
            _ => Err(SassError::Eval("hsl 需要 3-4 个参数".into())),
        },
        "hsla" => match args {
            [
                Value::Number(h, _),
                Value::Number(s, _),
                Value::Number(l, _),
                Value::Number(a, _),
            ] => {
                let mut c = Evaluator::hsl_to_rgb(*h, *s / 100.0, *l / 100.0);
                c.a = *a as f32;
                Ok(Some(Value::Color(c)))
            }
            _ => Err(SassError::Eval("hsla 需要 4 个参数".into())),
        },
        "adjust-hue" => match args {
            [Value::Color(c), Value::Number(deg, _)] => {
                let (h, s, l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);
                let new_h = (h + *deg).rem_euclid(360.0);
                Ok(Some(Value::Color(Evaluator::hsl_to_rgb(new_h, s, l))))
            }
            _ => Err(SassError::Eval(
                "adjust-hue 需要 (color, degrees) 参数".into(),
            )),
        },
        "saturate" => match args {
            [Value::Color(c), Value::Number(amount, _)] => {
                let (h, s, l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);
                Ok(Some(Value::Color(Evaluator::hsl_to_rgb(
                    h,
                    (s + *amount / 100.0).min(1.0),
                    l,
                ))))
            }
            [Value::Number(n, _)] => Ok(Some(Value::String(format!("saturate({})", n), false))),
            _ => Err(SassError::Eval("saturate 需要 (color, amount) 参数".into())),
        },
        "desaturate" => match args {
            [Value::Color(c), Value::Number(amount, _)] => {
                let (h, s, l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);
                Ok(Some(Value::Color(Evaluator::hsl_to_rgb(
                    h,
                    (s - *amount / 100.0).max(0.0),
                    l,
                ))))
            }
            _ => Err(SassError::Eval(
                "desaturate 需要 (color, amount) 参数".into(),
            )),
        },
        "transparentize" | "fade-out" => match args {
            [Value::Color(c), Value::Number(amount, _)] => Ok(Some(Value::Color(Color::rgba(
                c.r,
                c.g,
                c.b,
                (c.a - *amount as f32).max(0.0),
            )))),
            _ => Err(SassError::Eval(
                "transparentize 需要 (color, amount) 参数".into(),
            )),
        },
        "opacify" | "fade-in" => match args {
            [Value::Color(c), Value::Number(amount, _)] => Ok(Some(Value::Color(Color::rgba(
                c.r,
                c.g,
                c.b,
                (c.a + *amount as f32).min(1.0),
            )))),
            _ => Err(SassError::Eval("opacify 需要 (color, amount) 参数".into())),
        },
        "alpha" | "opacity" => match args {
            [Value::Color(c)] => Ok(Some(Value::Number(c.a as f64, None))),
            _ => Err(SassError::Eval("alpha 需要 1 个颜色参数".into())),
        },
        "red" => match args {
            [Value::Color(c)] => Ok(Some(Value::Number(c.r as f64, None))),
            _ => Err(SassError::Eval("red 需要 1 个颜色参数".into())),
        },
        "green" => match args {
            [Value::Color(c)] => Ok(Some(Value::Number(c.g as f64, None))),
            _ => Err(SassError::Eval("green 需要 1 个颜色参数".into())),
        },
        "blue" => match args {
            [Value::Color(c)] => Ok(Some(Value::Number(c.b as f64, None))),
            _ => Err(SassError::Eval("blue 需要 1 个颜色参数".into())),
        },
        "hue" => match args {
            [Value::Color(c)] => {
                let (h, _, _) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);
                Ok(Some(Value::Number(h, Some("deg".into()))))
            }
            _ => Err(SassError::Eval("hue 需要 1 个颜色参数".into())),
        },
        "saturation" => match args {
            [Value::Color(c)] => {
                let (_, s, _) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);
                Ok(Some(Value::Number(s * 100.0, Some("%".into()))))
            }
            _ => Err(SassError::Eval("saturation 需要 1 个颜色参数".into())),
        },
        "lightness" => match args {
            [Value::Color(c)] => {
                let (_, _, l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);
                Ok(Some(Value::Number(l * 100.0, Some("%".into()))))
            }
            _ => Err(SassError::Eval("lightness 需要 1 个颜色参数".into())),
        },
        _ => Ok(None),
    }
}
