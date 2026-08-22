//! math 内建函数。

use crate::error::{Result, SassError};
use crate::eval::value::Value;
use crate::eval::env::Env;
use crate::parse::ast::Arg;
use crate::eval::eval_value;

pub fn dispatch(field: &str, args: &[Arg], env: &Env) -> Result<Value> {
    let args: Vec<Value> = args.iter().map(|a| eval_value(&a.value, env)).collect();
    match field {
        "abs" => match &args[..] {
            [Value::Number(n, u)] => Ok(Value::Number(n.abs(), u.clone())),
            _ => Err(SassError::eval("abs() expects a number")),
        },
        "ceil" => match &args[..] {
            [Value::Number(n, u)] => Ok(Value::Number(n.ceil(), u.clone())),
            _ => Err(SassError::eval("ceil() expects a number")),
        },
        "floor" => match &args[..] {
            [Value::Number(n, u)] => Ok(Value::Number(n.floor(), u.clone())),
            _ => Err(SassError::eval("floor() expects a number")),
        },
        "round" => match &args[..] {
            [Value::Number(n, u)] => Ok(Value::Number(n.round(), u.clone())),
            _ => Err(SassError::eval("round() expects a number")),
        },
        "max" => {
            let nums: Vec<(f64, Option<String>)> = args.iter().filter_map(|v| match v {
                Value::Number(n, u) => Some((*n, u.clone())),
                _ => None,
            }).collect();
            if nums.is_empty() {
                return Err(SassError::eval("max() expects numbers"));
            }
            let mut result = nums[0].clone();
            for (n, u) in &nums[1..] {
                if *n > result.0 {
                    result = (*n, u.clone());
                }
            }
            Ok(Value::Number(result.0, result.1))
        },
        "min" => {
            let nums: Vec<(f64, Option<String>)> = args.iter().filter_map(|v| match v {
                Value::Number(n, u) => Some((*n, u.clone())),
                _ => None,
            }).collect();
            if nums.is_empty() {
                return Err(SassError::eval("min() expects numbers"));
            }
            let mut result = nums[0].clone();
            for (n, u) in &nums[1..] {
                if *n < result.0 {
                    result = (*n, u.clone());
                }
            }
            Ok(Value::Number(result.0, result.1))
        },
        "is_unitless" => match &args[..] {
            [Value::Number(_, unit)] => Ok(Value::Bool(unit.is_none())),
            _ => Err(SassError::eval("is-unitless() expects a number")),
        },
        "percentage" => match &args[..] {
            [Value::Number(n, _)] => Ok(Value::Number(n * 100.0, Some("%".to_string()))),
            _ => Err(SassError::eval("percentage() expects a number")),
        },
        "unit" => match &args[..] {
            [Value::Number(_, Some(u))] => Ok(Value::String(u.clone(), crate::lex::token::QuoteStyle::None)),
            [Value::Number(_, None)] => Ok(Value::String(String::new(), crate::lex::token::QuoteStyle::None)),
            _ => Err(SassError::eval("unit() expects a number")),
        },
        "random" => match &args[..] {
            [] => Ok(Value::Number(0.0, None)),  // placeholder
            [Value::Number(n, _)] if *n > 0.0 => Ok(Value::Number(1.0, None)),  // placeholder
            _ => Err(SassError::eval("random() expects optional positive number")),
        },
        "div" => match &args[..] {
            [Value::Number(a, ua), Value::Number(b, ub)] => {
                if *b == 0.0 {
                    return Err(SassError::eval("Cannot divide by zero"));
                }
                let unit = ua.clone().or(ub.clone());
                Ok(Value::Number(a / b, unit))
            }
            _ => Err(SassError::eval("math.div() expects two numbers")),
        },
        "clamp" => match &args[..] {
            [Value::Number(min, _), Value::Number(n, u), Value::Number(max, _)] => {
                if min > max {
                    Ok(Value::Number(*min, u.clone()))
                } else {
                    Ok(Value::Number(n.clamp(*min, *max), u.clone()))
                }
            }
            _ => Err(SassError::eval("clamp() expects three numbers")),
        },
        "hypot" => {
            let nums: Vec<f64> = args.iter().filter_map(|v| match v {
                Value::Number(n, _) => Some(*n),
                _ => None,
            }).collect();
            if nums.is_empty() {
                return Err(SassError::eval("hypot() expects numbers"));
            }
            let sum: f64 = nums.iter().map(|n| n * n).sum();
            Ok(Value::Number(sum.sqrt(), None))
        },
        "sqrt" => match &args[..] {
            [Value::Number(n, _)] => Ok(Value::Number(n.sqrt(), None)),
            _ => Err(SassError::eval("sqrt() expects a number")),
        },
        "pow" => match &args[..] {
            [Value::Number(base, _), Value::Number(exp, _)] => {
                Ok(Value::Number(base.powf(*exp), None))
            }
            _ => Err(SassError::eval("pow() expects two numbers")),
        },
        "log" => match &args[..] {
            [Value::Number(n, _)] => Ok(Value::Number(n.ln(), None)),
            [Value::Number(n, _), Value::Number(base, _)] => {
                Ok(Value::Number(n.log(*base), None))
            }
            _ => Err(SassError::eval("log() expects a number")),
        },
        "sin" => match &args[..] {
            [Value::Number(n, _)] => Ok(Value::Number(n.to_radians().sin(), None)),
            _ => Err(SassError::eval("sin() expects a number")),
        },
        "cos" => match &args[..] {
            [Value::Number(n, _)] => Ok(Value::Number(n.to_radians().cos(), None)),
            _ => Err(SassError::eval("cos() expects a number")),
        },
        "tan" => match &args[..] {
            [Value::Number(n, _)] => Ok(Value::Number(n.to_radians().tan(), None)),
            _ => Err(SassError::eval("tan() expects a number")),
        },
        "asin" => match &args[..] {
            [Value::Number(n, _)] => Ok(Value::Number(n.asin().to_degrees(), None)),
            _ => Err(SassError::eval("asin() expects a number")),
        },
        "acos" => match &args[..] {
            [Value::Number(n, _)] => Ok(Value::Number(n.acos().to_degrees(), None)),
            _ => Err(SassError::eval("acos() expects a number")),
        },
        "atan" => match &args[..] {
            [Value::Number(n, _)] => Ok(Value::Number(n.atan().to_degrees(), None)),
            _ => Err(SassError::eval("atan() expects a number")),
        },
        "atan2" => match &args[..] {
            [Value::Number(y, _), Value::Number(x, _)] => {
                Ok(Value::Number(y.atan2(*x).to_degrees(), None))
            }
            _ => Err(SassError::eval("atan2() expects two numbers")),
        },
        _ => Err(SassError::eval(format!("Unknown math function: {field}"))),
    }
}
