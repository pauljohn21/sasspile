//! sass:math 内建函数。

use crate::error::{Result, SassError};
use crate::parse::ast::Value;

/// 断言数值参数。
fn assert_number(arg: &Value) -> Result<(f64, Option<String>)> {
    match arg {
        Value::Number(n, unit) => Ok((*n, unit.clone())),
        _ => Err(SassError::TypeError {
            expected: "number".to_string(),
            actual: "other".to_string(),
        }),
    }
}

/// 获取第一个参数。
fn first_arg(args: &[Value]) -> Result<&Value> {
    args.first()
        .ok_or_else(|| SassError::EvalError("函数需要至少 1 个参数".to_string()))
}

/// 转换角度到弧度。
fn to_radians(n: f64, unit: &Option<String>) -> f64 {
    match unit.as_deref() {
        Some("deg") => n.to_radians(),
        Some("grad") => n * std::f64::consts::PI / 200.0,
        Some("turn") => n * 2.0 * std::f64::consts::PI,
        _ => n,
    }
}

// ── 基础运算 ──

pub fn abs(args: &[Value]) -> Result<Value> {
    let (n, unit) = assert_number(first_arg(args)?)?;
    Ok(Value::Number(n.abs(), unit))
}

pub fn ceil(args: &[Value]) -> Result<Value> {
    let (n, unit) = assert_number(first_arg(args)?)?;
    Ok(Value::Number(n.ceil(), unit))
}

pub fn floor(args: &[Value]) -> Result<Value> {
    let (n, unit) = assert_number(first_arg(args)?)?;
    Ok(Value::Number(n.floor(), unit))
}

pub fn round(args: &[Value]) -> Result<Value> {
    let (n, unit) = assert_number(first_arg(args)?)?;
    Ok(Value::Number(n.round(), unit))
}

pub fn clamp(args: &[Value]) -> Result<Value> {
    let (min, _) = assert_number(
        args.first()
            .ok_or_else(|| SassError::EvalError("clamp 需要 3 个参数".to_string()))?,
    )?;
    let (n, unit) = assert_number(
        args.get(1)
            .ok_or_else(|| SassError::EvalError("clamp 需要 3 个参数".to_string()))?,
    )?;
    let (max, _) = assert_number(
        args.get(2)
            .ok_or_else(|| SassError::EvalError("clamp 需要 3 个参数".to_string()))?,
    )?;
    Ok(Value::Number(n.clamp(min, max), unit))
}

pub fn min(args: &[Value]) -> Result<Value> {
    let mut min_val = assert_number(first_arg(args)?)?;
    for arg in &args[1..] {
        let (n, unit) = assert_number(arg)?;
        if n < min_val.0 {
            min_val = (n, unit);
        }
    }
    Ok(Value::Number(min_val.0, min_val.1))
}

pub fn max(args: &[Value]) -> Result<Value> {
    let mut max_val = assert_number(first_arg(args)?)?;
    for arg in &args[1..] {
        let (n, unit) = assert_number(arg)?;
        if n > max_val.0 {
            max_val = (n, unit);
        }
    }
    Ok(Value::Number(max_val.0, max_val.1))
}

pub fn percentage(args: &[Value]) -> Result<Value> {
    let (n, _) = assert_number(first_arg(args)?)?;
    Ok(Value::Number(n * 100.0, Some("%".to_string())))
}

pub fn compatible(args: &[Value]) -> Result<Value> {
    let (_, unit1) = assert_number(
        args.first()
            .ok_or_else(|| SassError::EvalError("compatible 需要 2 个参数".to_string()))?,
    )?;
    let (_, unit2) = assert_number(
        args.get(1)
            .ok_or_else(|| SassError::EvalError("compatible 需要 2 个参数".to_string()))?,
    )?;
    Ok(Value::Bool(
        unit1 == unit2 || unit1.is_none() || unit2.is_none(),
    ))
}

pub fn is_unitless(args: &[Value]) -> Result<Value> {
    let (_, unit) = assert_number(first_arg(args)?)?;
    Ok(Value::Bool(unit.is_none()))
}

// ── 幂/根/对数 ──

pub fn sqrt(args: &[Value]) -> Result<Value> {
    let (n, _) = assert_number(first_arg(args)?)?;
    Ok(Value::Number(n.sqrt(), None))
}

pub fn pow(args: &[Value]) -> Result<Value> {
    let (base, _) = assert_number(
        args.first()
            .ok_or_else(|| SassError::EvalError("pow 需要 2 个参数".to_string()))?,
    )?;
    let (exp, _) = assert_number(
        args.get(1)
            .ok_or_else(|| SassError::EvalError("pow 需要 2 个参数".to_string()))?,
    )?;
    Ok(Value::Number(base.powf(exp), None))
}

pub fn log(args: &[Value]) -> Result<Value> {
    let (n, _) = assert_number(
        args.first()
            .ok_or_else(|| SassError::EvalError("log 需要 1-2 个参数".to_string()))?,
    )?;
    if args.len() >= 2 {
        let (base, _) = assert_number(args.get(1).unwrap())?;
        Ok(Value::Number(n.log(base), None))
    } else {
        Ok(Value::Number(n.ln(), None))
    }
}

pub fn hypot(args: &[Value]) -> Result<Value> {
    let mut sum_sq = 0.0;
    for arg in args {
        let (n, _) = assert_number(arg)?;
        sum_sq += n * n;
    }
    Ok(Value::Number(sum_sq.sqrt(), None))
}

// ── 三角函数 ──

pub fn sin(args: &[Value]) -> Result<Value> {
    let (n, unit) = assert_number(first_arg(args)?)?;
    Ok(Value::Number(to_radians(n, &unit).sin(), None))
}

pub fn cos(args: &[Value]) -> Result<Value> {
    let (n, unit) = assert_number(first_arg(args)?)?;
    Ok(Value::Number(to_radians(n, &unit).cos(), None))
}

pub fn tan(args: &[Value]) -> Result<Value> {
    let (n, unit) = assert_number(first_arg(args)?)?;
    Ok(Value::Number(to_radians(n, &unit).tan(), None))
}

pub fn asin(args: &[Value]) -> Result<Value> {
    let (n, _) = assert_number(first_arg(args)?)?;
    Ok(Value::Number(n.asin().to_degrees(), None))
}

pub fn acos(args: &[Value]) -> Result<Value> {
    let (n, _) = assert_number(first_arg(args)?)?;
    Ok(Value::Number(n.acos().to_degrees(), None))
}

pub fn atan(args: &[Value]) -> Result<Value> {
    let (n, _) = assert_number(first_arg(args)?)?;
    Ok(Value::Number(n.atan().to_degrees(), None))
}

pub fn atan2(args: &[Value]) -> Result<Value> {
    let (y, _) = assert_number(
        args.first()
            .ok_or_else(|| SassError::EvalError("atan2 需要 2 个参数".to_string()))?,
    )?;
    let (x, _) = assert_number(
        args.get(1)
            .ok_or_else(|| SassError::EvalError("atan2 需要 2 个参数".to_string()))?,
    )?;
    Ok(Value::Number(y.atan2(x).to_degrees(), None))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abs() {
        assert_eq!(
            abs(&[Value::Number(-10.0, None)]).unwrap(),
            Value::Number(10.0, None)
        );
    }

    #[test]
    fn test_ceil() {
        assert_eq!(
            ceil(&[Value::Number(3.2, None)]).unwrap(),
            Value::Number(4.0, None)
        );
    }

    #[test]
    fn test_floor() {
        assert_eq!(
            floor(&[Value::Number(3.8, None)]).unwrap(),
            Value::Number(3.0, None)
        );
    }

    #[test]
    fn test_round() {
        assert_eq!(
            round(&[Value::Number(3.5, None)]).unwrap(),
            Value::Number(4.0, None)
        );
    }

    #[test]
    fn test_sqrt() {
        assert_eq!(
            sqrt(&[Value::Number(16.0, None)]).unwrap(),
            Value::Number(4.0, None)
        );
    }

    #[test]
    fn test_pow() {
        assert_eq!(
            pow(&[Value::Number(2.0, None), Value::Number(3.0, None)]).unwrap(),
            Value::Number(8.0, None)
        );
    }

    #[test]
    fn test_percentage() {
        assert_eq!(
            percentage(&[Value::Number(0.5, None)]).unwrap(),
            Value::Number(50.0, Some("%".to_string()))
        );
    }

    #[test]
    fn test_sin() {
        let result = sin(&[Value::Number(90.0, Some("deg".to_string()))]).unwrap();
        match result {
            Value::Number(n, None) => assert!((n - 1.0).abs() < 1e-10),
            _ => panic!("Expected Number"),
        }
    }

    #[test]
    fn test_cos() {
        let result = cos(&[Value::Number(0.0, None)]).unwrap();
        match result {
            Value::Number(n, None) => assert!((n - 1.0).abs() < 1e-10),
            _ => panic!("Expected Number"),
        }
    }

    #[test]
    fn test_clamp() {
        assert_eq!(
            clamp(&[
                Value::Number(0.0, None),
                Value::Number(150.0, Some("%".to_string())),
                Value::Number(100.0, None)
            ])
            .unwrap(),
            Value::Number(100.0, Some("%".to_string()))
        );
    }
}
