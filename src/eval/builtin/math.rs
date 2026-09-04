//! Math 内建函数。
//!
//! 包含 abs/ceil/floor/round/min/max/percentage/div/pow/sqrt/sin/cos/tan/
//! atan2/asin/acos/atan/hypot/log/random/clamp/unit/is-unitless/compatible/comparable。
//! CSS round/mod/rem 函数：css_round（1-3 `参数+策略+单位转换）、css_mod（floored）、css_rem（truncated`）。
//!
//! 支持命名参数（如 `math.abs($number: 3)`、`math.clamp($min: 0, $number: 1, $max: 2)`）。
//! 辅助函数（参数名映射、合并、验证）在 `math_helpers` 模块中。

use super::super::Evaluator;
use super::math_css::{css_mod, css_rem, css_round};
use super::math_helpers::{merge_math_args, validate_single_number};
use crate::error::{Result, SassError};
use crate::parse::ast::*;
use std::collections::HashMap;
use std::fmt::Write;

/// Math 函数分派。返回 Ok(Some(value)) 表示已处理，Ok(None) 表示不匹配。
pub fn call(
    name: &str,
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
) -> Result<Option<Value>> {
    let args = merge_math_args(pos_args, kw_args, name);
    let args = args.as_slice();

    match name {
        "abs" => {
            validate_single_number(args)?;
            match &args[0] {
                Value::Number(n, u) => Ok(Some(Value::Number(n.abs(), u.clone()))),
                Value::Calc(c) => {
                    let inner = c
                        .strip_prefix("calc(")
                        .and_then(|s| s.strip_suffix(")"))
                        .unwrap_or(c.as_str());
                    Ok(Some(Value::String(format!("abs({inner})"), false)))
                }
                _ => unreachable!(),
            }
        }
        "ceil" => {
            validate_single_number(args)?;
            match &args[0] {
                Value::Number(n, u) => Ok(Some(Value::Number(n.ceil(), u.clone()))),
                Value::Calc(c) => {
                    let inner = c
                        .strip_prefix("calc(")
                        .and_then(|s| s.strip_suffix(")"))
                        .unwrap_or(c.as_str());
                    Ok(Some(Value::String(format!("ceil({inner})"), false)))
                }
                _ => unreachable!(),
            }
        }
        "floor" => {
            validate_single_number(args)?;
            match &args[0] {
                Value::Number(n, u) => Ok(Some(Value::Number(n.floor(), u.clone()))),
                Value::Calc(c) => {
                    let inner = c
                        .strip_prefix("calc(")
                        .and_then(|s| s.strip_suffix(")"))
                        .unwrap_or(c.as_str());
                    Ok(Some(Value::String(format!("floor({inner})"), false)))
                }
                _ => unreachable!(),
            }
        }
        "round" => {
            // CSS round(): 1 arg = math.round(), 2-3 args = CSS round(strategy?, number, step)
            if args.is_empty() {
                return Err(SassError::Eval("Missing argument $number.".into()));
            }
            match args.len() {
                1 => {
                    // 传统 math.round(number)
                    validate_single_number(args)?;
                    match &args[0] {
                        Value::Number(n, u) => Ok(Some(Value::Number(n.round(), u.clone()))),
                        Value::Calc(c) => {
                            let inner = c
                                .strip_prefix("calc(")
                                .and_then(|s| s.strip_suffix(")"))
                                .unwrap_or(c.as_str());
                            Ok(Some(Value::String(format!("round({inner})"), false)))
                        }
                        _ => unreachable!(),
                    }
                }
                2 => css_round("nearest", &args[0], &args[1]),
                3 => {
                    // round(strategy, number, step)
                    let strategy = match &args[0] {
                        Value::String(s, _) => s.as_str(),
                        _ => return Err(SassError::Eval("$strategy: must be a string.".into())),
                    };
                    css_round(strategy, &args[1], &args[2])
                }
                _ => Err(SassError::Eval(format!(
                    "Only 3 arguments allowed, but {} were passed.",
                    args.len()
                ))),
            }
        }
        "mod" => {
            // CSS mod(number, step) — floored modulo
            if args.len() != 2 {
                return Err(SassError::Eval(format!(
                    "mod() expects 2 arguments, got {}.",
                    args.len()
                )));
            }
            css_mod(&args[0], &args[1])
        }
        "rem" => {
            // CSS rem(number, step) — truncated modulo
            if args.len() != 2 {
                return Err(SassError::Eval(format!(
                    "rem() expects 2 arguments, got {}.",
                    args.len()
                )));
            }
            css_rem(&args[0], &args[1])
        }
        "min" => {
            if args.is_empty() {
                return Err(SassError::Eval("min requires at least 1 argument".into()));
            }
            let result = args
                .iter()
                .try_fold(Value::Number(f64::INFINITY, None), |acc, v| {
                    match (acc, v) {
                        (Value::Number(a, ua), Value::Number(b, ub)) => {
                            // 检查单位兼容性
                            if !crate::eval::value::units_compatible(ua.as_deref(), ub.as_deref()) {
                                return Err(SassError::Eval(
                                    "min requires number arguments".into(),
                                ));
                            }
                            Ok(Value::Number(a.min(*b), ub.clone()))
                        }
                        _ => Err(SassError::Eval("min requires number arguments".into())),
                    }
                })?;
            Ok(Some(result))
        }
        "max" => {
            if args.is_empty() {
                return Err(SassError::Eval("max requires at least 1 argument".into()));
            }
            let result =
                args.iter()
                    .try_fold(Value::Number(f64::NEG_INFINITY, None), |acc, v| {
                        match (acc, v) {
                            (Value::Number(a, ua), Value::Number(b, ub)) => {
                                // 检查单位兼容性
                                if !crate::eval::value::units_compatible(
                                    ua.as_deref(),
                                    ub.as_deref(),
                                ) {
                                    return Err(SassError::Eval(
                                        "max requires number arguments".into(),
                                    ));
                                }
                                Ok(Value::Number(a.max(*b), ub.clone()))
                            }
                            _ => Err(SassError::Eval("max requires number arguments".into())),
                        }
                    })?;
            Ok(Some(result))
        }
        "percentage" => {
            validate_single_number(args)?;
            match &args[0] {
                Value::Number(n, _) => Ok(Some(Value::Number(n * 100.0, Some("%".into())))),
                _ => unreachable!(),
            }
        }
        "div" => {
            if args.is_empty() {
                return Err(SassError::Eval("Missing argument $number1.".into()));
            }
            if args.len() < 2 {
                return Err(SassError::Eval("Missing argument $number2.".into()));
            }
            if args.len() > 2 {
                return Err(SassError::Eval(format!(
                    "Only 2 arguments allowed, but {} were passed.",
                    args.len()
                )));
            }
            match (&args[0], &args[1]) {
                (Value::Number(a, u1), Value::Number(b, u2)) => {
                    if *b == 0.0 {
                        if *a == 0.0 {
                            // 0/0 = NaN，有单位时需要 calc(NaN * 1unit / 1unit) 格式
                            if u1.is_some() || u2.is_some() {
                                let mut calc = String::from("calc(NaN");
                                if let Some(u) = u1
                                    && !u.is_empty()
                                {
                                    let _ = write!(calc, " * 1{u}");
                                }
                                if let Some(u) = u2
                                    && !u.is_empty()
                                {
                                    let _ = write!(calc, " / 1{u}");
                                }
                                calc.push(')');
                                return Ok(Some(Value::Calc(calc)));
                            }
                            return Ok(Some(Value::Number(f64::NAN, None)));
                        }
                        // 有单位时需要 calc(infinity * 1unit / 1unit) 格式
                        if u1.is_some() || u2.is_some() {
                            let sign = if *a < 0.0 { "-" } else { "" };
                            let mut calc = format!("calc({sign}infinity");
                            if let Some(u) = u1
                                && !u.is_empty()
                            {
                                let _ = write!(calc, " * 1{u}");
                            }
                            if let Some(u) = u2
                                && !u.is_empty()
                            {
                                let _ = write!(calc, " / 1{u}");
                            }
                            calc.push(')');
                            return Ok(Some(Value::Calc(calc)));
                        }
                        // 无单位时返回 f64::INFINITY，display.rs 负责序列化为 calc(infinity)
                        let val = if *a < 0.0 {
                            f64::NEG_INFINITY
                        } else {
                            f64::INFINITY
                        };
                        return Ok(Some(Value::Number(val, None)));
                    }
                    Ok(Some(Value::Number(a / b, u1.clone())))
                }
                (other, Value::Number(..)) => Err(SassError::Eval(format!(
                    "$number1: {other} is not a number."
                ))),
                (Value::Number(..), other) => Err(SassError::Eval(format!(
                    "$number2: {other} is not a number."
                ))),
                (other, _) => Err(SassError::Eval(format!(
                    "$number1: {other} is not a number."
                ))),
            }
        }
        "pow" | "sqrt" | "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "atan2" | "log"
        | "hypot" => super::math_trig::call(name, args),
        "random" => {
            if args.len() > 1 {
                return Err(SassError::Eval(format!(
                    "Only 1 argument allowed, but {} {} passed.",
                    args.len(),
                    if args.len() == 1 { "was" } else { "were" }
                )));
            }
            match args {
                [] => Ok(Some(Value::Number(Evaluator::simple_random(), None))),
                [Value::Number(n, _)] => {
                    if *n <= 0.0 {
                        return Err(SassError::Eval(format!(
                            "$limit: {n} must be a positive integer."
                        )));
                    }
                    if n.fract() != 0.0 {
                        return Err(SassError::Eval(format!("$limit: {n} is not an int.")));
                    }
                    Ok(Some(Value::Number(
                        (Evaluator::simple_random() * n).floor() + 1.0,
                        None,
                    )))
                }
                [other] => Err(SassError::Eval(format!("$limit: {other} is not a number."))),
                _ => unreachable!(),
            }
        }
        "clamp" => {
            if args.is_empty() {
                return Err(SassError::Eval("Missing argument $min.".into()));
            }
            if args.len() < 2 {
                return Err(SassError::Eval("Missing argument $number.".into()));
            }
            if args.len() < 3 {
                return Err(SassError::Eval("Missing argument $max.".into()));
            }
            if args.len() > 3 {
                return Err(SassError::Eval(format!(
                    "Only 3 arguments allowed, but {} were passed.",
                    args.len()
                )));
            }
            match (&args[0], &args[1], &args[2]) {
                (Value::Number(min, _), Value::Number(val, _), Value::Number(max, _)) => {
                    Ok(Some(Value::Number(val.max(*min).min(*max), None)))
                }
                (non_num, _, _) if !matches!(non_num, Value::Number(..)) => {
                    Err(SassError::Eval(format!("$min: {non_num} is not a number.")))
                }
                (_, non_num, _) if !matches!(non_num, Value::Number(..)) => Err(SassError::Eval(
                    format!("$number: {non_num} is not a number."),
                )),
                (_, _, non_num) => {
                    Err(SassError::Eval(format!("$max: {non_num} is not a number.")))
                }
            }
        }
        "unit" => {
            validate_single_number(args)?;
            match &args[0] {
                Value::Number(_, Some(u)) => Ok(Some(Value::String(u.clone(), false))),
                Value::Number(_, None) => Ok(Some(Value::String(String::new(), false))),
                _ => unreachable!(),
            }
        }
        "is-unitless" => {
            if args.is_empty() {
                return Err(SassError::Eval("Missing argument $number.".into()));
            }
            if args.len() > 1 {
                return Err(SassError::Eval(format!(
                    "Only 1 argument allowed, but {} {} passed.",
                    args.len(),
                    if args.len() == 1 { "was" } else { "were" }
                )));
            }
            match &args[0] {
                Value::Number(_, None) => Ok(Some(Value::Bool(true))),
                Value::Number(_, Some(_)) => Ok(Some(Value::Bool(false))),
                other => Err(SassError::Eval(format!(
                    "$number: {other} is not a number."
                ))),
            }
        }
        "compatible" | "comparable" => {
            if args.is_empty() {
                return Err(SassError::Eval("Missing argument $number1.".into()));
            }
            if args.len() < 2 {
                return Err(SassError::Eval("Missing argument $number2.".into()));
            }
            if args.len() > 2 {
                return Err(SassError::Eval(format!(
                    "Only 2 arguments allowed, but {} were passed.",
                    args.len()
                )));
            }
            let u1 = match &args[0] {
                Value::Number(_, u) => u.clone(),
                other => {
                    return Err(SassError::Eval(format!(
                        "$number1: {other} is not a number."
                    )));
                }
            };
            let u2 = match &args[1] {
                Value::Number(_, u) => u.clone(),
                other => {
                    return Err(SassError::Eval(format!(
                        "$number2: {other} is not a number."
                    )));
                }
            };
            Ok(Some(Value::Bool(crate::eval::value::units_compatible(
                u1.as_deref(),
                u2.as_deref(),
            ))))
        }
        _ => Ok(None),
    }
}
