#![allow(
    clippy::many_single_char_names,
    clippy::single_char_pattern,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::float_cmp,
    clippy::cast_precision_loss
)]
use super::*;
use crate::error::Result;
use crate::parse::ast::{ColorOutput, ColorSpace};

impl Evaluator {
    /// HSL → RGB 转换 (W3C CSS Color 4 算法)。
    pub(crate) fn hsl_to_rgb(h: f64, s: f64, l: f64) -> Color {
        let h = h.rem_euclid(360.0);
        crate::__tracing::trace!(
            target: "sasspile::color",
            fn = "hsl_to_rgb",
            h = h, s = s, l = l,
            "converting HSL to RGB"
        );
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;
        let (r1, g1, b1) = match h {
            h if h < 60.0 => (c, x, 0.0),
            h if h < 120.0 => (x, c, 0.0),
            h if h < 180.0 => (0.0, c, x),
            h if h < 240.0 => (0.0, x, c),
            h if h < 300.0 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        let result = Color::rgb((r1 + m) * 255.0, (g1 + m) * 255.0, (b1 + m) * 255.0);
        crate::__tracing::trace!(
            target: "sasspile::color",
            fn = "hsl_to_rgb",
            r = result.legacy_rgb[0], g = result.legacy_rgb[1], b = result.legacy_rgb[2],
            "HSL to RGB result"
        );
        result
    }

    /// HWB → RGB 转换 (W3C CSS Color 4 算法)。
    pub(crate) fn hwb_to_rgb(h: f64, w: f64, b: f64, alpha: f64) -> Color {
        crate::__tracing::trace!(
            target: "sasspile::color",
            fn = "hwb_to_rgb",
            h = h, w = w, b = b, alpha = alpha,
            "converting HWB to RGB"
        );
        let h = (h % 360.0) / 360.0;
        let mut w = w;
        let mut b = b;
        let sum = w + b;
        match sum > 1.0 {
            true => { w /= sum; b /= sum; }
            false => {}
        }
        let factor = 1.0 - w - b;
        let hue_to_rgb = |m1: f64, m2: f64, mut hue: f64| -> f64 {
            match hue < 0.0 { true => hue += 1.0, false => {} }
            match hue > 1.0 { true => hue -= 1.0, false => {} }
            match hue {
                h if h < 1.0 / 6.0 => m1 + (m2 - m1) * hue * 6.0,
                h if h < 0.5 => m2,
                h if h < 2.0 / 3.0 => m1 + (m2 - m1) * (2.0 / 3.0 - hue) * 6.0,
                _ => m1,
            }
        };
        let to_rgb = |hue: f64| -> f64 { hue_to_rgb(0.0, 1.0, hue) * factor + w };
        let r = to_rgb(h + 1.0 / 3.0);
        let g = to_rgb(h);
        let bl = to_rgb(h - 1.0 / 3.0);
        Color::rgba(r * 255.0, g * 255.0, bl * 255.0, alpha)
    }

    /// RGB → HSL 转换。
    pub(crate) fn rgb_to_hsl(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
        crate::__tracing::trace!(
            target: "sasspile::color",
            fn = "rgb_to_hsl",
            r = r, g = g, b = b,
            "converting RGB to HSL"
        );
        let r = r / 255.0;
        let g = g / 255.0;
        let b = b / 255.0;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = f64::midpoint(max, min);
        match (max - min).abs() < f64::EPSILON {
            true => return (0.0, 0.0, l),
            false => {}
        }
        let d = max - min;
        let s = if l > 0.5 {
            d / (2.0 - max - min)
        } else {
            d / (max + min)
        };
        let h = match max {
            _ if max == r => ((g - b) / d + if g < b { 6.0 } else { 0.0 }) * 60.0,
            _ if max == g => ((b - r) / d + 2.0) * 60.0,
            _ => ((r - g) / d + 4.0) * 60.0,
        };
        let result = (h, s, l);
        crate::__tracing::trace!(
            target: "sasspile::color",
            fn = "rgb_to_hsl",
            h = result.0, s = result.1, l = result.2,
            "RGB to HSL result"
        );
        result
    }

    /// 简单伪随机数——基于系统时间。
    pub(crate) fn simple_random() -> f64 {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let val = (nanos % 1_000_000) as f64;
        val / 1_000_000.0
    }

    pub(crate) fn builtin_rgba(fn_name: &str, args: &[Value]) -> Result<Value> {
        // 检测是否为空格分隔的 CSS Level 4 语法（rgb(R G B) 或 rgb(R G B / A)）
        let is_space_sep = matches!(args.first(), Some(Value::List(_, Separator::Space, false)));
        // 展开空格分隔的 List（CSS Level 4 语法：rgb(R G B / A)）
        // 同时处理 SlashLiteral 分隔的情况（rgb(R G B / A) 中 / 被解析为 SlashLiteral）
        let args: Vec<Value> = match args.first() {
            Some(Value::List(items, Separator::Space, false)) if is_space_sep => {
                let mut flat = items.clone();
                // alpha 参数追加到末尾
                match args.len() > 1 {
                    true => flat.extend(args[1..].iter().cloned()),
                    false => {}
                }
                flat
            }
            Some(Value::List(items, Separator::SlashLiteral | Separator::Slash, false)) => {
                // rgb(R G B / A) — SlashLiteral 分隔的列表
                let mut flat = Vec::new();
                // 第一个元素可能是 Space 分隔的 [R, G, B]
                if let Some(Value::List(rgb_items, Separator::Space, false)) = items.first() {
                    flat.extend(rgb_items.iter().cloned());
                } else {
                    flat.extend(items[..items.len().saturating_sub(1)].iter().cloned());
                }
                // 最后一个元素是 alpha
                match items.len() >= 2 {
                    true => flat.push(items[items.len() - 1].clone()),
                    false => {}
                }
                flat
            }
            _ => args.to_vec(),
        };
        // 检测是否有 none 参数——有则 CSS 原样透传
        let has_none = args
            .iter()
            .any(|a| matches!(a, Value::String(s, false) if s == "none"));
        // 检测 alpha 参数是否存在（4 个参数时最后一个为 alpha）
        let has_alpha = args.len() == 4;
        match &args[..] {
            [
                Value::Number(r, ru),
                Value::Number(g, gu),
                Value::Number(b, bu),
            ] => {
                // 百分比参数转换为 0-255
                let r_val = if ru.as_deref() == Some("%") {
                    r * 255.0 / 100.0
                } else {
                    *r
                };
                let g_val = if gu.as_deref() == Some("%") {
                    g * 255.0 / 100.0
                } else {
                    *g
                };
                let b_val = if bu.as_deref() == Some("%") {
                    b * 255.0 / 100.0
                } else {
                    *b
                };
                crate::__tracing::debug!(
                    target: "sasspile::color",
                    fn = "rgba",
                    r = *r, g = *g, b = *b,
                    "rgba 3-arg input"
                );
                Ok(Value::Color(Color::with_rgb(
                    r_val,
                    g_val,
                    b_val,
                    1.0,
                    ColorSpace::Rgb,
                    ColorOutput::RgbExplicit,
                )))
            }
            [
                Value::Number(r, ru),
                Value::Number(g, gu),
                Value::Number(b, bu),
                Value::Number(a, ua),
            ] => {
                let r_val = if ru.as_deref() == Some("%") {
                    r * 255.0 / 100.0
                } else {
                    *r
                };
                let g_val = if gu.as_deref() == Some("%") {
                    g * 255.0 / 100.0
                } else {
                    *g
                };
                let b_val = if bu.as_deref() == Some("%") {
                    b * 255.0 / 100.0
                } else {
                    *b
                };
                let alpha = if ua.as_deref() == Some("%") {
                    *a / 100.0
                } else {
                    *a
                };
                crate::__tracing::debug!(
                    target: "sasspile::color",
                    fn = "rgba",
                    r = *r, g = *g, b = *b, a = *a,
                    "rgba 4-arg input"
                );
                Ok(Value::Color(Color::with_rgb(
                    r_val,
                    g_val,
                    b_val,
                    alpha,
                    ColorSpace::Rgb,
                    ColorOutput::RgbExplicit,
                )))
            }
            // rgba($color, $alpha) — 修改颜色的 alpha 通道
            [Value::Color(c), Value::Number(a, _)] => Ok(Value::Color(Color::with_rgb(
                c.legacy_rgb[0],
                c.legacy_rgb[1],
                c.legacy_rgb[2],
                *a,
                c.space,
                c.output,
            ))),
            // CSS 透传：参数包含 none/var()/calc() 等非数值时，原样输出
            _ if has_none
                || args.iter().any(|a| {
                    matches!(a, Value::Calc(_) | Value::String(_, false))
                        && !matches!(a, Value::Color(_))
                }) =>
            {
                let (rgb_args, alpha) = if has_alpha {
                    (&args[..3], Some(&args[3]))
                } else {
                    (&args[..], None)
                };
                let sep = if is_space_sep { " " } else { ", " };
                let rgb_str = rgb_args
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(sep);
                let full_str = if let Some(a) = alpha {
                    match is_space_sep {
                        true => format!("{rgb_str} / {a}"),
                        false => format!("{rgb_str}, {a}"),
                    }
                } else {
                    rgb_str
                };
                Ok(Value::String(format!("{fn_name}({full_str})"), false))
            }
            _ => Err(SassError::Eval("rgba requires 3-4 number arguments".into())),
        }
    }

    pub(crate) fn builtin_darken(args: &[Value]) -> Result<Value> {
        match args {
            [Value::Color(c), Value::Number(amount, _)] => {
                crate::__tracing::debug!(
                    target: "sasspile::color",
                    fn = "darken",
                    input_r = c.legacy_rgb[0], input_g = c.legacy_rgb[1], input_b = c.legacy_rgb[2], input_a = c.a,
                    amount = *amount,
                    "darken input"
                );
                // Sass darken = HSL lightness 减少
                let (h, s, l) =
                    Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
                let new_l = (l - *amount / 100.0).max(0.0);
                let new_c = Evaluator::hsl_to_rgb(h, s, new_l);
                let result = Value::Color(Color::with_hsl(
                    h,
                    s,
                    new_l,
                    c.a,
                    ColorOutput::RgbPercent,
                    new_c.legacy_rgb,
                ));
                crate::__tracing::debug!(
                    target: "sasspile::color",
                    fn = "darken",
                    result = %result,
                    "darken result"
                );
                Ok(result)
            }
            _ => Err(SassError::Eval(
                "darken requires (color, amount) arguments".into(),
            )),
        }
    }

    pub(crate) fn builtin_lighten(args: &[Value]) -> Result<Value> {
        match args {
            [Value::Color(c), Value::Number(amount, _)] => {
                crate::__tracing::debug!(
                    target: "sasspile::color",
                    fn = "lighten",
                    input_r = c.legacy_rgb[0], input_g = c.legacy_rgb[1], input_b = c.legacy_rgb[2], input_a = c.a,
                    amount = *amount,
                    "lighten input"
                );
                // Sass lighten = HSL lightness 增加
                let (h, s, l) =
                    Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
                let new_l = (l + *amount / 100.0).min(1.0);
                let new_c = Evaluator::hsl_to_rgb(h, s, new_l);
                let result = Value::Color(Color::with_hsl(
                    h,
                    s,
                    new_l,
                    c.a,
                    ColorOutput::RgbPercent,
                    new_c.legacy_rgb,
                ));
                crate::__tracing::debug!(
                    target: "sasspile::color",
                    fn = "lighten",
                    result = %result,
                    "lighten result"
                );
                Ok(result)
            }
            _ => Err(SassError::Eval(
                "lighten requires (color, amount) arguments".into(),
            )),
        }
    }

    pub(crate) fn builtin_mix(args: &[Value]) -> Result<Value> {
        match args {
            [Value::Color(a), Value::Color(b)] => {
                crate::__tracing::debug!(
                    target: "sasspile::color",
                    fn = "mix",
                    color_a = ?a, color_b = ?b, weight = 50.0_f64,
                    "mix 2-arg input"
                );
                Ok(Value::Color(Color::rgba(
                    f64::midpoint(a.legacy_rgb[0], b.legacy_rgb[0]),
                    f64::midpoint(a.legacy_rgb[1], b.legacy_rgb[1]),
                    f64::midpoint(a.legacy_rgb[2], b.legacy_rgb[2]),
                    f64::midpoint(a.a, b.a),
                )))
            }
            [Value::Color(a), Value::Color(b), Value::Number(w, _)] => {
                crate::__tracing::debug!(
                    target: "sasspile::color",
                    fn = "mix",
                    color_a = ?a, color_b = ?b, weight = *w,
                    "mix 3-arg input"
                );
                let weight = *w / 100.0;
                Ok(Value::Color(Color::rgba(
                    a.legacy_rgb[0] * (1.0 - weight) + b.legacy_rgb[0] * weight,
                    a.legacy_rgb[1] * (1.0 - weight) + b.legacy_rgb[1] * weight,
                    a.legacy_rgb[2] * (1.0 - weight) + b.legacy_rgb[2] * weight,
                    a.a * (1.0 - weight) + b.a * weight,
                )))
            }
            _ => {
                crate::__tracing::debug!(
                    target: "sasspile::color",
                    fn = "mix",
                    n_args = args.len(),
                    arg_types = ?args.iter().map(std::mem::discriminant).collect::<Vec<_>>(),
                    args_debug = ?args.iter().map(|a| format!("{a}")).collect::<Vec<_>>(),
                    "mix argument mismatch"
                );
                Err(SassError::Eval("mix requires 2-3 arguments".into()))
            }
        }
    }
}
