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
use im::HashMap;

/// Math 函数分派。返回 Ok(Some(value)) 表示已处理，Ok(None) 表示不匹配。
pub fn call(name: &str, pos_args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Option<Value>> {
    let args = merge_math_args(pos_args, kw_args, name);
    let args = args.as_slice();
    let result = match name {
        "abs" => {
            validate_single_number(args)?;
            match &args[0] {
                Value::Number(n, u) => Ok(Some(Value::Number(n.abs(), u.clone()))),
                _ => unreachable!(),
            }
        }
        "ceil" => {
            validate_single_number(args)?;
            match &args[0] {
                Value::Number(n, u) => Ok(Some(Value::Number(n.ceil(), u.clone()))),
                _ => unreachable!(),
            }
        }
        "floor" => {
            validate_single_number(args)?;
            match &args[0] {
                Value::Number(n, u) => Ok(Some(Value::Number(n.floor(), u.clone()))),
                _ => unreachable!(),
            }
        }
        "round" => {
            validate_single_number(args)?;
            match &args[0] {
                Value::Number(n, u) => Ok(Some(Value::Number(n.round(), u.clone()))),
                _ => unreachable!(),
            }
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
                                if let Some(u) = u1 {
                                    if !u.is_empty() {
                                        calc.push_str(&format!(" * 1{u}"));
                                    }
                                }
                                if let Some(u) = u2 {
                                    if !u.is_empty() {
                                        calc.push_str(&format!(" / 1{u}"));
                                    }
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
                            if let Some(u) = u1 {
                                if !u.is_empty() {
                                    calc.push_str(&format!(" * 1{u}"));
                                }
                            }
                            if let Some(u) = u2 {
                                if !u.is_empty() {
                                    calc.push_str(&format!(" / 1{u}"));
                                }
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
                _ => unreachable!(),
            }
        }
        "sin" => {
            validate_single_number(args)?;
            match &args[0] {
                Value::Number(n, _) => Ok(Some(Value::Number(n.sin(), None))),
                _ => unreachable!(),
            }
        }
        "cos" => {
            validate_single_number(args)?;
            match &args[0] {
                Value::Number(n, _) => Ok(Some(Value::Number(n.cos(), None))),
                _ => unreachable!(),
            }
        }
        "tan" => {
            validate_single_number(args)?;
            match &args[0] {
                Value::Number(n, _) => Ok(Some(Value::Number(n.tan(), None))),
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
                _ => unreachable!(),
            }
        }
        "hypot" => {
            if args.is_empty() {
                return Err(SassError::Eval("Missing argument $numbers.".into()));
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
                _ => Err(SassError::Eval("clamp requires 3 number arguments".into())),
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
        _ => return Ok(None),
    };
    result
}
