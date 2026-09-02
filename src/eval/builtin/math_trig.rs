//! Math 三角函数 + pow/log/hypot。
//!
//! sin/cos/tan/asin/acos/atan/pow/sqrt/log/hypot/atan2。
//! Calc 参数透传——当参数含 Calc 时返回 `func(args)` 字符串。

use super::math_helpers::validate_single_number;
use crate::error::{Result, SassError};
use crate::parse::ast::*;

/// 单参数三角/数学函数（sin/cos/tan/sqrt）。
/// Calc 参数透传为 `func(inner)` 字符串。
fn unary_math_func(
    args: &[Value],
    func_name: &str,
    f: impl Fn(f64) -> f64,
) -> Result<Option<Value>> {
    validate_single_number(args)?;
    match &args[0] {
        Value::Number(n, _) => Ok(Some(Value::Number(f(*n), None))),
        Value::Calc(c) => {
            let inner = c
                .strip_prefix("calc(")
                .and_then(|s| s.strip_suffix(")"))
                .unwrap_or(c.as_str());
            Ok(Some(Value::String(format!("{func_name}({inner})"), false)))
        }
        _ => unreachable!(),
    }
}

/// 单参数反三角函数（asin/acos/atan）——返回 deg 单位。
fn inverse_trig_func(
    args: &[Value],
    func_name: &str,
    f: impl Fn(f64) -> f64,
) -> Result<Option<Value>> {
    validate_single_number(args)?;
    match &args[0] {
        Value::Number(n, _) => {
            let result = f(*n).to_degrees();
            Ok(Some(Value::Number(result, Some("deg".to_string()))))
        }
        Value::Calc(c) => {
            let inner = c
                .strip_prefix("calc(")
                .and_then(|s| s.strip_suffix(")"))
                .unwrap_or(c.as_str());
            Ok(Some(Value::String(format!("{func_name}({inner})"), false)))
        }
        _ => unreachable!(),
    }
}

/// 将 Value 转换为 `数字+单位` 字符串（用于 Calc 透传）。
fn value_to_str(v: &Value) -> Result<String> {
    match v {
        Value::Number(n, u) => Ok(format!("{n}{}", u.as_deref().unwrap_or(""))),
        Value::Calc(c) => Ok(c
            .strip_prefix("calc(")
            .and_then(|s| s.strip_suffix(")"))
            .unwrap_or(c.as_str())
            .to_string()),
        _ => Err(SassError::Eval(format!("{v} is not a number."))),
    }
}

/// 提取无单位数字——报错如果有单位。
fn extract_unitless(v: &Value, param: &str) -> Result<f64> {
    match v {
        Value::Number(n, u) => {
            if u.is_some() {
                return Err(SassError::Eval(format!(
                    "${param}: Expected {n}{} to have no units.",
                    u.as_deref().unwrap_or("")
                )));
            }
            Ok(*n)
        }
        _ => Err(SassError::Eval(format!("${param}: {v} is not a number."))),
    }
}

/// Math 三角/pow/log/hypot 函数分派。
pub fn call(name: &str, args: &[Value]) -> Result<Option<Value>> {
    match name {
        "sqrt" => unary_math_func(args, "sqrt", f64::sqrt),
        "sin" => unary_math_func(args, "sin", f64::sin),
        "cos" => unary_math_func(args, "cos", f64::cos),
        "tan" => unary_math_func(args, "tan", f64::tan),
        "asin" => inverse_trig_func(args, "asin", f64::asin),
        "acos" => inverse_trig_func(args, "acos", f64::acos),
        "atan" => inverse_trig_func(args, "atan", f64::atan),
        "pow" => call_pow(args),
        "atan2" => call_atan2(args),
        "log" => call_log(args),
        "hypot" => call_hypot(args),
        _ => Ok(None),
    }
}

/// pow(base, exponent)——Calc 透传。
fn call_pow(args: &[Value]) -> Result<Option<Value>> {
    if args.is_empty() {
        return Err(SassError::Eval("Missing argument $base.".into()));
    }
    if args.len() < 2 {
        return Err(SassError::Eval("Missing argument $exponent.".into()));
    }
    if args.len() > 2 {
        return Err(SassError::Eval(format!(
            "Only 2 arguments allowed, but {} were passed.",
            args.len()
        )));
    }
    let a_is_calc = matches!(&args[0], Value::Calc(..));
    let b_is_calc = matches!(&args[1], Value::Calc(..));
    if a_is_calc || b_is_calc {
        let a_str = value_to_str(&args[0]).map_err(|e| SassError::Eval(format!("$base: {e}")))?;
        let b_str =
            value_to_str(&args[1]).map_err(|e| SassError::Eval(format!("$exponent: {e}")))?;
        return Ok(Some(Value::String(format!("pow({a_str}, {b_str})"), false)));
    }
    let a = extract_unitless(&args[0], "base")?;
    let b = extract_unitless(&args[1], "exponent")?;
    Ok(Some(Value::Number(a.powf(b), None)))
}

/// atan2(y, x)——返回 deg 单位，Calc 透传。
fn call_atan2(args: &[Value]) -> Result<Option<Value>> {
    if args.is_empty() {
        return Err(SassError::Eval("Missing argument $y.".into()));
    }
    if args.len() < 2 {
        return Err(SassError::Eval("Missing argument $x.".into()));
    }
    if args.len() > 2 {
        return Err(SassError::Eval(format!(
            "Only 2 arguments allowed, but {} were passed.",
            args.len()
        )));
    }
    let y_is_calc = matches!(&args[0], Value::Calc(..));
    let x_is_calc = matches!(&args[1], Value::Calc(..));
    if y_is_calc || x_is_calc {
        let y_str = value_to_str(&args[0]).map_err(|e| SassError::Eval(format!("$y: {e}")))?;
        let x_str = value_to_str(&args[1]).map_err(|e| SassError::Eval(format!("$x: {e}")))?;
        return Ok(Some(Value::String(
            format!("atan2({y_str}, {x_str})"),
            false,
        )));
    }
    let (y, uy) = match &args[0] {
        Value::Number(n, u) => (*n, u.clone()),
        other => return Err(SassError::Eval(format!("$y: {other} is not a number."))),
    };
    let (x, ux) = match &args[1] {
        Value::Number(n, u) => (*n, u.clone()),
        other => return Err(SassError::Eval(format!("$x: {other} is not a number."))),
    };
    if !crate::eval::value::units_compatible(uy.as_deref(), ux.as_deref()) {
        let u1_str = uy.as_deref().unwrap_or("");
        let u2_str = ux.as_deref().unwrap_or("");
        if (uy.is_some() && ux.is_none()) || (uy.is_none() && ux.is_some()) {
            return Err(SassError::Eval(format!(
                "$x: {x}{u2_str} and $y: {y}{u1_str} have incompatible units (one has units and the other doesn't)."
            )));
        }
        return Err(SassError::Eval(format!(
            "$x: {x}{u2_str} and $y: {y}{u1_str} have incompatible units."
        )));
    }
    let result = y.atan2(x).to_degrees();
    Ok(Some(Value::Number(result, Some("deg".to_string()))))
}

/// log(number, base?)——base 为 null 时自然对数。
fn call_log(args: &[Value]) -> Result<Option<Value>> {
    if args.is_empty() {
        return Err(SassError::Eval("Missing argument $number.".into()));
    }
    if args.len() > 2 {
        return Err(SassError::Eval(format!(
            "Only 2 arguments allowed, but {} were passed.",
            args.len()
        )));
    }
    let n_is_calc = matches!(&args[0], Value::Calc(..));
    let b_is_calc = args.len() == 2 && matches!(&args[1], Value::Calc(..));
    if n_is_calc || b_is_calc {
        let n_str = value_to_str(&args[0]).map_err(|e| SassError::Eval(format!("$number: {e}")))?;
        if args.len() == 2 && !matches!(&args[1], Value::Null) {
            let b_str =
                value_to_str(&args[1]).map_err(|e| SassError::Eval(format!("$base: {e}")))?;
            return Ok(Some(Value::String(format!("log({n_str}, {b_str})"), false)));
        }
        return Ok(Some(Value::String(format!("log({n_str})"), false)));
    }
    let n = extract_unitless(&args[0], "number")?;
    if n < 0.0 {
        return Ok(Some(Value::Calc("calc(NaN)".to_string())));
    }
    if n == 0.0 {
        return Ok(Some(Value::Calc("calc(-infinity)".to_string())));
    }
    if args.len() == 2 {
        if matches!(&args[1], Value::Null) {
            return Ok(Some(Value::Number(n.ln(), None)));
        }
        let base = extract_unitless(&args[1], "base")?;
        return Ok(Some(Value::Number(n.log(base), None)));
    }
    Ok(Some(Value::Number(n.ln(), None)))
}

/// hypot(numbers...)——向量的欧几里得范数。
fn call_hypot(args: &[Value]) -> Result<Option<Value>> {
    if args.is_empty() {
        return Err(SassError::Eval("Missing argument $numbers.".into()));
    }
    let any_calc = args.iter().any(|a| matches!(a, Value::Calc(..)));
    if any_calc {
        let strs: Result<Vec<String>> = args.iter().map(value_to_str).collect();
        let strs = strs?;
        return Ok(Some(Value::String(
            format!("hypot({})", strs.join(", ")),
            false,
        )));
    }
    let mut nums: Vec<(f64, Option<String>)> = Vec::new();
    for (i, a) in args.iter().enumerate() {
        match a {
            Value::Number(n, u) => nums.push((*n, u.clone())),
            other => return Err(SassError::Eval(format!("{other} is not a number."))),
        }
        if i > 0 {
            let u0 = nums[0].1.as_deref();
            let ui = nums[i].1.as_deref();
            if !crate::eval::value::units_compatible(u0, ui) {
                return Err(SassError::Eval(format!(
                    "$numbers[{}]: {}{} and $numbers[1]: {}{} have incompatible units.",
                    i + 1,
                    nums[i].0,
                    ui.unwrap_or(""),
                    nums[0].0,
                    u0.unwrap_or("")
                )));
            }
        }
    }
    let sum: f64 = nums.iter().map(|(n, _)| n * n).sum();
    Ok(Some(Value::Number(sum.sqrt(), nums[0].1.clone())))
}
