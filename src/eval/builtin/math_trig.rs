//! Math 三角函数 + pow/log/hypot。
//!
//! sin/cos/tan/asin/acos/atan/pow/sqrt/log/hypot/atan2。
//! Calc 参数透传——当参数含 Calc 时返回 `func(args)` 字符串。

use super::math_helpers::validate_single_number;
use crate::error::{Result, SassError};
use crate::parse::ast::*;

/// 将角度单位转为弧度。
/// 接受的单位：`deg`、`rad`、`grad`、`turn`、无单位（弧度）。
/// 其它单位报错。
fn angle_to_radians(n: f64, unit: Option<&String>, param: &str) -> Result<f64> {
    let rad = match unit.map(String::as_str) {
        None => n,
        Some("rad") => n,
        Some("deg") => n * std::f64::consts::PI / 180.0,
        Some("grad") => n * std::f64::consts::PI / 200.0,
        Some("turn") => n * 2.0 * std::f64::consts::PI,
        Some(other) => {
            return Err(SassError::Eval(format!(
                "${param}: Expected {n}{other} to have an angle unit (deg, rad, grad, turn) or no units."
            )))
        }
    };
    Ok(rad)
}

/// 单参数三角函数（sin/cos/tan）——支持角度单位转换。
/// 接受 deg/rad/grad/turn/unitless(弧度)，其它角度单位报错。
fn trig_func(
    args: &[Value],
    func_name: &str,
    f: impl Fn(f64) -> f64,
) -> Result<Option<Value>> {
    validate_single_number(args)?;
    match &args[0] {
        Value::Number(n, unit) => {
            let rad = angle_to_radians(*n, unit.as_ref(), "number")?;
            Ok(Some(Value::Number(f(rad), None)))
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

/// 单参数 math 函数——严格 unitless（sqrt）。
/// 计算前校验无单位。
fn unitless_unary_func(
    args: &[Value],
    func_name: &str,
    f: impl Fn(f64) -> f64,
) -> Result<Option<Value>> {
    validate_single_number(args)?;
    let n = extract_unitless(&args[0], "number")?;
    match &args[0] {
        Value::Number(_, _) => Ok(Some(Value::Number(f(n), None))),
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

/// 单参数反三角函数（asin/acos/atan）——参数 unitless，返回 deg。
/// 参数含变量或无法编译时求值时保留函数形式。
fn inverse_trig_func(
    args: &[Value],
    func_name: &str,
    f: impl Fn(f64) -> f64,
) -> Result<Option<Value>> {
    validate_single_number(args)?;
    // 含变量或字符串表达式 → 保留函数形式
    if let Some(arg_str) = inverse_trig_arg_str(&args[0]) {
        return Ok(Some(Value::String(format!("{func_name}({arg_str})"), false)));
    }
    let n = extract_unitless(&args[0], "number")?;
    match &args[0] {
        Value::Number(_, _) => {
            let result = f(n).to_degrees();
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

/// 当反三角函数参数含变量/字符串时，返回应保留的参数字符串。
/// 纯数字/单位返回 None 以走编译时求值路径。
fn inverse_trig_arg_str(v: &Value) -> Option<String> {
    match v {
        Value::Variable(name) => Some(name.clone()),
        Value::String(s, _) => match s.contains("var(") || s.contains('$') {
            true => Some(s.clone()),
            false => None,
        },
        Value::Interp(segments) => {
            // 检查 Text 段是否含 var( 或 $，Expr 段保留为占位符
            let mut has_var = false;
            let mut combined = String::new();
            for seg in segments {
                match seg {
                    crate::parse::ast::InterpSegment::Text(t) => {
                        if t.contains("var(") || t.contains('$') {
                            has_var = true;
                        }
                        combined.push_str(t);
                    }
                    crate::parse::ast::InterpSegment::Expr(_) => {
                        combined.push_str("#{}");
                    }
                }
            }
            match has_var {
                true => Some(combined),
                false => None,
            }
        }
        _ => None,
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
/// 特殊浮点常量（infinity、-infinity、NaN）作为字符串值传入时映射为 f64 特殊值。
fn extract_unitless(v: &Value, param: &str) -> Result<f64> {
    match v {
        Value::Number(n, u) => match u.is_some() {
            true => Err(SassError::Eval(format!(
                "${param}: Expected {n}{} to have no units.",
                u.as_deref().unwrap_or("")
            ))),
            false => Ok(*n),
        },
        Value::String(s, _) => match s.trim() {
            "infinity" => Ok(f64::INFINITY),
            "-infinity" => Ok(f64::NEG_INFINITY),
            "nan" | "NaN" => Ok(f64::NAN),
            _ => Err(SassError::Eval(format!("${param}: {v} is not a number."))),
        },
        _ => Err(SassError::Eval(format!("${param}: {v} is not a number."))),
    }
}

/// Math 三角/pow/log/hypot 函数分派。
pub fn call(name: &str, args: &[Value]) -> Result<Option<Value>> {
    match name {
        "sqrt" => unitless_unary_func(args, "sqrt", f64::sqrt),
        "sin" => trig_func(args, "sin", f64::sin),
        "cos" => trig_func(args, "cos", f64::cos),
        "tan" => trig_func(args, "tan", f64::tan),
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
    match args.is_empty() {
        true => return Err(SassError::Eval("Missing argument $base.".into())),
        false => {}
    }
    match args.len() < 2 {
        true => return Err(SassError::Eval("Missing argument $exponent.".into())),
        false => {}
    }
    match args.len() > 2 {
        true => return Err(SassError::Eval(format!(
            "Only 2 arguments allowed, but {} were passed.",
            args.len()
        ))),
        false => {}
    }
    let a_is_calc = matches!(&args[0], Value::Calc(..));
    let b_is_calc = matches!(&args[1], Value::Calc(..));
    match a_is_calc || b_is_calc {
        true => {
            let a_str = value_to_str(&args[0]).map_err(|e| SassError::Eval(format!("$base: {e}")))?;
            let b_str =
                value_to_str(&args[1]).map_err(|e| SassError::Eval(format!("$exponent: {e}")))?;
            return Ok(Some(Value::String(format!("pow({a_str}, {b_str})"), false)));
        }
        false => {}
    }
    let a = extract_unitless(&args[0], "base")?;
    let b = extract_unitless(&args[1], "exponent")?;
    Ok(Some(Value::Number(a.powf(b), None)))
}

/// atan2(y, x)——返回 deg 单位，Calc 透传。
fn call_atan2(args: &[Value]) -> Result<Option<Value>> {
    match args.is_empty() {
        true => return Err(SassError::Eval("Missing argument $y.".into())),
        false => {}
    }
    match args.len() < 2 {
        true => return Err(SassError::Eval("Missing argument $x.".into())),
        false => {}
    }
    match args.len() > 2 {
        true => return Err(SassError::Eval(format!(
            "Only 2 arguments allowed, but {} were passed.",
            args.len()
        ))),
        false => {}
    }
    let y_is_calc = matches!(&args[0], Value::Calc(..));
    let x_is_calc = matches!(&args[1], Value::Calc(..));
    match y_is_calc || x_is_calc {
        true => {
            let y_str = value_to_str(&args[0]).map_err(|e| SassError::Eval(format!("$y: {e}")))?;
            let x_str = value_to_str(&args[1]).map_err(|e| SassError::Eval(format!("$x: {e}")))?;
            return Ok(Some(Value::String(
                format!("atan2({y_str}, {x_str})"),
                false,
            )));
        }
        false => {}
    }
    let (y, uy) = match &args[0] {
        Value::Number(n, u) => (*n, u.clone()),
        other => return Err(SassError::Eval(format!("$y: {other} is not a number."))),
    };
    let (x, ux) = match &args[1] {
        Value::Number(n, u) => (*n, u.clone()),
        other => return Err(SassError::Eval(format!("$x: {other} is not a number."))),
    };
    // atan2 要求两个参数同为 unitless 或同为有单位（不可混用）
    if uy.is_some() != ux.is_some() {
        let u1_str = uy.as_deref().unwrap_or("");
        let u2_str = ux.as_deref().unwrap_or("");
        return Err(SassError::Eval(format!(
            "$x: {x}{u2_str} and $y: {y}{u1_str} have incompatible units (one has units and the other doesn't)."
        )));
    }
    if !crate::eval::value::units_compatible(uy.as_deref(), ux.as_deref()) {
        let u1_str = uy.as_deref().unwrap_or("");
        let u2_str = ux.as_deref().unwrap_or("");
        return Err(SassError::Eval(format!(
            "$x: {x}{u2_str} and $y: {y}{u1_str} have incompatible units."
        )));
    }
    let result = y.atan2(x).to_degrees();
    // 保留负零的符号：atan2(-0.0, +infinity) 应该返回 -0.0
    let result = if result == 0.0 && y.is_sign_negative() {
        -0.0
    } else {
        result
    };
    Ok(Some(Value::Number(result, Some("deg".to_string()))))
}

/// log(number, base?)——base 为 null 时自然对数。
fn call_log(args: &[Value]) -> Result<Option<Value>> {
    match args.is_empty() {
        true => return Err(SassError::Eval("Missing argument $number.".into())),
        false => {}
    }
    match args.len() > 2 {
        true => return Err(SassError::Eval(format!(
            "Only 2 arguments allowed, but {} were passed.",
            args.len()
        ))),
        false => {}
    }
    let n_is_calc = matches!(&args[0], Value::Calc(..));
    let b_is_calc = args.len() == 2 && matches!(&args[1], Value::Calc(..));
    match n_is_calc || b_is_calc {
        true => {
            let n_str = value_to_str(&args[0]).map_err(|e| SassError::Eval(format!("$number: {e}")))?;
            match args.len() == 2 && !matches!(&args[1], Value::Null) {
                true => {
                    let b_str =
                        value_to_str(&args[1]).map_err(|e| SassError::Eval(format!("$base: {e}")))?;
                    return Ok(Some(Value::String(format!("log({n_str}, {b_str})"), false)));
                }
                false => return Ok(Some(Value::String(format!("log({n_str})"), false))),
            }
        }
        false => {}
    }
    let n = extract_unitless(&args[0], "number")?;
    match n < 0.0 {
        true => return Ok(Some(Value::Calc("calc(NaN)".to_string()))),
        false => {}
    }
    match n == 0.0 {
        true => return Ok(Some(Value::Calc("calc(-infinity)".to_string()))),
        false => {}
    }
    match args.len() == 2 {
        true => match matches!(&args[1], Value::Null) {
            true => return Ok(Some(Value::Number(n.ln(), None))),
            false => {
                let base = extract_unitless(&args[1], "base")?;
                return Ok(Some(Value::Number(n.log(base), None)));
            }
        },
        false => {}
    }
    Ok(Some(Value::Number(n.ln(), None)))
}

/// hypot(numbers...)——向量的欧几里得范数。
fn call_hypot(args: &[Value]) -> Result<Option<Value>> {
    match args.is_empty() {
        true => return Err(SassError::Eval("Missing argument $numbers.".into())),
        false => {}
    }
    let any_calc = args.iter().any(|a| matches!(a, Value::Calc(..)));
    match any_calc {
        true => {
            let strs: Result<Vec<String>> = args.iter().map(value_to_str).collect();
            let strs = strs?;
            return Ok(Some(Value::String(
                format!("hypot({})", strs.join(", ")),
                false,
            )));
        }
        false => {}
    }
    let mut nums: Vec<(f64, Option<String>)> = Vec::new();
    for a in args {
        match a {
            Value::Number(n, u) => nums.push((*n, u.clone())),
            other => return Err(SassError::Eval(format!("{other} is not a number."))),
        }
    }
    // 检查：所有参数必须同为 unitless 或同为有单位（不可混用）
    let first_has_unit = nums[0].1.is_some();
    for (i, (_, u)) in nums.iter().enumerate().skip(1) {
        let this_has_unit = u.is_some();
        if first_has_unit != this_has_unit {
            let u0 = nums[0].1.as_deref().unwrap_or("");
            let ui = u.as_deref().unwrap_or("");
            return Err(SassError::Eval(format!(
                "$numbers[{}]: {}{} and $numbers[1]: {}{} have incompatible units.",
                i + 1,
                nums[i].0,
                ui,
                nums[0].0,
                u0
            )));
        }
    }
    // 检查单位兼容性（有单位时）
    for (i, (_, u)) in nums.iter().enumerate().skip(1) {
        let u0 = nums[0].1.as_deref();
        let ui = u.as_deref();
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
    let sum: f64 = nums.iter().map(|(n, _)| n * n).sum();
    Ok(Some(Value::Number(sum.sqrt(), nums[0].1.clone())))
}
