//! sass:color built-in module.
//!
//! Implements: adjust, scale, change, mix, complement, invert, grayscale,
//! channel accessors, ie-hex-str, is-legacy, is-in-gamut,
//! to-space, to-gamut, and legacy global functions.

use crate::ast::Arg;
use crate::env::Env;
use crate::error::{SassError, SourcePos};
use crate::value::{Color, ColorSpace, Value};
use super::helpers::*;

/// Register all color builtins.
pub fn register(env: &mut Env) {
    let span = tracing::debug_span!("register_color", stage = "init", module = "color");
    let _enter = span.enter();

    // Channel accessors
    env.register_builtin("red".into(), color_red);
    env.register_builtin("green".into(), color_green);
    env.register_builtin("blue".into(), color_blue);
    env.register_builtin("alpha".into(), color_alpha);
    env.register_builtin("opacity".into(), color_alpha);
    env.register_builtin("hue".into(), color_hue);
    env.register_builtin("saturation".into(), color_saturation);
    env.register_builtin("lightness".into(), color_lightness);

    // Color manipulation
    env.register_builtin("color-adjust".into(), color_adjust);
    env.register_builtin("color-scale".into(), color_scale);
    env.register_builtin("color-change".into(), color_change);
    env.register_builtin("color-mix".into(), color_mix);
    env.register_builtin("color-complement".into(), color_complement);
    env.register_builtin("color-invert".into(), color_invert);
    env.register_builtin("color-grayscale".into(), color_grayscale);
    env.register_builtin("color-channel".into(), color_channel);
    env.register_builtin("color-ie-hex-str".into(), color_ie_hex_str);
    env.register_builtin("color-is-legacy".into(), color_is_legacy);
    env.register_builtin("color-is-in-gamut".into(), color_is_in_gamut);
    env.register_builtin("color-to-space".into(), color_to_space);
    env.register_builtin("color-to-gamut".into(), color_to_gamut);

    // Legacy global functions
    env.register_builtin("rgb".into(), legacy_rgb);
    env.register_builtin("rgba".into(), legacy_rgba);
    env.register_builtin("hsl".into(), legacy_hsl);
    env.register_builtin("hsla".into(), legacy_hsla);
    env.register_builtin("lighten".into(), legacy_lighten);
    env.register_builtin("darken".into(), legacy_darken);
    env.register_builtin("saturate".into(), legacy_saturate);
    env.register_builtin("desaturate".into(), legacy_desaturate);
    env.register_builtin("adjust-hue".into(), legacy_adjust_hue);
    env.register_builtin("fade-in".into(), legacy_fade_in);
    env.register_builtin("fade-out".into(), legacy_fade_out);
    env.register_builtin("grayscale".into(), color_grayscale);
    env.register_builtin("complement".into(), color_complement);
    env.register_builtin("invert".into(), color_invert);
    env.register_builtin("mix".into(), legacy_mix);
    env.register_builtin("ie-hex-str".into(), color_ie_hex_str);
}

fn get_args(args: &[Arg], env: &mut Env) -> Result<Vec<Value>, SassError> {
    eval_args(args, env, &[])
}

fn color_red(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let c = expect_color(&vals[0], "red")?;
    Ok(num(c.red().round()))
}

fn color_green(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let c = expect_color(&vals[0], "green")?;
    Ok(num(c.green().round()))
}

fn color_blue(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let c = expect_color(&vals[0], "blue")?;
    Ok(num(c.blue().round()))
}

fn color_alpha(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let c = expect_color(&vals[0], "alpha")?;
    Ok(num(c.alpha()))
}

fn color_hue(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let c = expect_color(&vals[0], "hue")?;
    Ok(num_unit(c.hue(), "deg"))
}

fn color_saturation(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let c = expect_color(&vals[0], "saturation")?;
    Ok(num_unit(c.saturation(), "%"))
}

fn color_lightness(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let c = expect_color(&vals[0], "lightness")?;
    Ok(num_unit(c.lightness(), "%"))
}

fn color_channel(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.len() < 2 {
        return Err(SassError::eval("channel: expected 2 arguments", SourcePos::default()));
    }
    let c = expect_color(&vals[0], "channel")?;
    let ch = expect_string(&vals[1], "channel")?;
    match ch.value.as_str() {
        "red" | "r" => Ok(num(c.red())),
        "green" | "g" => Ok(num(c.green())),
        "blue" | "b" => Ok(num(c.blue())),
        "alpha" | "a" => Ok(num(c.alpha())),
        "hue" | "h" => Ok(num_unit(c.hue(), "deg")),
        "saturation" | "s" => Ok(num_unit(c.saturation(), "%")),
        "lightness" | "l" => Ok(num_unit(c.lightness(), "%")),
        _ => Err(SassError::eval(
            format!("channel: unknown channel {}", ch.value), SourcePos::default())),
    }
}

fn color_adjust(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.is_empty() {
        return Err(SassError::eval("adjust: expected at least 1 argument", SourcePos::default()));
    }
    let c = expect_color(&vals[0], "adjust")?.clone();
    let mut h = c.hue(); let mut s = c.saturation(); let mut l = c.lightness(); let mut a = c.alpha();
    for (i, arg) in args.iter().enumerate().skip(1) {
        if let Some(name) = &arg.name {
            if let Some(val) = vals.get(i) {
                let n = expect_number(val, "adjust")?;
                match name.as_str() {
                    "red" | "green" | "blue" => {}
                    "alpha" => a = (a + n.value).clamp(0.0, 1.0),
                    "hue" => h += n.value,
                    "saturation" => s = (s + n.value).clamp(0.0, 100.0),
                    "lightness" => l = (l + n.value).clamp(0.0, 100.0),
                    _ => {}
                }
            }
        }
    }
    Ok(Value::Color(Color::hsl(h, s, l, a)))
}

fn color_scale(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.is_empty() {
        return Err(SassError::eval("scale: expected at least 1 argument", SourcePos::default()));
    }
    let c = expect_color(&vals[0], "scale")?.clone();
    let h = c.hue(); let mut s = c.saturation(); let mut l = c.lightness(); let mut a = c.alpha();
    for (i, arg) in args.iter().enumerate().skip(1) {
        if let Some(name) = &arg.name {
            if let Some(val) = vals.get(i) {
                let n = expect_number(val, "scale")?;
                let pct = n.value / 100.0;
                match name.as_str() {
                    "alpha" => a = scale_value(a, 1.0, pct),
                    "saturation" => s = scale_value(s, 100.0, pct),
                    "lightness" => l = scale_value(l, 100.0, pct),
                    _ => {}
                }
            }
        }
    }
    Ok(Value::Color(Color::hsl(h, s, l, a)))
}

fn color_change(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.is_empty() {
        return Err(SassError::eval("change: expected at least 1 argument", SourcePos::default()));
    }
    let c = expect_color(&vals[0], "change")?.clone();
    let mut h = c.hue(); let mut s = c.saturation(); let mut l = c.lightness(); let mut a = c.alpha();
    for (i, arg) in args.iter().enumerate().skip(1) {
        if let Some(name) = &arg.name {
            if let Some(val) = vals.get(i) {
                let n = expect_number(val, "change")?;
                match name.as_str() {
                    "alpha" => a = n.value.clamp(0.0, 1.0),
                    "hue" => h = n.value,
                    "saturation" => s = n.value.clamp(0.0, 100.0),
                    "lightness" => l = n.value.clamp(0.0, 100.0),
                    _ => {}
                }
            }
        }
    }
    Ok(Value::Color(Color::hsl(h, s, l, a)))
}

fn color_mix(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.len() < 2 {
        return Err(SassError::eval("mix: expected at least 2 arguments", SourcePos::default()));
    }
    let c1 = expect_color_or_name(&vals[0], "mix")?;
    let c2 = expect_color_or_name(&vals[1], "mix")?;
    let weight = if vals.len() >= 3 { expect_number(&vals[2], "mix")?.value / 100.0 } else { 0.5 };
    let w = weight;
    let r1 = c1.red(); let g1 = c1.green(); let b1 = c1.blue(); let a1 = c1.alpha();
    let r2 = c2.red(); let g2 = c2.green(); let b2 = c2.blue(); let a2 = c2.alpha();
    let a = a1 * (1.0 - w) + a2 * w;
    let r = (r1 * a1 * (1.0 - w) + r2 * a2 * w) / a;
    let g = (g1 * a1 * (1.0 - w) + g2 * a2 * w) / a;
    let b = (b1 * a1 * (1.0 - w) + b2 * a2 * w) / a;
    Ok(Value::Color(Color::rgb(r, g, b, a)))
}

fn color_complement(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let c = expect_color_or_name(&vals[0], "complement")?.clone();
    Ok(Value::Color(Color::hsl(c.hue() + 180.0, c.saturation(), c.lightness(), c.alpha())))
}

fn color_invert(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let c = expect_color_or_name(&vals[0], "invert")?.clone();
    let rgb = c.to_rgb();
    Ok(Value::Color(Color::rgb(255.0 - rgb.red(), 255.0 - rgb.green(), 255.0 - rgb.blue(), rgb.alpha())))
}

fn color_grayscale(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let c = expect_color_or_name(&vals[0], "grayscale")?.clone();
    Ok(Value::Color(Color::hsl(c.hue(), 0.0, c.lightness(), c.alpha())))
}

fn color_ie_hex_str(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let c = expect_color(&vals[0], "ie-hex-str")?;
    let rgb = c.to_rgb();
    let a = (c.alpha() * 255.0).round() as u8;
    let r = (rgb.red().round() as u8).min(255);
    let g = (rgb.green().round() as u8).min(255);
    let b = (rgb.blue().round() as u8).min(255);
    Ok(quoted_str(&format!("#{:02X}{:02X}{:02X}{:02X}", a, r, g, b)))
}

fn color_is_legacy(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let c = expect_color(&vals[0], "is-legacy")?;
    Ok(Value::Bool(c.legacy))
}

fn color_is_in_gamut(_args: &[Arg], _env: &mut Env) -> Result<Value, SassError> {
    Ok(Value::Bool(true))
}

fn color_to_space(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.len() < 2 {
        return Err(SassError::eval("to-space: expected 2 arguments", SourcePos::default()));
    }
    let c = expect_color(&vals[0], "to-space")?.clone();
    let space_name = expect_string(&vals[1], "to-space")?;
    let space = match space_name.value.as_str() {
        "rgb" => ColorSpace::Rgb,
        "hsl" => ColorSpace::Hsl,
        "hwb" => ColorSpace::Hwb,
        "lab" => ColorSpace::Lab,
        "lch" => ColorSpace::Lch,
        "oklab" => ColorSpace::Oklab,
        "oklch" => ColorSpace::Oklch,
        _ => return Err(SassError::eval(
            format!("to-space: unknown space {}", space_name.value), SourcePos::default())),
    };
    let mut new_color = c;
    new_color.space = space;
    Ok(Value::Color(new_color))
}

fn color_to_gamut(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let c = expect_color(&vals[0], "to-gamut")?.clone();
    Ok(Value::Color(c))
}

// Legacy global functions

fn legacy_rgb(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.is_empty() {
        return Err(SassError::eval("rgb: expected arguments", SourcePos::default()));
    }
    if vals.len() == 1 {
        if let Value::String(s) = &vals[0] {
            let hex = s.value.trim_start_matches('#');
            if hex.len() == 6 {
                let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f64;
                let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f64;
                let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f64;
                return Ok(Value::Color(Color::rgb(r, g, b, 1.0)));
            }
        }
    }
    let r = expect_number(&vals[0], "rgb")?.value;
    let g = if vals.len() >= 2 { expect_number(&vals[1], "rgb")?.value } else { 0.0 };
    let b = if vals.len() >= 3 { expect_number(&vals[2], "rgb")?.value } else { 0.0 };
    let a = if vals.len() >= 4 { expect_number(&vals[3], "rgb")?.value } else { 1.0 };
    Ok(Value::Color(Color::rgb(r, g, b, a)))
}

fn legacy_rgba(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    legacy_rgb(args, env)
}

fn legacy_hsl(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.is_empty() {
        return Err(SassError::eval("hsl: expected arguments", SourcePos::default()));
    }
    let h = expect_number(&vals[0], "hsl")?.value;
    let s = if vals.len() >= 2 { expect_number(&vals[1], "hsl")?.value } else { 0.0 };
    let l = if vals.len() >= 3 { expect_number(&vals[2], "hsl")?.value } else { 0.0 };
    let a = if vals.len() >= 4 { expect_number(&vals[3], "hsl")?.value } else { 1.0 };
    Ok(Value::Color(Color::hsl(h, s, l, a)))
}

fn legacy_hsla(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    legacy_hsl(args, env)
}

fn legacy_lighten(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let c = expect_color_or_name(&vals[0], "lighten")?.clone();
    let amount = expect_number(&vals[1], "lighten")?.value;
    Ok(Value::Color(Color::hsl(c.hue(), c.saturation(), (c.lightness() + amount).clamp(0.0, 100.0), c.alpha())))
}

fn legacy_darken(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let c = expect_color_or_name(&vals[0], "darken")?.clone();
    let amount = expect_number(&vals[1], "darken")?.value;
    Ok(Value::Color(Color::hsl(c.hue(), c.saturation(), (c.lightness() - amount).clamp(0.0, 100.0), c.alpha())))
}

fn legacy_saturate(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let c = expect_color_or_name(&vals[0], "saturate")?.clone();
    let amount = expect_number(&vals[1], "saturate")?.value;
    Ok(Value::Color(Color::hsl(c.hue(), (c.saturation() + amount).clamp(0.0, 100.0), c.lightness(), c.alpha())))
}

fn legacy_desaturate(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let c = expect_color_or_name(&vals[0], "desaturate")?.clone();
    let amount = expect_number(&vals[1], "desaturate")?.value;
    Ok(Value::Color(Color::hsl(c.hue(), (c.saturation() - amount).clamp(0.0, 100.0), c.lightness(), c.alpha())))
}

fn legacy_adjust_hue(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let c = expect_color_or_name(&vals[0], "adjust-hue")?.clone();
    let amount = expect_number(&vals[1], "adjust-hue")?.value;
    Ok(Value::Color(Color::hsl(c.hue() + amount, c.saturation(), c.lightness(), c.alpha())))
}

fn legacy_fade_in(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let c = expect_color_or_name(&vals[0], "fade-in")?.clone();
    let amount = expect_number(&vals[1], "fade-in")?.value;
    Ok(Value::Color(Color::hsl(c.hue(), c.saturation(), c.lightness(), (c.alpha() + amount).clamp(0.0, 1.0))))
}

fn legacy_fade_out(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let c = expect_color_or_name(&vals[0], "fade-out")?.clone();
    let amount = expect_number(&vals[1], "fade-out")?.value;
    Ok(Value::Color(Color::hsl(c.hue(), c.saturation(), c.lightness(), (c.alpha() - amount).clamp(0.0, 1.0))))
}

fn legacy_mix(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    color_mix(args, env)
}

/// Scale a value towards the max (positive) or min (negative).
fn scale_value(current: f64, max: f64, pct: f64) -> f64 {
    if pct > 0.0 {
        current + (max - current) * pct
    } else {
        current + current * pct
    }
}
