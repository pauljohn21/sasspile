use super::*;
use crate::error::Result;

impl Evaluator {
    pub(crate) fn hsl_to_rgb(h: f64, s: f64, l: f64) -> Color {
        let h = h.rem_euclid(360.0);
        tracing::trace!(
            target: "sasspile::color",
            fn = "hsl_to_rgb",
            h = h, s = s, l = l,
            "converting HSL to RGB"
        );
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;
        let (r1, g1, b1) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
        let result = Color::rgb(
            ((r1 + m) * 255.0).round() as u8,
            ((g1 + m) * 255.0).round() as u8,
            ((b1 + m) * 255.0).round() as u8,
        );
        tracing::trace!(
            target: "sasspile::color",
            fn = "hsl_to_rgb",
            r = result.r, g = result.g, b = result.b,
            "HSL to RGB result"
        );
        result
    }

    /// HWB → RGB 转换 (W3C CSS Color 4 算法)。
    pub(crate) fn hwb_to_rgb(h: f64, w: f64, b: f64, alpha: f32) -> Color {
        tracing::trace!(
            target: "sasspile::color",
            fn = "hwb_to_rgb",
            h = h, w = w, b = b, alpha = alpha,
            "converting HWB to RGB"
        );
        let h = (h % 360.0) / 360.0;
        let mut w = w;
        let mut b = b;
        let sum = w + b;
        if sum > 1.0 {
            w /= sum;
            b /= sum;
        }
        let factor = 1.0 - w - b;
        let hue_to_rgb = |m1: f64, m2: f64, mut hue: f64| -> f64 {
            if hue < 0.0 {
                hue += 1.0;
            }
            if hue > 1.0 {
                hue -= 1.0;
            }
            if hue < 1.0 / 6.0 {
                m1 + (m2 - m1) * hue * 6.0
            } else if hue < 0.5 {
                m2
            } else if hue < 2.0 / 3.0 {
                m1 + (m2 - m1) * (2.0 / 3.0 - hue) * 6.0
            } else {
                m1
            }
        };
        let to_rgb = |hue: f64| -> f64 { hue_to_rgb(0.0, 1.0, hue) * factor + w };
        let r = to_rgb(h + 1.0 / 3.0);
        let g = to_rgb(h);
        let bl = to_rgb(h - 1.0 / 3.0);
        Color::rgba(
            (r * 255.0).round() as u8,
            (g * 255.0).round() as u8,
            (bl * 255.0).round() as u8,
            alpha,
        )
    }

    /// RGB → HSL 转换。
    pub(crate) fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
        tracing::trace!(
            target: "sasspile::color",
            fn = "rgb_to_hsl",
            r = r, g = g, b = b,
            "converting RGB to HSL"
        );
        let r = r as f64 / 255.0;
        let g = g as f64 / 255.0;
        let b = b as f64 / 255.0;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;
        if (max - min).abs() < f64::EPSILON {
            return (0.0, 0.0, l);
        }
        let d = max - min;
        let s = if l > 0.5 {
            d / (2.0 - max - min)
        } else {
            d / (max + min)
        };
        let h = if max == r {
            ((g - b) / d + if g < b { 6.0 } else { 0.0 }) * 60.0
        } else if max == g {
            ((b - r) / d + 2.0) * 60.0
        } else {
            ((r - g) / d + 4.0) * 60.0
        };
        let result = (h, s, l);
        tracing::trace!(
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
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let val = (nanos % 1_000_000) as f64;
        val / 1_000_000.0
    }

    pub(crate) fn builtin_rgba(args: &[Value]) -> Result<Value> {
        match args {
            [
                Value::Number(r, _),
                Value::Number(g, _),
                Value::Number(b, _),
            ] => {
                tracing::debug!(
                    target: "sasspile::color",
                    fn = "rgba",
                    r = *r, g = *g, b = *b,
                    "rgba 3-arg input"
                );
                Ok(Value::Color(Color::rgb(*r as u8, *g as u8, *b as u8)))
            }
            [
                Value::Number(r, _),
                Value::Number(g, _),
                Value::Number(b, _),
                Value::Number(a, _),
            ] => {
                tracing::debug!(
                    target: "sasspile::color",
                    fn = "rgba",
                    r = *r, g = *g, b = *b, a = *a,
                    "rgba 4-arg input"
                );
                Ok(Value::Color(Color::rgba(
                    *r as u8, *g as u8, *b as u8, *a as f32,
                )))
            }
            _ => Err(SassError::Eval("rgba 需要 3-4 个数字参数".into())),
        }
    }

    pub(crate) fn builtin_darken(args: &[Value]) -> Result<Value> {
        match args {
            [Value::Color(c), Value::Number(amount, _)] => {
                tracing::debug!(
                    target: "sasspile::color",
                    fn = "darken",
                    input_r = c.r, input_g = c.g, input_b = c.b, input_a = c.a,
                    amount = *amount,
                    "darken input"
                );
                let factor = 1.0 - (*amount as f32 / 100.0);
                let result = Value::Color(Color::rgba(
                    (c.r as f32 * factor) as u8,
                    (c.g as f32 * factor) as u8,
                    (c.b as f32 * factor) as u8,
                    c.a,
                ));
                tracing::debug!(
                    target: "sasspile::color",
                    fn = "darken",
                    result = %result,
                    "darken result"
                );
                Ok(result)
            }
            _ => Err(SassError::Eval("darken 需要 (color, amount) 参数".into())),
        }
    }

    pub(crate) fn builtin_lighten(args: &[Value]) -> Result<Value> {
        match args {
            [Value::Color(c), Value::Number(amount, _)] => {
                tracing::debug!(
                    target: "sasspile::color",
                    fn = "lighten",
                    input_r = c.r, input_g = c.g, input_b = c.b, input_a = c.a,
                    amount = *amount,
                    "lighten input"
                );
                let factor = *amount as f32 / 100.0;
                let result = Value::Color(Color::rgba(
                    (c.r as f32 + (255.0 - c.r as f32) * factor) as u8,
                    (c.g as f32 + (255.0 - c.g as f32) * factor) as u8,
                    (c.b as f32 + (255.0 - c.b as f32) * factor) as u8,
                    c.a,
                ));
                tracing::debug!(
                    target: "sasspile::color",
                    fn = "lighten",
                    result = %result,
                    "lighten result"
                );
                Ok(result)
            }
            _ => Err(SassError::Eval("lighten 需要 (color, amount) 参数".into())),
        }
    }

    pub(crate) fn builtin_mix(args: &[Value]) -> Result<Value> {
        match args {
            [Value::Color(a), Value::Color(b)] => {
                tracing::debug!(
                    target: "sasspile::color",
                    fn = "mix",
                    color_a = ?a, color_b = ?b, weight = 50.0_f64,
                    "mix 2-arg input"
                );
                Ok(Value::Color(Color::rgba(
                    ((a.r as u16 + b.r as u16) / 2) as u8,
                    ((a.g as u16 + b.g as u16) / 2) as u8,
                    ((a.b as u16 + b.b as u16) / 2) as u8,
                    (a.a + b.a) / 2.0,
                )))
            }
            [Value::Color(a), Value::Color(b), Value::Number(w, _)] => {
                tracing::debug!(
                    target: "sasspile::color",
                    fn = "mix",
                    color_a = ?a, color_b = ?b, weight = *w,
                    "mix 3-arg input"
                );
                let weight = *w as f32 / 100.0;
                Ok(Value::Color(Color::rgba(
                    (a.r as f32 * (1.0 - weight) + b.r as f32 * weight) as u8,
                    (a.g as f32 * (1.0 - weight) + b.g as f32 * weight) as u8,
                    (a.b as f32 * (1.0 - weight) + b.b as f32 * weight) as u8,
                    a.a * (1.0 - weight) + b.a * weight,
                )))
            }
            _ => Err(SassError::Eval("mix 需要 2-3 个参数".into())),
        }
    }
}
