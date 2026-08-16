//! sass:color module — color manipulation functions.

use crate::eval::error::EvalError;
use crate::eval::evaluator::EvalContext;
use crate::parser::Expr;
use crate::value::{Number, SassColor, Value};

/// Dispatch a sass:color function call.
pub fn call(
    func: &str,
    args: &[Expr],
    ctx: &mut EvalContext<'_>,
) -> Result<Option<Value>, EvalError> {
    match func {
        "adjust-hue" => adjust_hue(args, ctx).map(Some),
        "lighten" => lighten(args, ctx).map(Some),
        "darken" => darken(args, ctx).map(Some),
        "saturate" => saturate(args, ctx).map(Some),
        "desaturate" => desaturate(args, ctx).map(Some),
        "grayscale" => grayscale(args, ctx).map(Some),
        "invert" => invert(args, ctx).map(Some),
        "alpha" | "opacity" => alpha(args, ctx).map(Some),
        "rgba" => rgba(args, ctx).map(Some),
        "mix" => mix(args, ctx).map(Some),
        "complement" => complement(args, ctx).map(Some),
        "hue" => hue(args, ctx).map(Some),
        "saturation" => saturation(args, ctx).map(Some),
        "lightness" => lightness(args, ctx).map(Some),
        "red" => channel(args, ctx, |c| c.red()).map(Some),
        "green" => channel(args, ctx, |c| c.green()).map(Some),
        "blue" => channel(args, ctx, |c| c.blue()).map(Some),
        "fade-in" | "opacify" => fade_in(args, ctx).map(Some),
        "fade-out" | "transparentize" => fade_out(args, ctx).map(Some),
        "scale" => scale(args, ctx).map(Some),
        "adjust" => adjust(args, ctx).map(Some),
        "change" => change(args, ctx).map(Some),
        _ => Ok(None),
    }
}

/// Extract the first argument as a color.
fn eval_color(name: &str, args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<SassColor, EvalError> {
    if args.is_empty() {
        return Err(EvalError::ArityMismatch(name.into(), "1+".into(), 0));
    }
    let val = ctx.eval_expr(&args[0])?;
    match &val {
        Value::Color(c) => Ok(c.clone()),
        _ => Err(EvalError::type_error("color", val.type_name())),
    }
}

/// Adjust the hue of a color.
fn adjust_hue(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let color = eval_color("adjust-hue", args, ctx)?;
    let degrees = eval_number("adjust-hue", &args[1..], ctx)?;
    Ok(Value::Color(color.with_hue(color.hue() + degrees)))
}

/// Lighten a color.
fn lighten(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let color = eval_color("lighten", args, ctx)?;
    let amount = eval_number("lighten", &args[1..], ctx)?;
    Ok(Value::Color(color.lighten(amount)))
}

/// Darken a color.
fn darken(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let color = eval_color("darken", args, ctx)?;
    let amount = eval_number("darken", &args[1..], ctx)?;
    Ok(Value::Color(color.darken(amount)))
}

/// Saturate a color.
fn saturate(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let color = eval_color("saturate", args, ctx)?;
    let amount = eval_number("saturate", &args[1..], ctx)?;
    Ok(Value::Color(color.saturate(amount)))
}

/// Desaturate a color.
fn desaturate(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let color = eval_color("desaturate", args, ctx)?;
    let amount = eval_number("desaturate", &args[1..], ctx)?;
    Ok(Value::Color(color.desaturate(amount)))
}

/// Convert to grayscale.
fn grayscale(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let color = eval_color("grayscale", args, ctx)?;
    Ok(Value::Color(color.grayscale()))
}

/// Invert a color.
fn invert(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let color = eval_color("invert", args, ctx)?;
    Ok(Value::Color(color.invert()))
}

/// Get or set alpha/opacity.
fn alpha(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let color = eval_color("alpha", args, ctx)?;
    if args.len() >= 2 {
        let new_alpha = eval_number("alpha", &args[1..], ctx)?;
        Ok(Value::Color(color.with_alpha(new_alpha)))
    } else {
        Ok(Value::Number(Number::new(color.alpha(), crate::value::Unit::None)))
    }
}

/// Mix two colors.
fn mix(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let color1 = eval_color("mix", args, ctx)?;
    let color2 = if args.len() >= 2 {
        let val = ctx.eval_expr(&args[1])?;
        match &val {
            Value::Color(c) => c.clone(),
            _ => return Err(EvalError::type_error("color", val.type_name())),
        }
    } else {
        return Err(EvalError::ArityMismatch("mix".into(), "2+".into(), args.len()));
    };
    let weight = if args.len() >= 3 {
        eval_number("mix", &args[2..], ctx)?
    } else {
        50.0
    };
    Ok(Value::Color(color1.mix(&color2, weight / 100.0)))
}

/// Complement of a color.
fn complement(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let color = eval_color("complement", args, ctx)?;
    Ok(Value::Color(color.complement()))
}

/// Get hue of a color.
fn hue(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let color = eval_color("hue", args, ctx)?;
    Ok(Value::Number(Number::new(color.hue(), crate::value::Unit::Deg)))
}

/// Get saturation of a color (%).
fn saturation(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let color = eval_color("saturation", args, ctx)?;
    Ok(Value::Number(Number::new(color.saturation() * 100.0, crate::value::Unit::Percent)))
}

/// Get lightness of a color (%).
fn lightness(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let color = eval_color("lightness", args, ctx)?;
    Ok(Value::Number(Number::new(color.lightness() * 100.0, crate::value::Unit::Percent)))
}

/// Get a channel value (red, green, blue).
fn channel(args: &[Expr], ctx: &mut EvalContext<'_>, f: impl Fn(&SassColor) -> u8) -> Result<Value, EvalError> {
    let color = eval_color("channel", args, ctx)?;
    Ok(Value::Number(Number::unitless(f(&color) as f64)))
}

/// Fade in (increase alpha).
fn fade_in(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let color = eval_color("fade-in", args, ctx)?;
    let amount = eval_number("fade-in", &args[1..], ctx)?;
    Ok(Value::Color(color.fade_in(amount / 100.0)))
}

/// Fade out (decrease alpha).
fn fade_out(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let color = eval_color("fade-out", args, ctx)?;
    let amount = eval_number("fade-out", &args[1..], ctx)?;
    Ok(Value::Color(color.fade_out(amount / 100.0)))
}

/// Create rgba color from channels.
fn rgba(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let r = eval_number("rgba", args, ctx)? as u8;
    let g = eval_number("rgba", &args[1..], ctx)? as u8;
    let b = eval_number("rgba", &args[2..], ctx)? as u8;
    let a = if args.len() >= 4 {
        eval_number("rgba", &args[3..], ctx)?
    } else {
        1.0
    };
    Ok(Value::Color(SassColor::new(r, g, b, a)))
}

/// Scale channels.
fn scale(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let color = eval_color("scale", args, ctx)?;
    let r = eval_number("scale", &args[1..], ctx)?;
    let g = eval_number("scale", &args[2..], ctx)?;
    let b = eval_number("scale", &args[3..], ctx)?;
    let a = if args.len() >= 5 {
        eval_number("scale", &args[4..], ctx)?
    } else {
        0.0
    };
    Ok(Value::Color(color.scale(r, g, b, a)))
}

/// Adjust channels.
fn adjust(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let color = eval_color("adjust", args, ctx)?;
    let r = eval_number("adjust", &args[1..], ctx)?;
    let g = eval_number("adjust", &args[2..], ctx)?;
    let b = eval_number("adjust", &args[3..], ctx)?;
    let a = if args.len() >= 5 {
        eval_number("adjust", &args[4..], ctx)?
    } else {
        0.0
    };
    Ok(Value::Color(color.adjust(r, g, b, a)))
}

/// Change color properties.
fn change(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let color = eval_color("change", args, ctx)?;
    let mut result = color;
    if args.len() >= 2 {
        // Second arg might be a map of changes.
        let val = ctx.eval_expr(&args[1])?;
        if let Value::Map(entries) = &val {
            for (k, v) in entries {
                let key = k.to_string_value();
                let val_num = match v {
                    Value::Number(n) => n.value,
                    _ => continue,
                };
                result = match key.as_str() {
                    "hue" => result.with_hue(val_num),
                    "saturation" => result.with_saturation(val_num / 100.0),
                    "lightness" => result.with_lightness(val_num / 100.0),
                    "alpha" => result.with_alpha(val_num),
                    _ => result,
                };
            }
        }
    }
    Ok(Value::Color(result))
}

/// Evaluate a number argument at the given index.
fn eval_number(name: &str, args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<f64, EvalError> {
    if args.is_empty() {
        return Err(EvalError::ArityMismatch(name.into(), "more".into(), 0));
    }
    let val = ctx.eval_expr(&args[0])?;
    match &val {
        Value::Number(n) => Ok(n.value),
        _ => Err(EvalError::type_error("number", val.type_name())),
    }
}
