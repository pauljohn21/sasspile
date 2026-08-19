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
        | "asin" | "acos" | "atan" | "unit" | "is-unitless" | "unitless"
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
        "abs" => match args {
            [Value::Number(n, u)] => Ok(Some(Value::Number(n.abs(), u.clone()))),
            _ => Err(SassError::Eval("abs 需要 1 个数字参数".into())),
        },
        "ceil" => match args {
            [Value::Number(n, u)] => Ok(Some(Value::Number(n.ceil(), u.clone()))),
            _ => Err(SassError::Eval("ceil 需要 1 个数字参数".into())),
        },
        "floor" => match args {
            [Value::Number(n, u)] => Ok(Some(Value::Number(n.floor(), u.clone()))),
            _ => Err(SassError::Eval("floor 需要 1 个数字参数".into())),
        },
        "round" => match args {
            [Value::Number(n, u)] => Ok(Some(Value::Number(n.round(), u.clone()))),
            _ => Err(SassError::Eval("round 需要 1 个数字参数".into())),
        },
        "min" => {
            if args.is_empty() {
                return Err(SassError::Eval("min 需要至少 1 个参数".into()));
            }
            let result = args.iter().try_fold(
                Value::Number(f64::INFINITY, None),
                |acc, v| match (acc, v) {
                    (Value::Number(a, _), Value::Number(b, u)) => {
                        Ok(Value::Number(a.min(*b), u.clone()))
                    }
                    _ => Err(SassError::Eval("min 需要数字参数".into())),
                },
            )?;
            Ok(Some(result))
        }
        "max" => {
            if args.is_empty() {
                return Err(SassError::Eval("max 需要至少 1 个参数".into()));
            }
            let result = args.iter().try_fold(
                Value::Number(f64::NEG_INFINITY, None),
                |acc, v| match (acc, v) {
                    (Value::Number(a, _), Value::Number(b, u)) => {
                        Ok(Value::Number(a.max(*b), u.clone()))
                    }
                    _ => Err(SassError::Eval("max 需要数字参数".into())),
                },
            )?;
            Ok(Some(result))
        }
        "percentage" => match args {
            [Value::Number(n, _)] => Ok(Some(Value::Number(n * 100.0, Some("%".into())))),
            _ => Err(SassError::Eval("percentage 需要 1 个数字参数".into())),
        },
        "div" => match args {
            [Value::Number(a, u1), Value::Number(b, _)] => {
                if *b == 0.0 {
                    if *a == 0.0 {
                        return Ok(Some(Value::Number(f64::NAN, u1.clone())));
                    }
                    return Ok(Some(Value::Number(a / b, u1.clone())));
                }
                Ok(Some(Value::Number(a / b, u1.clone())))
            }
            _ => Err(SassError::Eval("div 需要 2 个数字参数".into())),
        },
        "pow" => match args {
            [Value::Number(a, _), Value::Number(b, _)] => {
                Ok(Some(Value::Number(a.powf(*b), None)))
            }
            _ => Err(SassError::Eval("pow 需要 2 个数字参数".into())),
        },
        "sqrt" => match args {
            [Value::Number(n, _)] => Ok(Some(Value::Number(n.sqrt(), None))),
            _ => Err(SassError::Eval("sqrt 需要 1 个数字参数".into())),
        },
        "sin" => match args {
            [Value::Number(n, _)] => Ok(Some(Value::Number(n.sin(), None))),
            _ => Err(SassError::Eval("sin 需要 1 个参数".into())),
        },
        "cos" => match args {
            [Value::Number(n, _)] => Ok(Some(Value::Number(n.cos(), None))),
            _ => Err(SassError::Eval("cos 需要 1 个参数".into())),
        },
        "tan" => match args {
            [Value::Number(n, _)] => Ok(Some(Value::Number(n.tan(), None))),
            _ => Err(SassError::Eval("tan 需要 1 个参数".into())),
        },
        "atan2" => match args {
            [Value::Number(y, _), Value::Number(x, _)] => {
                let result = y.atan2(*x).to_degrees();
                Ok(Some(Value::Number(result, Some("deg".to_string()))))
            }
            _ => Err(SassError::Eval("atan2 需要 2 个数字参数".into())),
        },
        "asin" => match args {
            [Value::Number(n, _)] => {
                let result = n.asin().to_degrees();
                Ok(Some(Value::Number(result, Some("deg".to_string()))))
            }
            _ => Err(SassError::Eval("asin 需要 1 个参数".into())),
        },
        "acos" => match args {
            [Value::Number(n, _)] => {
                let result = n.acos().to_degrees();
                Ok(Some(Value::Number(result, Some("deg".to_string()))))
            }
            _ => Err(SassError::Eval("acos 需要 1 个参数".into())),
        },
        "atan" => match args {
            [Value::Number(n, _)] => {
                let result = n.atan().to_degrees();
                Ok(Some(Value::Number(result, Some("deg".to_string()))))
            }
            _ => Err(SassError::Eval("atan 需要 1 个参数".into())),
        },
        "hypot" => {
            if args.is_empty() {
                return Err(SassError::Eval("hypot 需要 1+ 个参数".into()));
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
            _ => Err(SassError::Eval("log 需要 1-2 个数字参数".into())),
        },
        "random" => match args {
            [] => Ok(Some(Value::Number(Evaluator::simple_random(), None))),
            [Value::Number(n, _)] => Ok(Some(Value::Number(
                (Evaluator::simple_random() * n).floor() + 1.0,
                None,
            ))),
            _ => Err(SassError::Eval("random 需要 0-1 个参数".into())),
        },
        "clamp" => match args {
            [Value::Number(min, _), Value::Number(val, _), Value::Number(max, _)] => {
                Ok(Some(Value::Number(val.max(*min).min(*max), None)))
            }
            _ => Err(SassError::Eval("clamp 需要 3 个数字参数".into())),
        },
        "unit" => match args {
            [Value::Number(_, Some(u))] => Ok(Some(Value::String(u.clone(), false))),
            [Value::Number(_, None)] => Ok(Some(Value::String("".into(), false))),
            _ => Err(SassError::Eval("unit 需要 1 个数字参数".into())),
        },
        "is-unitless" => match args {
            [Value::Number(_, None)] => Ok(Some(Value::Bool(true))),
            [Value::Number(_, Some(_))] => Ok(Some(Value::Bool(false))),
            _ => Err(SassError::Eval("is-unitless 需要 1 个数字参数".into())),
        },
        "unitless" => match args {
            [Value::Number(_, None)] => Ok(Some(Value::Bool(true))),
            [Value::Number(_, Some(_))] => Ok(Some(Value::Bool(false))),
            _ => Err(SassError::Eval("unitless 需要 1 个数字参数".into())),
        },
        "compatible" => match args {
            [Value::Number(_, u1), Value::Number(_, u2)] => Ok(Some(Value::Bool(
                crate::eval::value::units_compatible(u1.as_deref(), u2.as_deref()),
            ))),
            _ => Err(SassError::Eval("compatible 需要 2 个数字参数".into())),
        },
        "comparable" => match args {
            [Value::Number(_, u1), Value::Number(_, u2)] => Ok(Some(Value::Bool(
                crate::eval::value::units_compatible(u1.as_deref(), u2.as_deref()),
            ))),
            _ => Err(SassError::Eval("comparable 需要 2 个数字参数".into())),
        },
        _ => return Ok(None),
    };
    result
}
