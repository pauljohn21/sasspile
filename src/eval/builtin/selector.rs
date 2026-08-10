//! Selector 内建函数。
//!
//! 包含 selector-append/nest/is-super/parse/simple-selectors/unify/extend。

use crate::error::{Result, SassError};
use crate::parse::ast::*;

pub fn call(name: &str, args: &[Value]) -> Result<Option<Value>> {
    match name {
        "selector-append" => {
            let parts: Vec<String> = args
                .iter()
                .map(|a| match a {
                    Value::String(s, _) => s.clone(),
                    _ => a.to_string(),
                })
                .collect();
            Ok(Some(Value::String(parts.join(""), false)))
        }
        "selector-nest" => {
            let parts: Vec<String> = args
                .iter()
                .map(|a| match a {
                    Value::String(s, _) => s.clone(),
                    _ => a.to_string(),
                })
                .collect();
            Ok(Some(Value::String(parts.join(" "), false)))
        }
        "selector-is-super" => match args {
            [Value::String(a, _), Value::String(b, _)] => {
                Ok(Some(Value::Bool(b.contains(a.as_str()))))
            }
            _ => Ok(Some(Value::Bool(false))),
        },
        "selector-parse" => match args {
            [Value::String(s, _)] => {
                let parts: Vec<Value> = s
                    .split(',')
                    .map(|p| Value::String(p.trim().to_string(), false))
                    .collect();
                Ok(Some(Value::List(parts, Separator::Comma, false)))
            }
            _ => Err(SassError::Eval("selector-parse 需要 1 个参数".into())),
        },
        "selector-simple-selectors" => match args {
            [Value::String(s, _)] => {
                let mut result = Vec::new();
                let mut current = String::new();
                for c in s.chars() {
                    if c == '.' || c == '#' || c == ':' || c == '[' {
                        if !current.is_empty() {
                            result.push(Value::String(current.clone(), false));
                        }
                        current = c.to_string();
                    } else {
                        current.push(c);
                    }
                }
                if !current.is_empty() {
                    result.push(Value::String(current, false));
                }
                Ok(Some(Value::List(result, Separator::Comma, false)))
            }
            _ => Err(SassError::Eval(
                "selector-simple-selectors 需要 1 个参数".into(),
            )),
        },
        "selector-unify" => match args {
            [Value::String(a, _), Value::String(b, _)] => {
                if a.contains(b.as_str()) {
                    Ok(Some(Value::String(a.clone(), false)))
                } else if b.contains(a.as_str()) {
                    Ok(Some(Value::String(b.clone(), false)))
                } else {
                    Ok(Some(Value::String(format!("{a}{b}"), false)))
                }
            }
            _ => Ok(Some(Value::Null)),
        },
        "selector-extend" => match args {
            [
                Value::String(selector, _),
                Value::String(target, _),
                Value::String(extender, _),
            ] => {
                let result = if selector.contains(target.as_str()) {
                    format!("{selector}, {extender}")
                } else {
                    selector.clone()
                };
                Ok(Some(Value::String(result, false)))
            }
            _ => Err(SassError::Eval("selector-extend 需要 3 个参数".into())),
        },
        _ => Ok(None),
    }
}
