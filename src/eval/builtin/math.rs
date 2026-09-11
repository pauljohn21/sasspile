//! Math 内建函数。
//!
//! 包含 abs/ceil/floor/round/min/max/percentage/div/pow/sqrt/sin/cos/tan/
//! atan2/asin/acos/atan/hypot/log/random/clamp/unit/is-unitless/compatible/comparable。
//! CSS round/mod/rem 函数：css_round（1-3 `参数+策略+单位转换）、css_mod（floored）、css_rem（truncated`）。
//!
//! 支持命名参数（如 `math.abs($number: 3)`、`math.clamp($min: 0, $number: 1, $max: 2)`）。
//! 辅助函数（参数名映射、合并、验证）在 `math_helpers` 模块中。

use super::super::Evaluator;
use super::math_css::{css_mod, css_rem};
use super::math_helpers::{merge_math_args, validate_single_number};
use crate::error::{Result, SassError};
use crate::parse::ast::*;
use imbl::HashMap;
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
                _ => Err(SassError::Eval("$number is not a number.".into())),
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
                _ => Err(SassError::Eval("$number is not a number.".into())),
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
                _ => Err(SassError::Eval("$number is not a number.".into())),
            }
        }
        "round" => {
            // math.round() 仅接受 1 个参数（传统 Sass 行为）
            validate_single_number(args)?;
            match args.len() {
                1 => {}
                n => {
                    return Err(SassError::Eval(format!(
                        "Only 1 argument allowed, but {n} {} passed.",
                        match n == 1 { true => "was", false => "were" }
                    )))
                }
            }
            match &args[0] {
                Value::Number(n, u) => Ok(Some(Value::Number(n.round(), u.clone()))),
                Value::Calc(c) => {
                    let inner = c
                        .strip_prefix("calc(")
                        .and_then(|s| s.strip_suffix(")"))
                        .unwrap_or(c.as_str());
                    Ok(Some(Value::String(format!("round({inner})"), false)))
                }
                _ => Err(SassError::Eval("$number is not a number.".into())),
            }
        }
        "mod" => {
            // CSS mod(number, step) — floored modulo
            match args.len() {
                2 => css_mod(&args[0], &args[1]),
                n => Err(SassError::Eval(format!(
                    "mod() expects 2 arguments, got {n}."
                ))),
            }
        }
        "rem" => {
            // CSS rem(number, step) — truncated modulo
            match args.len() {
                2 => css_rem(&args[0], &args[1]),
                n => Err(SassError::Eval(format!(
                    "rem() expects 2 arguments, got {n}."
                ))),
            }
        }
        "min" | "max" => {
            let is_min = name == "min";
            match args.is_empty() {
                true => {
                    return Err(SassError::Eval(format!(
                        "{name} requires at least 1 argument"
                    )))
                }
                false => {}
            }
            // 使用第一个参数的单位作为目标单位，将所有参数转换后比较
            let first_unit = match &args[0] {
                Value::Number(_, u) => u.clone(),
                _ => return Err(SassError::Eval(format!("{name} requires number arguments"))),
            };
            let mut result = match &args[0] {
                Value::Number(n, _) => *n,
                _ => unreachable!(),
            };
            let result_unit = first_unit.clone();
            for arg in args.iter().skip(1) {
                let (val, unit) = match arg {
                    Value::Number(n, u) => (*n, u.clone()),
                    _ => {
                        return Err(SassError::Eval(format!(
                            "{name} requires number arguments"
                        )))
                    }
                };
                // 检查单位兼容性
                if !crate::eval::value::units_compatible(first_unit.as_deref(), unit.as_deref()) {
                    return Err(SassError::Eval(format!(
                        "{name} requires number arguments"
                    )));
                }
                // 转换到目标单位
                let converted = match (first_unit.as_deref(), unit.as_deref()) {
                    (None, None) => val,
                    (Some(fu), Some(u)) => {
                        val * crate::eval::builtin::math_css::unit_conversion_factor(u, fu)
                    }
                    _ => val, // unitless + unit 保持原值
                };
                result = match is_min {
                    true => result.min(converted),
                    false => result.max(converted),
                };
            }
            Ok(Some(Value::Number(result, result_unit)))
        }
        "percentage" => {
            validate_single_number(args)?;
            match &args[0] {
                Value::Number(n, None) => Ok(Some(Value::Number(n * 100.0, Some("%".into())))),
                Value::Number(n, Some(u)) => Err(SassError::Eval(format!(
                    "$number: Expected {n} to have no units but it has {u}."
                ))),
                _ => Err(SassError::Eval("$number is not a number.".into())),
            }
        }
        "div" => {
            match args.len() {
                0 => return Err(SassError::Eval("Missing argument $number1.".into())),
                1 => return Err(SassError::Eval("Missing argument $number2.".into())),
                2 => {}
                n => return Err(SassError::Eval(format!(
                    "Only 2 arguments allowed, but {n} were passed."
                ))),
            }
            // 非数字参数返回字符串形式（如 6/b）
            let a_is_num = matches!(&args[0], Value::Number(..));
            let b_is_num = matches!(&args[1], Value::Number(..));
            if !a_is_num || !b_is_num {
                let a_str = match &args[0] {
                    Value::Number(n, u) => format!("{n}{}", u.as_deref().unwrap_or("")),
                    other => other.to_string(),
                };
                let b_str = match &args[1] {
                    Value::Number(n, u) => format!("{n}{}", u.as_deref().unwrap_or("")),
                    other => other.to_string(),
                };
                return Ok(Some(Value::String(format!("{a_str}/{b_str}"), false)));
            }
            let (a, u1) = match &args[0] {
                Value::Number(n, u) => (*n, u.clone()),
                _ => unreachable!(),
            };
            let (b, u2) = match &args[1] {
                Value::Number(n, u) => (*n, u.clone()),
                _ => unreachable!(),
            };
            // div by zero handling
            match (b == 0.0, a == 0.0) {
                (true, true) => {
                    // 0/0 = NaN
                    match u1.is_some() || u2.is_some() {
                        true => {
                            let mut calc = String::from("calc(NaN");
                            if let Some(u) = u1.as_ref() {
                                if !u.is_empty() {
                                    let _ = write!(calc, " * 1{u}");
                                }
                            }
                            if let Some(u) = u2.as_ref() {
                                if !u.is_empty() {
                                    let _ = write!(calc, " / 1{u}");
                                }
                            }
                            calc.push(')');
                            Ok(Some(Value::Calc(calc)))
                        }
                        false => Ok(Some(Value::Number(f64::NAN, None))),
                    }
                }
                (true, false) => {
                    // n/0 = infinity
                    match u1.is_some() || u2.is_some() {
                        true => {
                            let sign = match a < 0.0 { true => "-", false => "" };
                            let mut calc = format!("calc({sign}infinity");
                            if let Some(u) = u1.as_ref() {
                                if !u.is_empty() {
                                    let _ = write!(calc, " * 1{u}");
                                }
                            }
                            if let Some(u) = u2.as_ref() {
                                if !u.is_empty() {
                                    let _ = write!(calc, " / 1{u}");
                                }
                            }
                            calc.push(')');
                            Ok(Some(Value::Calc(calc)))
                        }
                        false => {
                            let val = match a < 0.0 {
                                true => f64::NEG_INFINITY,
                                false => f64::INFINITY,
                            };
                            Ok(Some(Value::Number(val, None)))
                        }
                    }
                }
                (false, _) => {
                    // 单位处理：相同单位 → unitless；不同兼容单位 → 转换
                    match (u1.as_deref(), u2.as_deref()) {
                        (Some(u1_str), Some(u2_str)) if u1_str == u2_str => {
                            // 相同单位，结果 unitless
                            Ok(Some(Value::Number(a / b, None)))
                        }
                        (Some(u1_str), Some(u2_str)) => {
                            // 不同兼容单位，转换
                            let factor = crate::eval::builtin::math_css::unit_conversion_factor(u2_str, u1_str);
                            Ok(Some(Value::Number(a / (b * factor), Some(u1_str.to_string()))))
                        }
                        _ => Ok(Some(Value::Number(a / b, u1.clone()))),
                    }
                }
            }
        }
        "pow" | "sqrt" | "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "atan2" | "log"
        | "hypot" => super::math_trig::call(name, args),
        "random" => {
            match args.len() > 1 {
                true => return Err(SassError::Eval(format!(
                    "Only 1 argument allowed, but {} {} passed.",
                    args.len(),
                    match args.len() == 1 { true => "was", false => "were" }
                ))),
                false => {}
            }
            match args {
                [] => Ok(Some(Value::Number(Evaluator::simple_random(), None))),
                [Value::Number(n, _)] => {
                    match *n <= 0.0 {
                        true => return Err(SassError::Eval(format!(
                            "$limit: {n} must be a positive integer."
                        ))),
                        false => {}
                    }
                    match n.fract() != 0.0 {
                        true => return Err(SassError::Eval(format!("$limit: {n} is not an int."))),
                        false => {}
                    }
                    Ok(Some(Value::Number(
                        (Evaluator::simple_random() * n).floor() + 1.0,
                        None,
                    )))
                }
                [other] => Err(SassError::Eval(format!("$limit: {other} is not a number."))),
                _ => Err(SassError::Eval("$number is not a number.".into())),
            }
        }
        "clamp" => {
            match args.len() {
                0 => return Err(SassError::Eval("Missing argument $min.".into())),
                1 => return Err(SassError::Eval("Missing argument $number.".into())),
                2 => return Err(SassError::Eval("Missing argument $max.".into())),
                3 => {}
                n => return Err(SassError::Eval(format!(
                    "Only 3 arguments allowed, but {n} were passed."
                ))),
            }
            match (&args[0], &args[1], &args[2]) {
                (Value::Number(min, u_min), Value::Number(val, u_val), Value::Number(max, u_max)) => {
                    // 检查：所有参数必须同为 unitless 或同为有单位（不可混用）
                    let min_has_unit = u_min.is_some();
                    let val_has_unit = u_val.is_some();
                    let max_has_unit = u_max.is_some();
                    if min_has_unit != val_has_unit || val_has_unit != max_has_unit {
                        return Err(SassError::Eval("Incompatible units.".to_string()));
                    }
                    // 校验所有参数的单位兼容
                    if !crate::eval::value::units_compatible(u_min.as_deref(), u_val.as_deref()) {
                        return Err(SassError::Eval("Incompatible units.".to_string()));
                    }
                    if !crate::eval::value::units_compatible(u_val.as_deref(), u_max.as_deref()) {
                        return Err(SassError::Eval("Incompatible units.".to_string()));
                    }
                    // clamp: 先限制下限，再限制上限
                    Ok(Some(Value::Number(val.max(*min).min(*max), u_val.clone())))
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
                // 返回带引号的字符串，如 "px"、"px*em" 等
                Value::Number(_, Some(u)) => Ok(Some(Value::String(u.clone(), true))),
                Value::Number(_, None) => Ok(Some(Value::String(String::new(), true))),
                Value::Calc(_) => Err(SassError::Eval(
                    "$number: calc expressions can't be used to determine units.".into(),
                )),
                _ => Err(SassError::Eval("$number is not a number.".into())),
            }
        }
        "is-unitless" => {
            match args.len() {
                0 => return Err(SassError::Eval("Missing argument $number.".into())),
                1 => {}
                n => return Err(SassError::Eval(format!(
                    "Only 1 argument allowed, but {n} {} passed.",
                    match n == 1 { true => "was", false => "were" }
                ))),
            }
            match &args[0] {
                Value::Number(_, None) => Ok(Some(Value::Bool(true))),
                Value::Number(_, Some(_)) => Ok(Some(Value::Bool(false))),
                other => Err(SassError::Eval(format!(
                    "$number: {other} is not a number."
                ))),
            }
        }
        "compatible" => {
            match args.len() {
                0 => return Err(SassError::Eval("Missing argument $number1.".into())),
                1 => return Err(SassError::Eval("Missing argument $number2.".into())),
                2 => {}
                n => return Err(SassError::Eval(format!(
                    "Only 2 arguments allowed, but {n} were passed."
                ))),
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
