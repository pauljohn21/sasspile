//! Math 内建函数。
//!
//! 包含 abs/ceil/floor/round/min/max/percentage/div/pow/sqrt/sin/cos/tan/
//! atan2/asin/acos/atan/hypot/log/random/clamp/unit/is-unitless/compatible/comparable。
//!
//! 支持命名参数（如 `math.abs($number: 3)`、`math.clamp($min: 0, $number: 1, $max: 2)`）。
//! 辅助函数（参数名映射、合并、验证）在 `math_helpers` 模块中。

use super::super::Evaluator;
use super::math_helpers::{merge_math_args, validate_single_number};
use crate::error::{Result, SassError};
use crate::parse::ast::*;
use std::collections::HashMap;

/// Math 函数分派。返回 Ok(Some(value)) 表示已处理，Ok(None) 表示不匹配。
pub fn call(name: &str, pos_args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Option<Value>> {
    let args = merge_math_args(pos_args, kw_args, name);
    let args = args.as_slice();
    
    match name {
        "abs" => {
            validate_single_number(args)?;
            match &args[0] {
                Value::Number(n, u) => Ok(Some(Value::Number(n.abs(), u.clone()))),
                Value::Calc(c) => {
                    let inner = c.strip_prefix("calc(").and_then(|s| s.strip_suffix(")")).unwrap_or(c.as_str());
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
                    let inner = c.strip_prefix("calc(").and_then(|s| s.strip_suffix(")")).unwrap_or(c.as_str());
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
                    let inner = c.strip_prefix("calc(").and_then(|s| s.strip_suffix(")")).unwrap_or(c.as_str());
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
            if args.len() == 1 {
                // 传统 math.round(number)
                validate_single_number(args)?;
                match &args[0] {
                    Value::Number(n, u) => Ok(Some(Value::Number(n.round(), u.clone()))),
                    Value::Calc(c) => {
                        let inner = c.strip_prefix("calc(").and_then(|s| s.strip_suffix(")")).unwrap_or(c.as_str());
                        Ok(Some(Value::String(format!("round({inner})"), false)))
                    }
                    _ => unreachable!(),
                }
            } else if args.len() == 2 {
                // round(number, step) = round(nearest, number, step)
                css_round("nearest", &args[0], &args[1])
            } else if args.len() == 3 {
                // round(strategy, number, step)
                let strategy = match &args[0] {
                    Value::String(s, _) => s.as_str(),
                    _ => return Err(SassError::Eval("$strategy: must be a string.".into())),
                };
                css_round(strategy, &args[1], &args[2])
            } else {
                Err(SassError::Eval(format!(
                    "Only 3 arguments allowed, but {} were passed.",
                    args.len()
                )))
            }
        }
        "mod" => {
            // CSS mod(number, step) — floored modulo
            if args.len() != 2 {
                return Err(SassError::Eval(format!(
                    "mod() expects 2 arguments, got {}.", args.len()
                )));
            }
            css_mod(&args[0], &args[1])
        }
        "rem" => {
            // CSS rem(number, step) — truncated modulo
            if args.len() != 2 {
                return Err(SassError::Eval(format!(
                    "rem() expects 2 arguments, got {}.", args.len()
                )));
            }
            css_rem(&args[0], &args[1])
        }
        "min" => {
            if args.is_empty() {
                return Err(SassError::Eval("min requires at least 1 argument".into()));
            }
            let result = args.iter().try_fold(
                Value::Number(f64::INFINITY, None),
                |acc, v| match (acc, v) {
                    (Value::Number(a, ua), Value::Number(b, ub)) => {
                        // 检查单位兼容性
                        if !crate::eval::value::units_compatible(ua.as_deref(), ub.as_deref()) {
                            return Err(SassError::Eval("min requires number arguments".into()));
                        }
                        Ok(Value::Number(a.min(*b), ub.clone()))
                    }
                    _ => Err(SassError::Eval("min requires number arguments".into())),
                },
            )?;
            Ok(Some(result))
        }
        "max" => {
            if args.is_empty() {
                return Err(SassError::Eval("max requires at least 1 argument".into()));
            }
            let result = args.iter().try_fold(
                Value::Number(f64::NEG_INFINITY, None),
                |acc, v| match (acc, v) {
                    (Value::Number(a, ua), Value::Number(b, ub)) => {
                        // 检查单位兼容性
                        if !crate::eval::value::units_compatible(ua.as_deref(), ub.as_deref()) {
                            return Err(SassError::Eval("max requires number arguments".into()));
                        }
                        Ok(Value::Number(a.max(*b), ub.clone()))
                    }
                    _ => Err(SassError::Eval("max requires number arguments".into())),
                },
            )?;
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
                                    && !u.is_empty() {
                                        calc.push_str(&format!(" * 1{u}"));
                                    }
                                if let Some(u) = u2
                                    && !u.is_empty() {
                                        calc.push_str(&format!(" / 1{u}"));
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
                                && !u.is_empty() {
                                    calc.push_str(&format!(" * 1{u}"));
                                }
                            if let Some(u) = u2
                                && !u.is_empty() {
                                    calc.push_str(&format!(" / 1{u}"));
                                }
                            calc.push(')');
                            return Ok(Some(Value::Calc(calc)));
                        }
                        // 无单位时返回 f64::INFINITY，display.rs 负责序列化为 calc(infinity)
                        let val = if *a < 0.0 { f64::NEG_INFINITY } else { f64::INFINITY };
                        return Ok(Some(Value::Number(val, None)));
                    }
                    Ok(Some(Value::Number(a / b, u1.clone())))
                }
                (other, Value::Number(..)) => Err(SassError::Eval(format!(
                    "$number1: {} is not a number.", other
                ))),
                (Value::Number(..), other) => Err(SassError::Eval(format!(
                    "$number2: {} is not a number.", other
                ))),
                (other, _) => Err(SassError::Eval(format!(
                    "$number1: {} is not a number.", other
                ))),
            }
        },
        "pow" => {
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
            // Calc 参数透传——返回 pow(arg1, arg2) 字符串
            let a_is_calc = matches!(&args[0], Value::Calc(..));
            let b_is_calc = matches!(&args[1], Value::Calc(..));
            if a_is_calc || b_is_calc {
                let a_str = match &args[0] {
                    Value::Number(n, u) => format!("{n}{}", u.as_deref().unwrap_or("")),
                    Value::Calc(c) => c.strip_prefix("calc(").and_then(|s| s.strip_suffix(")")).unwrap_or(c.as_str()).to_string(),
                    other => return Err(SassError::Eval(format!(
                        "$base: {} is not a number.", other
                    ))),
                };
                let b_str = match &args[1] {
                    Value::Number(n, u) => format!("{n}{}", u.as_deref().unwrap_or("")),
                    Value::Calc(c) => c.strip_prefix("calc(").and_then(|s| s.strip_suffix(")")).unwrap_or(c.as_str()).to_string(),
                    other => return Err(SassError::Eval(format!(
                        "$exponent: {} is not a number.", other
                    ))),
                };
                return Ok(Some(Value::String(format!("pow({a_str}, {b_str})"), false)));
            }
            let (a, ua) = match &args[0] {
                Value::Number(n, u) => (*n, u.clone()),
                other => return Err(SassError::Eval(format!(
                    "$base: {} is not a number.", other
                ))),
            };
            if ua.is_some() {
                return Err(SassError::Eval(format!(
                    "$base: Expected {}{} to have no units.",
                    a,
                    ua.as_deref().unwrap_or("")
                )));
            }
            let (b, ub) = match &args[1] {
                Value::Number(n, u) => (*n, u.clone()),
                other => return Err(SassError::Eval(format!(
                    "$exponent: {} is not a number.", other
                ))),
            };
            if ub.is_some() {
                return Err(SassError::Eval(format!(
                    "$exponent: Expected {}{} to have no units.",
                    b,
                    ub.as_deref().unwrap_or("")
                )));
            }
            Ok(Some(Value::Number(a.powf(b), None)))
        },
        "sqrt" => {
            validate_single_number(args)?;
            match &args[0] {
                Value::Number(n, _) => Ok(Some(Value::Number(n.sqrt(), None))),
                Value::Calc(c) => {
                    let inner = c.strip_prefix("calc(").and_then(|s| s.strip_suffix(")")).unwrap_or(c.as_str());
                    Ok(Some(Value::String(format!("sqrt({inner})"), false)))
                }
                _ => unreachable!(),
            }
        }
        "sin" => {
            validate_single_number(args)?;
            match &args[0] {
                Value::Number(n, _) => Ok(Some(Value::Number(n.sin(), None))),
                Value::Calc(c) => {
                    let inner = c.strip_prefix("calc(").and_then(|s| s.strip_suffix(")")).unwrap_or(c.as_str());
                    Ok(Some(Value::String(format!("sin({inner})"), false)))
                }
                _ => unreachable!(),
            }
        }
        "cos" => {
            validate_single_number(args)?;
            match &args[0] {
                Value::Number(n, _) => Ok(Some(Value::Number(n.cos(), None))),
                Value::Calc(c) => {
                    let inner = c.strip_prefix("calc(").and_then(|s| s.strip_suffix(")")).unwrap_or(c.as_str());
                    Ok(Some(Value::String(format!("cos({inner})"), false)))
                }
                _ => unreachable!(),
            }
        }
        "tan" => {
            validate_single_number(args)?;
            match &args[0] {
                Value::Number(n, _) => Ok(Some(Value::Number(n.tan(), None))),
                Value::Calc(c) => {
                    let inner = c.strip_prefix("calc(").and_then(|s| s.strip_suffix(")")).unwrap_or(c.as_str());
                    Ok(Some(Value::String(format!("tan({inner})"), false)))
                }
                _ => unreachable!(),
            }
        }
        "atan2" => {
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
            // Calc 参数透传
            let y_is_calc = matches!(&args[0], Value::Calc(..));
            let x_is_calc = matches!(&args[1], Value::Calc(..));
            if y_is_calc || x_is_calc {
                let y_str = match &args[0] {
                    Value::Number(n, u) => format!("{n}{}", u.as_deref().unwrap_or("")),
                    Value::Calc(c) => c.strip_prefix("calc(").and_then(|s| s.strip_suffix(")")).unwrap_or(c.as_str()).to_string(),
                    other => return Err(SassError::Eval(format!(
                        "$y: {} is not a number.", other
                    ))),
                };
                let x_str = match &args[1] {
                    Value::Number(n, u) => format!("{n}{}", u.as_deref().unwrap_or("")),
                    Value::Calc(c) => c.strip_prefix("calc(").and_then(|s| s.strip_suffix(")")).unwrap_or(c.as_str()).to_string(),
                    other => return Err(SassError::Eval(format!(
                        "$x: {} is not a number.", other
                    ))),
                };
                return Ok(Some(Value::String(format!("atan2({y_str}, {x_str})"), false)));
            }
            let (y, uy) = match &args[0] {
                Value::Number(n, u) => (*n, u.clone()),
                other => return Err(SassError::Eval(format!(
                    "$y: {} is not a number.", other
                ))),
            };
            let (x, ux) = match &args[1] {
                Value::Number(n, u) => (*n, u.clone()),
                other => return Err(SassError::Eval(format!(
                    "$x: {} is not a number.", other
                ))),
            };
            // 检查单位兼容性
            if !crate::eval::value::units_compatible(uy.as_deref(), ux.as_deref()) {
                let u1_str = uy.as_deref().unwrap_or("");
                let u2_str = ux.as_deref().unwrap_or("");
                // 判断是否一个有单位一个没有
                if (uy.is_some() && ux.is_none()) || (uy.is_none() && ux.is_some()) {
                    return Err(SassError::Eval(format!(
                        "$x: {}{} and $y: {}{} have incompatible units (one has units and the other doesn't).",
                        x, u2_str, y, u1_str
                    )));
                }
                return Err(SassError::Eval(format!(
                    "$x: {}{} and $y: {}{} have incompatible units.",
                    x, u2_str, y, u1_str
                )));
            }
            let result = y.atan2(x).to_degrees();
            Ok(Some(Value::Number(result, Some("deg".to_string()))))
        },
        "asin" => {
            validate_single_number(args)?;
            match &args[0] {
                Value::Number(n, _) => {
                    let result = n.asin().to_degrees();
                    Ok(Some(Value::Number(result, Some("deg".to_string()))))
                }
                Value::Calc(c) => {
                    let inner = c.strip_prefix("calc(").and_then(|s| s.strip_suffix(")")).unwrap_or(c.as_str());
                    Ok(Some(Value::String(format!("asin({inner})"), false)))
                }
                _ => unreachable!(),
            }
        }
        "acos" => {
            validate_single_number(args)?;
            match &args[0] {
                Value::Number(n, _) => {
                    let result = n.acos().to_degrees();
                    Ok(Some(Value::Number(result, Some("deg".to_string()))))
                }
                Value::Calc(c) => {
                    let inner = c.strip_prefix("calc(").and_then(|s| s.strip_suffix(")")).unwrap_or(c.as_str());
                    Ok(Some(Value::String(format!("acos({inner})"), false)))
                }
                _ => unreachable!(),
            }
        }
        "atan" => {
            validate_single_number(args)?;
            match &args[0] {
                Value::Number(n, _) => {
                    let result = n.atan().to_degrees();
                    Ok(Some(Value::Number(result, Some("deg".to_string()))))
                }
                Value::Calc(c) => {
                    let inner = c.strip_prefix("calc(").and_then(|s| s.strip_suffix(")")).unwrap_or(c.as_str());
                    Ok(Some(Value::String(format!("atan({inner})"), false)))
                }
                _ => unreachable!(),
            }
        }
        "hypot" => {
            if args.is_empty() {
                return Err(SassError::Eval("Missing argument $numbers.".into()));
            }
            // Calc 参数透传
            let any_calc = args.iter().any(|a| matches!(a, Value::Calc(..)));
            if any_calc {
                let strs: Result<Vec<String>> = args.iter().map(|a| match a {
                    Value::Number(n, u) => Ok(format!("{n}{}", u.as_deref().unwrap_or(""))),
                    Value::Calc(c) => Ok(c.strip_prefix("calc(").and_then(|s| s.strip_suffix(")")).unwrap_or(c.as_str()).to_string()),
                    other => Err(SassError::Eval(format!("{} is not a number.", other))),
                }).collect();
                let strs = strs?;
                return Ok(Some(Value::String(format!("hypot({})", strs.join(", ")), false)));
            }
            // 验证所有参数都是数字，并收集值和单位
            let mut nums: Vec<(f64, Option<String>)> = Vec::new();
            for (i, a) in args.iter().enumerate() {
                match a {
                    Value::Number(n, u) => nums.push((*n, u.clone())),
                    other => return Err(SassError::Eval(format!(
                        "{} is not a number.", other
                    ))),
                }
                // 检查单位兼容性（从第二个参数开始与第一个比较）
                if i > 0 {
                    let u0 = nums[0].1.as_deref();
                    let ui = nums[i].1.as_deref();
                    if !crate::eval::value::units_compatible(u0, ui) {
                        return Err(SassError::Eval(format!(
                            "$numbers[{}]: {}{} and $numbers[1]: {}{} have incompatible units.",
                            i + 1, nums[i].0, ui.unwrap_or(""), nums[0].0, u0.unwrap_or("")
                        )));
                    }
                }
            }
            let sum: f64 = nums.iter().map(|(n, _)| n * n).sum();
            Ok(Some(Value::Number(sum.sqrt(), nums[0].1.clone())))
        }
        "log" => {
            if args.is_empty() {
                return Err(SassError::Eval("Missing argument $number.".into()));
            }
            if args.len() > 2 {
                return Err(SassError::Eval(format!(
                    "Only 2 arguments allowed, but {} were passed.",
                    args.len()
                )));
            }
            // Calc 参数透传
            let n_is_calc = matches!(&args[0], Value::Calc(..));
            let b_is_calc = args.len() == 2 && matches!(&args[1], Value::Calc(..));
            if n_is_calc || b_is_calc {
                let n_str = match &args[0] {
                    Value::Number(n, u) => format!("{n}{}", u.as_deref().unwrap_or("")),
                    Value::Calc(c) => c.strip_prefix("calc(").and_then(|s| s.strip_suffix(")")).unwrap_or(c.as_str()).to_string(),
                    other => return Err(SassError::Eval(format!(
                        "$number: {} is not a number.", other
                    ))),
                };
                if args.len() == 2 && !matches!(&args[1], Value::Null) {
                    let b_str = match &args[1] {
                        Value::Number(n, u) => format!("{n}{}", u.as_deref().unwrap_or("")),
                        Value::Calc(c) => c.strip_prefix("calc(").and_then(|s| s.strip_suffix(")")).unwrap_or(c.as_str()).to_string(),
                        other => return Err(SassError::Eval(format!(
                            "$base: {} is not a number.", other
                        ))),
                    };
                    return Ok(Some(Value::String(format!("log({n_str}, {b_str})"), false)));
                }
                return Ok(Some(Value::String(format!("log({n_str})"), false)));
            }
            let n = match &args[0] {
                Value::Number(n, u) => {
                    if u.is_some() {
                        return Err(SassError::Eval(format!(
                            "$number: Expected {}{} to have no units.",
                            n, u.as_deref().unwrap_or("")
                        )));
                    }
                    *n
                }
                other => return Err(SassError::Eval(format!(
                    "$number: {} is not a number.", other
                ))),
            };
            if n < 0.0 {
                return Ok(Some(Value::Calc("calc(NaN)".to_string())));
            }
            if n == 0.0 {
                return Ok(Some(Value::Calc("calc(-infinity)".to_string())));
            }
            if args.len() == 2 {
                // null base → 自然对数
                if matches!(&args[1], Value::Null) {
                    return Ok(Some(Value::Number(n.ln(), None)));
                }
                let base = match &args[1] {
                    Value::Number(b, u) => {
                        if u.is_some() {
                            return Err(SassError::Eval(format!(
                                "$base: Expected {}{} to have no units.",
                                b, u.as_deref().unwrap_or("")
                            )));
                        }
                        *b
                    }
                    other => return Err(SassError::Eval(format!(
                        "$base: {} is not a number.", other
                    ))),
                };
                Ok(Some(Value::Number(n.log(base), None)))
            } else {
                Ok(Some(Value::Number(n.ln(), None)))
            }
        },
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
                            "$limit: {} must be a positive integer.", n
                        )));
                    }
                    if n.fract() != 0.0 {
                        return Err(SassError::Eval(format!(
                            "$limit: {} is not an int.", n
                        )));
                    }
                    Ok(Some(Value::Number(
                        (Evaluator::simple_random() * n).floor() + 1.0,
                        None,
                    )))
                }
                [other] => Err(SassError::Eval(format!(
                    "$limit: {} is not a number.", other
                ))),
                _ => unreachable!(),
            }
        },
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
                (_, non_num, _) if !matches!(non_num, Value::Number(..)) => {
                    Err(SassError::Eval(format!("$number: {non_num} is not a number.")))
                }
                (_, _, non_num) => {
                    Err(SassError::Eval(format!("$max: {non_num} is not a number.")))
                }
            }
        },
        "unit" => {
            validate_single_number(args)?;
            match &args[0] {
                Value::Number(_, Some(u)) => Ok(Some(Value::String(u.clone(), false))),
                Value::Number(_, None) => Ok(Some(Value::String("".into(), false))),
                _ => unreachable!(),
            }
        },
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
                    "$number: {} is not a number.", other
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
                other => return Err(SassError::Eval(format!(
                    "$number1: {} is not a number.", other
                ))),
            };
            let u2 = match &args[1] {
                Value::Number(_, u) => u.clone(),
                other => return Err(SassError::Eval(format!(
                    "$number2: {} is not a number.", other
                ))),
            };
            Ok(Some(Value::Bool(
                crate::eval::value::units_compatible(u1.as_deref(), u2.as_deref()),
            )))
        }
        _ => Ok(None),
    }
}

/// CSS round(strategy, number, step) 函数。
///
/// 根据 strategy 将 number 舍入到 step 的倍数：
/// - nearest: 最接近的倍数（默认）
/// - up: 向上舍入
/// - down: 向下舍入
/// - to-zero: 向零舍入
#[allow(clippy::pedantic)]
fn css_round(strategy: &str, number: &Value, step: &Value) -> Result<Option<Value>> {
    let (n, n_unit) = match number {
        Value::Number(n, u) => (*n, u.clone()),
        _ => return Err(SassError::Eval(format!("$number: {number} is not a number."))),
    };
    let (s, s_unit) = match step {
        Value::Number(s, u) => (*s, u.clone()),
        _ => return Err(SassError::Eval(format!("$step: {step} is not a number."))),
    };
    if s == 0.0 {
        return Err(SassError::Eval("Round step cannot be zero.".into()));
    }
    // 单位兼容性检查
    let compatible = crate::eval::value::units_compatible(n_unit.as_deref(), s_unit.as_deref());
    if !compatible {
        // 不兼容单位：保留 round() 输出
        let n_str = match &n_unit {
            Some(u) => format!("{n}{u}"),
            None => n.to_string(),
        };
        let s_str = match &s_unit {
            Some(u) => format!("{s}{u}"),
            None => s.to_string(),
        };
        return Ok(Some(Value::String(format!("round({strategy}, {n_str}, {s_str})"), false)));
    }
    // 单位转换：将 step 转为 number 的单位
    let (s_converted, out_unit) = match (&n_unit, &s_unit) {
        (None, None) => (s, None),
        (Some(u), None) => (s, Some(u.clone())),
        (None, Some(u)) => (s, Some(u.clone())),
        (Some(nu), Some(su)) if nu == su => (s, Some(nu.clone())),
        (Some(nu), Some(su)) => {
            let factor = unit_conversion_factor(su, nu);
            (s * factor, Some(nu.clone()))
        }
    };
    let ratio = n / s_converted;
    let rounded = match strategy {
        "nearest" => ratio.round(),
        "up" => ratio.ceil(),
        "down" => ratio.floor(),
        "to-zero" => ratio.trunc(),
        _ => return Err(SassError::Eval(format!("Unknown strategy: {strategy}"))),
    };
    let result = rounded * s_converted;
    Ok(Some(Value::Number(result, out_unit)))
}

/// 获取从 from_unit 到 to_unit 的转换因子。
/// 仅支持兼容单位（长度、角度、时间、频率、分辨率）。
fn unit_conversion_factor(from: &str, to: &str) -> f64 {
    if from == to {
        return 1.0;
    }
    // 长度单位到 px 的转换因子
    const LENGTH_TO_PX: &[(&str, f64)] = &[
        ("px", 1.0), ("in", 96.0), ("cm", 96.0 / 2.54), ("mm", 96.0 / 25.4),
        ("pt", 96.0 / 72.0), ("pc", 96.0 / 6.0), ("q", 96.0 / 254.0),
    ];
    // 角度单位到 deg 的转换因子
    const ANGLE_TO_DEG: &[(&str, f64)] = &[
        ("deg", 1.0), ("grad", 0.9), ("rad", 180.0 / std::f64::consts::PI), ("turn", 360.0),
    ];
    // 时间单位到 s 的转换因子
    const TIME_TO_S: &[(&str, f64)] = &[("s", 1.0), ("ms", 0.001)];
    // 频率单位到 Hz 的转换因子
    const FREQ_TO_HZ: &[(&str, f64)] = &[("hz", 1.0), ("khz", 1000.0)];
    // 分辨率单位到 dpi 的转换因子
    const RES_TO_DPI: &[(&str, f64)] = &[
        ("dpi", 1.0), ("dpcm", 2.54), ("dppx", 96.0),
    ];
    for table in [LENGTH_TO_PX, ANGLE_TO_DEG, TIME_TO_S, FREQ_TO_HZ, RES_TO_DPI] {
        let from_f = table.iter().find(|(u, _)| *u == from).map(|(_, f)| *f);
        let to_f = table.iter().find(|(u, _)| *u == to).map(|(_, f)| *f);
        if let (Some(f), Some(t)) = (from_f, to_f) {
            return f / t;
        }
    }
    1.0 // 不兼容——不转换
}

/// CSS mod(number, step) — floored modulo。
/// 结果符号跟随 step 的符号（Sass `%` 语义一致）。
#[allow(clippy::pedantic)]
fn css_mod(number: &Value, step: &Value) -> Result<Option<Value>> {
    let (n, n_unit) = match number {
        Value::Number(n, u) => (*n, u.clone()),
        _ => return Err(SassError::Eval(format!("$number: {number} is not a number."))),
    };
    let (s, s_unit) = match step {
        Value::Number(s, u) => (*s, u.clone()),
        _ => return Err(SassError::Eval(format!("$step: {step} is not a number."))),
    };
    if s == 0.0 {
        return Err(SassError::Eval("mod() step cannot be zero.".into()));
    }
    let compatible = crate::eval::value::units_compatible(n_unit.as_deref(), s_unit.as_deref());
    if !compatible {
        let n_str = match &n_unit { Some(u) => format!("{n}{u}"), None => n.to_string() };
        let s_str = match &s_unit { Some(u) => format!("{s}{u}"), None => s.to_string() };
        return Ok(Some(Value::String(format!("mod({n_str}, {s_str})"), false)));
    }
    let (s_converted, out_unit) = match (&n_unit, &s_unit) {
        (None, None) => (s, None),
        (Some(u), None) => (s, Some(u.clone())),
        (None, Some(u)) => (s, Some(u.clone())),
        (Some(nu), Some(su)) if nu == su => (s, Some(nu.clone())),
        (Some(nu), Some(su)) => (s * unit_conversion_factor(su, nu), Some(nu.clone())),
    };
    // floored modulo: n - s * floor(n / s)
    let result = n - s_converted * (n / s_converted).floor();
    Ok(Some(Value::Number(result, out_unit)))
}

/// CSS rem(number, step) — truncated modulo。
/// 结果符号跟随 number 的符号。
#[allow(clippy::pedantic)]
fn css_rem(number: &Value, step: &Value) -> Result<Option<Value>> {
    let (n, n_unit) = match number {
        Value::Number(n, u) => (*n, u.clone()),
        _ => return Err(SassError::Eval(format!("$number: {number} is not a number."))),
    };
    let (s, s_unit) = match step {
        Value::Number(s, u) => (*s, u.clone()),
        _ => return Err(SassError::Eval(format!("$step: {step} is not a number."))),
    };
    if s == 0.0 {
        return Err(SassError::Eval("rem() step cannot be zero.".into()));
    }
    let compatible = crate::eval::value::units_compatible(n_unit.as_deref(), s_unit.as_deref());
    if !compatible {
        let n_str = match &n_unit { Some(u) => format!("{n}{u}"), None => n.to_string() };
        let s_str = match &s_unit { Some(u) => format!("{s}{u}"), None => s.to_string() };
        return Ok(Some(Value::String(format!("rem({n_str}, {s_str})"), false)));
    }
    let (s_converted, out_unit) = match (&n_unit, &s_unit) {
        (None, None) => (s, None),
        (Some(u), None) => (s, Some(u.clone())),
        (None, Some(u)) => (s, Some(u.clone())),
        (Some(nu), Some(su)) if nu == su => (s, Some(nu.clone())),
        (Some(nu), Some(su)) => (s * unit_conversion_factor(su, nu), Some(nu.clone())),
    };
    // truncated modulo: n - s * trunc(n / s)
    let result = n - s_converted * (n / s_converted).trunc();
    Ok(Some(Value::Number(result, out_unit)))
}
