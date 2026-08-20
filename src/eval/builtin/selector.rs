//! Selector 内建函数。
//!
//! 包含 selector-append/nest/is-super/parse/simple-selectors/unify/extend/replace。
//! 支持命名参数（如 `selector.parse($selector: "c")`）。

use crate::error::{Result, SassError};
use crate::parse::ast::*;
use im::HashMap;

/// 返回每个 selector 函数的参数名列表（按位置顺序）。
fn selector_param_names(name: &str) -> &'static [&'static str] {
    match name {
        "selector-parse" => &["selector"],
        "selector-append" => &[],
        "selector-nest" => &[],
        "selector-is-superselector" | "selector-is-super" => &["super", "sub"],
        "selector-simple-selectors" => &["selector"],
        "selector-unify" => &["selector1", "selector2"],
        "selector-extend" => &["selector", "extendee", "extender"],
        "selector-replace" => &["selector", "original", "replacement"],
        _ => &[],
    }
}

/// 合并位置参数和命名参数。
fn merge_selector_args(pos_args: &[Value], kw_args: &HashMap<String, Value>, name: &str) -> Vec<Value> {
    let param_names = selector_param_names(name);
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
    if pos_args.len() > param_names.len() {
        result.extend_from_slice(&pos_args[param_names.len()..]);
    }
    result
}

pub fn call(name: &str, pos_args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Option<Value>> {
    let args = merge_selector_args(pos_args, kw_args, name);
    let args = args.as_slice();
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
        "selector-is-superselector" | "selector-is-super" => match args {
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
            _ => Err(SassError::Eval("selector-parse requires 1 argument".into())),
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
                "selector-simple-selectors requires 1 argument".into(),
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
            _ => Err(SassError::Eval("selector-extend requires 3 arguments".into())),
        },
        "selector-replace" => match args {
            [
                Value::String(selector, _),
                Value::String(original, _),
                Value::String(replacement, _),
            ] => {
                // 简化实现：在整个选择器中替换 original 为 replacement
                let result = selector.replace(original.as_str(), replacement.as_str());
                Ok(Some(Value::String(result, false)))
            }
            _ => Err(SassError::Eval("selector-replace requires 3 arguments".into())),
        },
        _ => Ok(None),
    }
}
