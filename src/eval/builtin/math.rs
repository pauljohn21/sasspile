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
            [Value::Number(n, _)] => Ok(Value::Number(n.abs(), None)),
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
            let nums: Vec<f64> = args.iter().filter_map(|v| match v {
                Value::Number(n, _) => Some(*n),
                _ => None,
            }).collect();
            nums.iter().copied().fold(None, |acc, n| {
                Some(acc.map_or(n, |a: f64| a.max(n)))
            }).map(|n| Value::Number(n, None))
                .ok_or_else(|| SassError::eval("max() expects numbers"))
        },
        "min" => {
            let nums: Vec<f64> = args.iter().filter_map(|v| match v {
                Value::Number(n, _) => Some(*n),
                _ => None,
            }).collect();
            nums.iter().copied().fold(None, |acc, n| {
                Some(acc.map_or(n, |a: f64| a.min(n)))
            }).map(|n| Value::Number(n, None))
                .ok_or_else(|| SassError::eval("min() expects numbers"))
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
        _ => Err(SassError::eval(format!("Unknown math function: {field}"))),
    }
}
