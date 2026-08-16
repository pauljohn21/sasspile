//! sass:math module — mathematical functions and constants.

use crate::eval::error::EvalError;
use crate::eval::evaluator::EvalContext;
use crate::parser::Expr;
use crate::value::{Number, Unit, Value};

/// Dispatch a sass:math function call.
pub fn call(
    func: &str,
    args: &[Expr],
    ctx: &mut EvalContext<'_>,
) -> Result<Option<Value>, EvalError> {
    match func {
        "pi" => Ok(Some(Value::Number(Number::new(std::f64::consts::PI, Unit::None)))),
        "e" => Ok(Some(Value::Number(Number::new(std::f64::consts::E, Unit::None)))),
        "ceil" => ceil(args, ctx).map(Some),
        "floor" => floor(args, ctx).map(Some),
        "round" => round(args, ctx).map(Some),
        "abs" => abs(args, ctx).map(Some),
        "min" => min(args, ctx).map(Some),
        "max" => max(args, ctx).map(Some),
        "random" => random(args, ctx).map(Some),
        "percentage" => percentage(args, ctx).map(Some),
        "sin" => trig(args, ctx, |v| v.sin()).map(Some),
        "cos" => trig(args, ctx, |v| v.cos()).map(Some),
        "tan" => trig(args, ctx, |v| v.tan()).map(Some),
        "asin" => trig(args, ctx, |v| v.asin()).map(Some),
        "acos" => trig(args, ctx, |v| v.acos()).map(Some),
        "atan" => trig(args, ctx, |v| v.atan()).map(Some),
        "atan2" => atan2(args, ctx).map(Some),
        "pow" => pow(args, ctx).map(Some),
        "sqrt" => sqrt(args, ctx).map(Some),
        "log" => log(args, ctx).map(Some),
        "log10" => log10(args, ctx).map(Some),
        "hypot" => hypot(args, ctx).map(Some),
        "unit" => unit(args, ctx).map(Some),
        "compatible" => compatible(args, ctx).map(Some),
        "unitless" => unitless(args, ctx).map(Some),
        _ => Ok(None),
    }
}

/// Evaluate a single number argument.
fn eval_number(name: &str, args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<f64, EvalError> {
    if args.is_empty() {
        return Err(EvalError::ArityMismatch(name.into(), "1+".into(), 0));
    }
    let val = ctx.eval_expr(&args[0])?;
    match &val {
        Value::Number(n) => Ok(n.value),
        _ => Err(EvalError::type_error("number", val.type_name())),
    }
}

/// Ceiling of a number.
fn ceil(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let n = eval_number("ceil", args, ctx)?;
    Ok(Value::Number(Number::new(n.ceil(), Unit::None)))
}

/// Floor of a number.
fn floor(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let n = eval_number("floor", args, ctx)?;
    Ok(Value::Number(Number::new(n.floor(), Unit::None)))
}

/// Round a number.
fn round(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let n = eval_number("round", args, ctx)?;
    Ok(Value::Number(Number::new(n.round(), Unit::None)))
}

/// Absolute value.
fn abs(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let n = eval_number("abs", args, ctx)?;
    Ok(Value::Number(Number::new(n.abs(), Unit::None)))
}

/// Minimum of a list of numbers.
fn min(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let values: Result<Vec<_>, _> = args.iter().map(|a| ctx.eval_expr(a)).collect();
    let values = values?;
    if values.is_empty() {
        return Err(EvalError::ArityMismatch("min".into(), "1+".into(), 0));
    }
    let mut min_val = match &values[0] {
        Value::Number(n) => n.clone(),
        _ => return Err(EvalError::TypeError("expected number".into())),
    };
    for val in &values[1..] {
        match val {
            Value::Number(n) => {
                if n.value < min_val.value {
                    min_val = n.clone();
                }
            }
            _ => return Err(EvalError::TypeError("expected number".into())),
        }
    }
    Ok(Value::Number(min_val))
}

/// Maximum of a list of numbers.
fn max(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let values: Result<Vec<_>, _> = args.iter().map(|a| ctx.eval_expr(a)).collect();
    let values = values?;
    if values.is_empty() {
        return Err(EvalError::ArityMismatch("max".into(), "1+".into(), 0));
    }
    let mut max_val = match &values[0] {
        Value::Number(n) => n.clone(),
        _ => return Err(EvalError::TypeError("expected number".into())),
    };
    for val in &values[1..] {
        match val {
            Value::Number(n) => {
                if n.value > max_val.value {
                    max_val = n.clone();
                }
            }
            _ => return Err(EvalError::TypeError("expected number".into())),
        }
    }
    Ok(Value::Number(max_val))
}

/// Generate a random number (0-1 or 0-n).
fn random(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let limit = if args.is_empty() {
        None
    } else {
        Some(eval_number("random", args, ctx)? as i64)
    };
    match limit {
        Some(n) if n > 0 => {
            let r: f64 = rand::random();
            Ok(Value::Number(Number::unitless((r * n as f64).floor() as i64 as f64)))
        }
        _ => {
            let r: f64 = rand::random();
            Ok(Value::Number(Number::unitless(r)))
        }
    }
}

/// Convert unitless number to percentage.
fn percentage(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let n = eval_number("percentage", args, ctx)?;
    Ok(Value::Number(Number::new(n * 100.0, Unit::Percent)))
}

/// Generic trig function.
fn trig(args: &[Expr], ctx: &mut EvalContext<'_>, f: impl Fn(f64) -> f64) -> Result<Value, EvalError> {
    let n = eval_number("trig", args, ctx)?;
    Ok(Value::Number(Number::new(f(n), Unit::None)))
}

/// atan2(y, x).
fn atan2(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let y = eval_number("atan2", args, ctx)?;
    let x = if args.len() >= 2 {
        eval_number("atan2", &args[1..], ctx)?
    } else {
        return Err(EvalError::ArityMismatch("atan2".into(), "2".into(), args.len()));
    };
    Ok(Value::Number(Number::new(y.atan2(x), Unit::None)))
}

/// pow(base, exponent).
fn pow(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let base = eval_number("pow", args, ctx)?;
    let exp = if args.len() >= 2 {
        eval_number("pow", &args[1..], ctx)?
    } else {
        return Err(EvalError::ArityMismatch("pow".into(), "2".into(), args.len()));
    };
    Ok(Value::Number(Number::new(base.powf(exp), Unit::None)))
}

/// sqrt(x).
fn sqrt(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let n = eval_number("sqrt", args, ctx)?;
    Ok(Value::Number(Number::new(n.sqrt(), Unit::None)))
}

/// log(x) natural logarithm.
fn log(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let n = eval_number("log", args, ctx)?;
    Ok(Value::Number(Number::new(n.ln(), Unit::None)))
}

/// log10(x).
fn log10(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let n = eval_number("log10", args, ctx)?;
    Ok(Value::Number(Number::new(n.log10(), Unit::None)))
}

/// hypot(x, y).
fn hypot(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let x = eval_number("hypot", args, ctx)?;
    let y = if args.len() >= 2 {
        eval_number("hypot", &args[1..], ctx)?
    } else {
        return Err(EvalError::ArityMismatch("hypot".into(), "2+".into(), args.len()));
    };
    Ok(Value::Number(Number::new((x * x + y * y).sqrt(), Unit::None)))
}

/// Get the unit of a number as a string.
fn unit(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let val = ctx.eval_expr(&args[0])?;
    match &val {
        Value::Number(n) => {
            let unit_str = format!("{:?}", n.unit);
            Ok(Value::String(unit_str, crate::value::Quoted::Quoted))
        }
        _ => Err(EvalError::type_error("number", val.type_name())),
    }
}

/// Check if two numbers are unit-compatible.
fn compatible(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let a = ctx.eval_expr(&args[0])?;
    let b = ctx.eval_expr(&args[1])?;
    match (&a, &b) {
        (Value::Number(na), Value::Number(nb)) => {
            Ok(Value::Boolean(na.unit.is_compatible(&nb.unit)))
        }
        _ => Err(EvalError::TypeError("expected two numbers".into())),
    }
}

/// Check if a number is unitless.
fn unitless(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let val = ctx.eval_expr(&args[0])?;
    match &val {
        Value::Number(n) => Ok(Value::Boolean(n.unit == Unit::None)),
        _ => Err(EvalError::type_error("number", val.type_name())),
    }
}
