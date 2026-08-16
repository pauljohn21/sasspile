//! CSS4 color functions — color-mix, hwb, oklch, oklab, color().
//!
//! Implements CSS Color Level 4 functions in the sass:color module.

use crate::color::oklab::{oklab_to_srgb, srgb_to_oklab, OklchColor};
use crate::eval::error::EvalError;
use crate::eval::evaluator::EvalContext;
use crate::parser::Expr;
use crate::value::{SassColor, Value};

/// Evaluate a CSS4 color function.
pub fn call(
    func: &str,
    args: &[Expr],
    ctx: &mut EvalContext<'_>,
) -> Result<Option<Value>, EvalError> {
    match func {
        "color-mix" => color_mix(args, ctx).map(Some),
        "hwb" => hwb(args, ctx).map(Some),
        "oklch" => oklch_fn(args, ctx).map(Some),
        "oklab" => oklab_fn(args, ctx).map(Some),
        "color" => color_fn(args, ctx).map(Some),
        _ => Ok(None),
    }
}

/// color-mix(in <space>, <color1> <pct1>, <color2> [<pct2>])
fn color_mix(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    if args.len() < 3 {
        return Err(EvalError::ArityMismatch(
            "color-mix".into(),
            "3+".into(),
            args.len(),
        ));
    }
    let space = ctx.eval_expr(&args[0])?.to_string_value();
    let space = space.trim();
    let space = if space.starts_with("in ") {
        space[3..].trim()
    } else {
        space
    };

    let c1 = eval_color(&args[1], ctx)?;
    let c2 = eval_color(&args[2], ctx)?;

    let pct1 = if args.len() > 3 {
        eval_number(&args[3], ctx)?
    } else {
        50.0
    };
    let t = (pct1 / 100.0).clamp(0.0, 1.0);

    match space {
        "srgb" => {
            let mixed = c1.mix(&c2, t);
            Ok(Value::Color(mixed))
        }
        "oklch" => {
            let ok1 = srgb_to_oklab(c1.r, c1.g, c1.b);
            let ok2 = srgb_to_oklab(c2.r, c2.g, c2.b);
            let lch1 = ok1.to_oklch();
            let lch2 = ok2.to_oklch();
            // Handle missing hue (achromatic).
            let h1 = if lch1.c < 0.001 { lch2.h } else { lch1.h };
            let h2 = if lch2.c < 0.001 { lch1.h } else { lch2.h };
            let mixed = OklchColor::new(
                lch1.l * (1.0 - t) + lch2.l * t,
                lch1.c * (1.0 - t) + lch2.c * t,
                h1 * (1.0 - t) + h2 * t,
            );
            let rgb = oklab_to_srgb(
                mixed.l,
                mixed.c * mixed.h.to_radians().cos(),
                mixed.c * mixed.h.to_radians().sin(),
            );
            Ok(Value::Color(SassColor::new(
                (rgb[0] * 255.0).round() as u8,
                (rgb[1] * 255.0).round() as u8,
                (rgb[2] * 255.0).round() as u8,
                c1.a * (1.0 - t) + c2.a * t,
            )))
        }
        _ => {
            // Default fallback to srgb interpolation.
            let mixed = c1.mix(&c2, t);
            Ok(Value::Color(mixed))
        }
    }
}

/// hwb(H, W, B [, A]) — hue-whiteness-blackness color.
fn hwb(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let h = eval_number(&args[0], ctx)?;
    let w = eval_number(&args[1], ctx)?;
    let b = eval_number(&args[2], ctx)?;
    let a = if args.len() > 3 {
        eval_number(&args[3], ctx)?
    } else {
        1.0
    };

    // HWB: W + B must be <= 1 (if > 1, scale down).
    let sum = w + b;
    let (w, b) = if sum > 1.0 {
        (w / sum, b / sum)
    } else {
        (w, b)
    };
    let hsl_color = SassColor::from_hsl(h, 1.0, 0.5, a);
    let r = hsl_color.r as f64 / 255.0;
    let g = hsl_color.g as f64 / 255.0;
    let bl = hsl_color.b as f64 / 255.0;
    // Apply whiteness/blackness.
    let r = r * (1.0 - w - b) + w;
    let g = g * (1.0 - w - b) + w;
    let bl = bl * (1.0 - w - b) + w;

    Ok(Value::Color(SassColor::new(
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (bl * 255.0).round() as u8,
        a,
    )))
}

/// oklch(L C H [, A]) — create color from Oklch values.
fn oklch_fn(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let l = eval_number(&args[0], ctx)?;
    let c = eval_number(&args[1], ctx)?;
    let h = eval_number(&args[2], ctx)?;
    let a = if args.len() > 3 {
        eval_number(&args[3], ctx)?
    } else {
        1.0
    };

    let oklch = OklchColor::new(l, c, h);
    let oklab = oklch.to_oklab();
    let rgb = oklab_to_srgb(oklab.l, oklab.a, oklab.b);

    Ok(Value::Color(SassColor::new(
        (rgb[0] * 255.0).round() as u8,
        (rgb[1] * 255.0).round() as u8,
        (rgb[2] * 255.0).round() as u8,
        a,
    )))
}

/// oklab(L a b [, A]) — create color from Oklab values.
fn oklab_fn(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let l = eval_number(&args[0], ctx)?;
    let a = eval_number(&args[1], ctx)?;
    let b = eval_number(&args[2], ctx)?;
    let alpha = if args.len() > 3 {
        eval_number(&args[3], ctx)?
    } else {
        1.0
    };

    let rgb = oklab_to_srgb(l, a, b);
    Ok(Value::Color(SassColor::new(
        (rgb[0] * 255.0).round() as u8,
        (rgb[1] * 255.0).round() as u8,
        (rgb[2] * 255.0).round() as u8,
        alpha,
    )))
}

/// color(<space> <channels>) — create color in a given color space.
fn color_fn(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let space = ctx.eval_expr(&args[0])?;
    let space_name = space.to_string_value();
    match space_name.trim() {
        "srgb" => {
            let r = eval_number(&args[1], ctx)?;
            let g = eval_number(&args[2], ctx)?;
            let b = eval_number(&args[3], ctx)?;
            let a = if args.len() > 4 {
                eval_number(&args[4], ctx)?
            } else {
                1.0
            };
            Ok(Value::Color(SassColor::new(
                (r * 255.0).round() as u8,
                (g * 255.0).round() as u8,
                (b * 255.0).round() as u8,
                a,
            )))
        }
        "oklch" => oklch_fn(&args[1..], ctx),
        "oklab" => oklab_fn(&args[1..], ctx),
        _ => Err(EvalError::TypeError(format!(
            "unsupported color space: {space_name}"
        ))),
    }
}

/// Evaluate a color argument.
fn eval_color(arg: &Expr, ctx: &mut EvalContext<'_>) -> Result<SassColor, EvalError> {
    let val = ctx.eval_expr(arg)?;
    match &val {
        Value::Color(c) => Ok(c.clone()),
        _ => Err(EvalError::type_error("color", val.type_name())),
    }
}

/// Evaluate a number argument.
fn eval_number(arg: &Expr, ctx: &mut EvalContext<'_>) -> Result<f64, EvalError> {
    let val = ctx.eval_expr(arg)?;
    match &val {
        Value::Number(n) => Ok(n.value),
        _ => Err(EvalError::type_error("number", val.type_name())),
    }
}
