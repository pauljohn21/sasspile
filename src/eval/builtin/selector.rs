//! Selector 内建函数。
//!
//! 包含 selector-append/nest/is-super/parse/simple-selectors/unify/extend/replace。
//! 支持命名参数（如 `selector.parse($selector: "c")`）。

use crate::error::{Result, SassError};
use crate::parse::ast::*;
use std::collections::HashMap;

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
fn merge_selector_args(
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
    name: &str,
) -> Vec<Value> {
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

pub fn call(
    name: &str,
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
) -> Result<Option<Value>> {
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
        "selector-parse" => {
            if args.is_empty() {
                return Err(SassError::Eval("Missing argument $selector.".into()));
            }
            if args.len() > 1 {
                return Err(SassError::Eval(format!(
                    "Only 1 argument allowed, but {} {} passed.",
                    args.len(),
                    if args.len() == 1 { "was" } else { "were" }
                )));
            }
            match &args[0] {
                Value::String(s, _) => {
                    let parts: Vec<Value> = s
                        .split(',')
                        .map(|p| Value::String(p.trim().to_string(), false))
                        .collect();
                    Ok(Some(Value::List(parts, Separator::Comma, false)))
                }
                other => Err(SassError::Eval(format!(
                    "$selector: {other} is not a string."
                ))),
            }
        }
        "selector-simple-selectors" => {
            if args.is_empty() {
                return Err(SassError::Eval("Missing argument $selector.".into()));
            }
            if args.len() > 1 {
                return Err(SassError::Eval(format!(
                    "Only 1 argument allowed, but {} {} passed.",
                    args.len(),
                    if args.len() == 1 { "was" } else { "were" }
                )));
            }
            match &args[0] {
                Value::String(s, _) => {
                    let (result, current) = s.chars().fold(
                        (Vec::<Value>::new(), String::new()),
                        |(mut result, mut current), c| {
                            if c == '.' || c == '#' || c == ':' || c == '[' {
                                if !current.is_empty() {
                                    result.push(Value::String(current, false));
                                }
                                current = c.to_string();
                            } else {
                                current.push(c);
                            }
                            (result, current)
                        },
                    );
                    let mut result = result;
                    if !current.is_empty() {
                        result.push(Value::String(current, false));
                    }
                    Ok(Some(Value::List(result, Separator::Comma, false)))
                }
                other => Err(SassError::Eval(format!(
                    "$selector: {other} is not a string."
                ))),
            }
        }
        "selector-unify" => {
            let params = selector_param_names("selector-unify");
            if args.len() < params.len() {
                let missing = params[args.len()];
                return Err(SassError::Eval(format!("Missing argument ${missing}.")));
            }
            if args.len() > params.len() {
                return Err(SassError::Eval(format!(
                    "Only {} arguments allowed, but {} {} passed.",
                    params.len(),
                    args.len(),
                    if args.len() == 1 { "was" } else { "were" }
                )));
            }
            match args {
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
            }
        }
        "selector-extend" => {
            let params = selector_param_names("selector-extend");
            if args.len() < params.len() {
                let missing = params[args.len()];
                return Err(SassError::Eval(format!("Missing argument ${missing}.")));
            }
            if args.len() > params.len() {
                return Err(SassError::Eval(format!(
                    "Only {} arguments allowed, but {} {} passed.",
                    params.len(),
                    args.len(),
                    if args.len() == 1 { "was" } else { "were" }
                )));
            }
            match args {
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
                _ => Err(SassError::Eval(format!(
                    "$selector: {} is not a string.",
                    args[0]
                ))),
            }
        }
        "selector-replace" => {
            let params = selector_param_names("selector-replace");
            if args.len() < params.len() {
                let missing = params[args.len()];
                return Err(SassError::Eval(format!("Missing argument ${missing}.")));
            }
            if args.len() > params.len() {
                return Err(SassError::Eval(format!(
                    "Only {} arguments allowed, but {} {} passed.",
                    params.len(),
                    args.len(),
                    if args.len() == 1 { "was" } else { "were" }
                )));
            }
            match args {
                [
                    Value::String(selector, _),
                    Value::String(original, _),
                    Value::String(replacement, _),
                ] => {
                    let result = selector.replace(original.as_str(), replacement.as_str());
                    Ok(Some(Value::String(result, false)))
                }
                _ => Err(SassError::Eval(format!(
                    "$selector: {} is not a string.",
                    args[0]
                ))),
            }
        }
        _ => Ok(None),
    }
}
