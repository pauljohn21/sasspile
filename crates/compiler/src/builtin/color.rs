//! sass:color — color manipulation builtins.
//!
//! All colors are represented as hex strings (e.g. "#ff0000") in the `Value`
//! system. This module provides helpers to parse, convert, and manipulate
//! those colors.

use crate::reactive::Value;
use crate::Error;

/// Parse a hex color string into (r, g, b) components (0-255 each).
fn parse_hex(hex: &str) -> std::result::Result<(u8, u8, u8), Error> {
    let hex = hex.trim();
    let body = hex.strip_prefix('#').ok_or_else(|| {
        Error::msg(format!("color.parse_hex: expected # prefix, got: {hex}"))
    })?;

    match body.len() {
        3 => {
            // Expand shorthand: "f00" → "ff0000"
            let expanded: String = body.chars().flat_map(|c| [c, c]).collect();
            parse_six_digit(&expanded)
        }
        6 => parse_six_digit(body),
        _ => Err(Error::msg(format!(
            "color.parse_hex: invalid hex length: {hex}"
        ))),
    }
}

fn parse_six_digit(s: &str) -> std::result::Result<(u8, u8, u8), Error> {
    let r = u8::from_str_radix(&s[0..2], 16)
        .map_err(|_| Error::msg(format!("color.parse_hex: invalid red component: {s}")))?;
    let g = u8::from_str_radix(&s[2..4], 16)
        .map_err(|_| Error::msg(format!("color.parse_hex: invalid green component: {s}")))?;
    let b = u8::from_str_radix(&s[4..6], 16)
        .map_err(|_| Error::msg(format!("color.parse_hex: invalid blue component: {s}")))?;
    Ok((r, g, b))
}

/// Convert RGB components back to a hex string.
fn to_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// Clamp a float to [0, 255] and round.
fn clamp_byte(v: f64) -> u8 {
    v.clamp(0.0, 255.0).round() as u8
}

/// Extract the color value from a Value, parsing if needed.
fn get_color(v: &Value) -> std::result::Result<(u8, u8, u8), Error> {
    match v {
        Value::String(s) => parse_hex(s),
        _ => Err(Error::msg(format!(
            "color: expected hex string, got: {v:?}"
        ))),
    }
}

/// Extract a percentage (0-100) from a Value.
fn get_percentage(v: &Value) -> std::result::Result<f64, Error> {
    match v {
        Value::Number(n) if (0.0..=100.0).contains(n) => Ok(*n),
        Value::Number(n) => Err(Error::msg(format!(
            "color: percentage must be 0-100, got: {n}"
        ))),
        _ => Err(Error::msg(format!(
            "color: expected number for percentage, got: {v:?}"
        ))),
    }
}

// ────────────────── Public Builtins ──────────────────

/// `darken($color, $amount)` — reduce lightness by percentage.
pub fn darken(args: &[Value]) -> std::result::Result<Value, Error> {
    let color = args.first().ok_or_else(|| Error::msg("darken: missing color"))?;
    let amount = args.get(1).ok_or_else(|| Error::msg("darken: missing amount"))?;

    let (r, g, b) = get_color(color)?;
    let pct = get_percentage(amount)?;
    let factor = 1.0 - pct / 100.0;

    let (r, g, b) = (
        clamp_byte(r as f64 * factor),
        clamp_byte(g as f64 * factor),
        clamp_byte(b as f64 * factor),
    );
    Ok(Value::String(to_hex(r, g, b)))
}

/// `lighten($color, $amount)` — increase lightness by percentage.
pub fn lighten(args: &[Value]) -> std::result::Result<Value, Error> {
    let color = args.first().ok_or_else(|| Error::msg("lighten: missing color"))?;
    let amount = args.get(1).ok_or_else(|| Error::msg("lighten: missing amount"))?;

    let (r, g, b) = get_color(color)?;
    let pct = get_percentage(amount)?;
    let factor = pct / 100.0;

    let (r, g, b) = (
        clamp_byte(r as f64 + (255.0 - r as f64) * factor),
        clamp_byte(g as f64 + (255.0 - g as f64) * factor),
        clamp_byte(b as f64 + (255.0 - b as f64) * factor),
    );
    Ok(Value::String(to_hex(r, g, b)))
}

/// `mix($color1, $color2, $weight)` — mix two colors.
/// `weight` defaults to 50 (percentage of color1).
pub fn mix(args: &[Value]) -> std::result::Result<Value, Error> {
    let c1 = args.first().ok_or_else(|| Error::msg("mix: missing color1"))?;
    let c2 = args.get(1).ok_or_else(|| Error::msg("mix: missing color2"))?;

    let weight = match args.get(2) {
        Some(Value::Number(n)) if (0.0..=100.0).contains(n) => *n,
        Some(other) => return Err(Error::msg(format!("mix: invalid weight: {other:?}"))),
        None => 50.0,
    };

    let (r1, g1, b1) = get_color(c1)?;
    let (r2, g2, b2) = get_color(c2)?;

    let w = weight / 100.0;
    let (r, g, b) = (
        clamp_byte(r1 as f64 * w + r2 as f64 * (1.0 - w)),
        clamp_byte(g1 as f64 * w + g2 as f64 * (1.0 - w)),
        clamp_byte(b1 as f64 * w + b2 as f64 * (1.0 - w)),
    );
    Ok(Value::String(to_hex(r, g, b)))
}

/// `opacify($color, $amount)` / `fade_in($color, $amount)` — increase opacity.
/// (stub: returns the hex color unchanged — full RGBA support pending Value type extension)
pub fn opacify(args: &[Value]) -> std::result::Result<Value, Error> {
    let color = args.first().ok_or_else(|| Error::msg("opacify: missing color"))?;
    // TODO: implement when Value supports RGBA
    Ok(color.clone())
}

/// `transparentize($color, $amount)` / `fade_out($color, $amount)` — decrease opacity.
pub fn transparentize(args: &[Value]) -> std::result::Result<Value, Error> {
    let color = args.first().ok_or_else(|| Error::msg("transparentize: missing color"))?;
    // TODO: implement when Value supports RGBA
    Ok(color.clone())
}
