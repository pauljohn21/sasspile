//! sass:math — mathematical builtins.

use crate::reactive::Value;
use crate::Error;

fn get_number(v: &Value) -> std::result::Result<f64, Error> {
    match v {
        Value::Number(n) => Ok(*n),
        _ => Err(Error::msg(format!("math: expected number, got: {v:?}"))),
    }
}

/// `clamp($min, $val, $max)` — restrict a value to a range.
pub fn clamp(args: &[Value]) -> std::result::Result<Value, Error> {
    let min = args.first().map(get_number).transpose()?.unwrap_or(f64::NEG_INFINITY);
    let val = args.get(1).map(get_number).transpose()?.unwrap_or(0.0);
    let max = args.get(2).map(get_number).transpose()?.unwrap_or(f64::INFINITY);

    Ok(Value::Number(val.clamp(min, max)))
}

/// `max($val1, $val2, ...)` — return the largest value.
pub fn max(args: &[Value]) -> std::result::Result<Value, Error> {
    if args.is_empty() {
        return Err(Error::msg("max: requires at least one argument"));
    }
    let mut result = f64::NEG_INFINITY;
    for arg in args {
        let n = get_number(arg)?;
        if n > result {
            result = n;
        }
    }
    Ok(Value::Number(result))
}

/// `min($val1, $val2, ...)` — return the smallest value.
pub fn min(args: &[Value]) -> std::result::Result<Value, Error> {
    if args.is_empty() {
        return Err(Error::msg("min: requires at least one argument"));
    }
    let mut result = f64::INFINITY;
    for arg in args {
        let n = get_number(arg)?;
        if n < result {
            result = n;
        }
    }
    Ok(Value::Number(result))
}

/// `round($val)` — round to nearest integer.
pub fn round(args: &[Value]) -> std::result::Result<Value, Error> {
    let val = args.first().map(get_number).transpose()?.unwrap_or(0.0);
    Ok(Value::Number(val.round()))
}

/// `abs($val)` — absolute value.
pub fn abs(args: &[Value]) -> std::result::Result<Value, Error> {
    let val = args.first().map(get_number).transpose()?.unwrap_or(0.0);
    Ok(Value::Number(val.abs()))
}

/// `percentage($val)` — convert a unitless number to a percentage (×100).
pub fn percentage(args: &[Value]) -> std::result::Result<Value, Error> {
    let val = args.first().map(get_number).transpose()?.unwrap_or(0.0);
    Ok(Value::Number(val * 100.0))
}
