//! Math 内建函数。
//!
//! 包含 abs/ceil/floor/round/min/max/percentage/div/pow/sqrt/sin/cos/tan/
//! atan2/asin/acos/atan/hypot/log/random/clamp/unit/is-unitless/compatible/comparable。
//!
//! 支持命名参数（如 `math.abs($number: 3)`、`math.clamp($min: 0, $number: 1, $max: 2)`）。

use super::super::Evaluator;
use crate::error::{Result, SassError};
use crate::parse::ast::*;
use im::HashMap;

/// 返回每个 math 函数的参数名列表（按位置顺序）。
/// 用于将命名参数（kw_args）按参数名映射到位置参数。
fn math_param_names(name: &str) -> &'static [&'static str] {
    match name {
        "abs" | "ceil" | "floor" | "round" | "sqrt" | "sin" | "cos" | "tan"
        | "asin" | "acos" | "atan" | "unit" | "is-unitless"
        | "percentage" => &["number"],
        "div" => &["number1", "number2"],
        "pow" => &["base", "exponent"],
        "atan2" => &["number", "exponent"],
        "log" => &["number", "base"],
        "clamp" => &["min", "number", "max"],
        "compatible" | "comparable" => &["number1", "number2"],
        "random" => &["limit"],
        // variadic：直接返回 pos_args
        "hypot" | "min" | "max" => &[],
        _ => &[],
    }
}

/// 将位置参数和命名参数合并为统一的位置参数列表。
/// 按 `param_names` 顺序填充：先取 pos_args 对应位置，不足的从 kw_args 按参数名查找。
pub(crate) fn merge_math_args(
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
    name: &str,
) -> Vec<Value> {
    let param_names = math_param_names(name);
    if param_names.is_empty() {
        return pos_args.to_vec();
    }
    let mut result = Vec::with_capacity(param_names.len().max(pos_args.len()));
    for (i, pname) in param_names.iter().enumerate() {
        if i < pos_args.len() {
            result.push(pos_args[i].clone());
        } else if let Some(v) = kw_args.get(*pname) {
            result.push(v.clone());
        } else if let Some(v) = kw_args.get(&format!("${pname}")) {
            result.push(v.clone());
        }
    }
    // 追加多余的 pos_args（如 rest 参数场景）
    if pos_args.len() > param_names.len() {
        result.extend_from_slice(&pos_args[param_names.len()..]);
    }
    result
}

/// Math 函数分派。返回 Ok(Some(value)) 表示已处理，Ok(None) 表示不匹配。
pub fn call(name: &str, pos_args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Option<Value>> {
    let args = merge_math_args(pos_args, kw_args, name);
    let args = args.as_slice();
    let result = match name {
        "abs" => {
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
                Value::Number(n, u) => Ok(Some(Value::Number(n.abs(), u.clone()))),
                other => Err(SassError::Eval(format!(
                    "$number: {} is not a number.", other
                ))),
            }
        }
        "ceil" => {
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
                Value::Number(n, u) => Ok(Some(Value::Number(n.ceil(), u.clone()))),
                other => Err(SassError::Eval(format!(
                    "$number: {} is not a number.", other
                ))),
            }
        }
        "floor" => {
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
                Value::Number(n, u) => Ok(Some(Value::Number(n.floor(), u.clone()))),
                other => Err(SassError::Eval(format!(
                    "$number: {} is not a number.", other
                ))),
            }
        }
        "round" => {
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
                Value::Number(n, u) => Ok(Some(Value::Number(n.round(), u.clone()))),
                other => Err(SassError::Eval(format!(
                    "$number: {} is not a number.", other
                ))),
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
                Value::Number(n, _) => Ok(Some(Value::Number(n * 100.0, Some("%".into())))),
                other => Err(SassError::Eval(format!(
                    "$number: {} is not a number.", other
                ))),
            }
        }
        "div" => match args {
            [Value::Number(a, u1), Value::Number(b, u2)] => {
                if *b == 0.0 {
                    if *a == 0.0 {
                        return Ok(Some(Value::Number(f64::NAN, u1.clone())));
                    }
                    // 除零产生 infinity——需要构建 calc(infinity) 表达式
                    // 包含分子和分母单位
                    let sign = if *a < 0.0 { "-" } else { "" };
                    let mut calc = format!("calc({sign}infinity");
                    // 分子单位
                    if let Some(u) = u1 {
                        if !u.is_empty() {
                            calc.push_str(&format!(" * 1{u}"));
                        }
                    }
                    // 分母单位
                    if let Some(u) = u2 {
                        if !u.is_empty() {
                            calc.push_str(&format!(" / 1{u}"));
                        }
                    }
                    calc.push(')');
                    return Ok(Some(Value::Calc(calc)));
                }
                Ok(Some(Value::Number(a / b, u1.clone())))
            }
            _ => Err(SassError::Eval("div requires 2 number arguments".into())),
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
        "sqrt" => match args {
            [Value::Number(n, _)] => Ok(Some(Value::Number(n.sqrt(), None))),
            _ => Err(SassError::Eval("sqrt requires 1 number argument".into())),
        },
        "sin" => match args {
            [Value::Number(n, _)] => Ok(Some(Value::Number(n.sin(), None))),
            _ => Err(SassError::Eval("sin requires 1 argument".into())),
        },
        "cos" => match args {
            [Value::Number(n, _)] => Ok(Some(Value::Number(n.cos(), None))),
            _ => Err(SassError::Eval("cos requires 1 argument".into())),
        },
        "tan" => match args {
            [Value::Number(n, _)] => Ok(Some(Value::Number(n.tan(), None))),
            _ => Err(SassError::Eval("tan requires 1 argument".into())),
        },
        "atan2" => match args {
            [Value::Number(y, _), Value::Number(x, _)] => {
                let result = y.atan2(*x).to_degrees();
                Ok(Some(Value::Number(result, Some("deg".to_string()))))
            }
            _ => Err(SassError::Eval("atan2 requires 2 number arguments".into())),
        },
        "asin" => match args {
            [Value::Number(n, _)] => {
                let result = n.asin().to_degrees();
                Ok(Some(Value::Number(result, Some("deg".to_string()))))
            }
            _ => Err(SassError::Eval("asin requires 1 argument".into())),
        },
        "acos" => match args {
            [Value::Number(n, _)] => {
                let result = n.acos().to_degrees();
                Ok(Some(Value::Number(result, Some("deg".to_string()))))
            }
            _ => Err(SassError::Eval("acos requires 1 argument".into())),
        },
        "atan" => match args {
            [Value::Number(n, _)] => {
                let result = n.atan().to_degrees();
                Ok(Some(Value::Number(result, Some("deg".to_string()))))
            }
            _ => Err(SassError::Eval("atan requires 1 argument".into())),
        },
        "hypot" => {
            if args.is_empty() {
                return Err(SassError::Eval("hypot requires 1+ arguments".into()));
            }
            let sum: f64 = args
                .iter()
                .map(|a| match a {
                    Value::Number(n, _) => n * n,
                    _ => 0.0,
                })
                .sum();
            Ok(Some(Value::Number(sum.sqrt(), None)))
        }
        "log" => match args {
            [Value::Number(n, _)] => {
                if *n < 0.0 {
                    return Ok(Some(Value::String("calc(NaN)".to_string(), false)));
                }
                if *n == 0.0 {
                    return Ok(Some(Value::String("calc(-infinity)".to_string(), false)));
                }
                Ok(Some(Value::Number(n.ln(), None)))
            }
            [Value::Number(n, _), Value::Number(base, _)] => {
                if *n < 0.0 {
                    return Ok(Some(Value::String("calc(NaN)".to_string(), false)));
                }
                if *n == 0.0 {
                    return Ok(Some(Value::String("calc(-infinity)".to_string(), false)));
                }
                Ok(Some(Value::Number(n.log(*base), None)))
            }
            _ => Err(SassError::Eval("log requires 1-2 number arguments".into())),
        },
        "random" => match args {
            [] => Ok(Some(Value::Number(Evaluator::simple_random(), None))),
            [Value::Number(n, _)] => Ok(Some(Value::Number(
                (Evaluator::simple_random() * n).floor() + 1.0,
                None,
            ))),
            _ => Err(SassError::Eval("random requires 0-1 arguments".into())),
        },
        "clamp" => match args {
            [Value::Number(min, _), Value::Number(val, _), Value::Number(max, _)] => {
                Ok(Some(Value::Number(val.max(*min).min(*max), None)))
            }
            _ => Err(SassError::Eval("clamp requires 3 number arguments".into())),
        },
        "unit" => match args {
            [Value::Number(_, Some(u))] => Ok(Some(Value::String(u.clone(), false))),
            [Value::Number(_, None)] => Ok(Some(Value::String("".into(), false))),
            _ => Err(SassError::Eval("unit requires 1 number argument".into())),
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
