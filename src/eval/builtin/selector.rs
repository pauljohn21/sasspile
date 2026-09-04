//! Selector 内建函数。
//!
//! 包含 selector-append/nest/is-super/parse/simple-selectors/unify/extend/replace。
//! 支持命名参数（如 `selector.parse($selector: "c")`）。

use crate::css::selector_ops;
use crate::css::selector_parser::parse_selector;
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
    match param_names.is_empty() {
        true => return pos_args.to_vec(),
        false => {}
    }
    let mut result: Vec<Value> = param_names
        .iter()
        .enumerate()
        .filter_map(|(i, pname)| {
            pos_args
                .get(i)
                .cloned()
                .or_else(|| kw_args.get(*pname).cloned())
                .or_else(|| kw_args.get(&format!("${pname}")).cloned())
        })
        .collect();
    match pos_args.len() > param_names.len() {
        true => result.extend_from_slice(&pos_args[param_names.len()..]),
        false => {}
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
                let super_sel = parse_selector(a);
                let sub_sel = parse_selector(b);
                Ok(Some(Value::Bool(selector_ops::is_superselector(
                    &super_sel,
                    &sub_sel,
                ))))
            }
            _ => Ok(Some(Value::Bool(false))),
        },
        "selector-parse" => {
            match args.len() {
                0 => return Err(SassError::Eval("Missing argument $selector.".into())),
                1 => {}
                n => return Err(SassError::Eval(format!(
                    "Only 1 argument allowed, but {n} {} passed.",
                    match n == 1 { true => "was", false => "were" }
                ))),
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
            match args.len() {
                0 => return Err(SassError::Eval("Missing argument $selector.".into())),
                1 => {}
                n => return Err(SassError::Eval(format!(
                    "Only 1 argument allowed, but {n} {} passed.",
                    match n == 1 { true => "was", false => "were" }
                ))),
            }
            match &args[0] {
                Value::String(s, _) => {
                    let (result, current) = s.chars().fold(
                        (Vec::<Value>::new(), String::new()),
                        |(mut result, mut current), c| {
                            match c {
                                '.' | '#' | ':' | '[' => {
                                    match current.is_empty() {
                                        false => result.push(Value::String(current, false)),
                                        true => {}
                                    }
                                    current = c.to_string();
                                }
                                _ => current.push(c),
                            }
                            (result, current)
                        },
                    );
                    let mut result = result;
                    match current.is_empty() {
                        false => result.push(Value::String(current, false)),
                        true => {}
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
            match args.len() < params.len() {
                true => {
                    let missing = params[args.len()];
                    return Err(SassError::Eval(format!("Missing argument ${missing}.")));
                }
                false => {}
            }
            match args.len() > params.len() {
                true => return Err(SassError::Eval(format!(
                    "Only {} arguments allowed, but {} {} passed.",
                    params.len(),
                    args.len(),
                    match args.len() == 1 { true => "was", false => "were" }
                ))),
                false => {}
            }
            match args {
                [Value::String(a, _), Value::String(b, _)] => {
                    let sel_a = parse_selector(a);
                    let sel_b = parse_selector(b);
                    match selector_ops::unify(&sel_a, &sel_b) {
                        Some(unified) => Ok(Some(Value::String(
                            unified.to_string(),
                            false,
                        ))),
                        None => Ok(Some(Value::Null)),
                    }
                }
                _ => Ok(Some(Value::Null)),
            }
        }
        "selector-extend" => {
            let params = selector_param_names("selector-extend");
            match args.len() < params.len() {
                true => {
                    let missing = params[args.len()];
                    return Err(SassError::Eval(format!("Missing argument ${missing}.")));
                }
                false => {}
            }
            match args.len() > params.len() {
                true => return Err(SassError::Eval(format!(
                    "Only {} arguments allowed, but {} {} passed.",
                    params.len(),
                    args.len(),
                    match args.len() == 1 { true => "was", false => "were" }
                ))),
                false => {}
            }
            match args {
                [
                    Value::String(selector, _),
                    Value::String(target, _),
                    Value::String(extender, _),
                ] => {
                    let sel = parse_selector(selector);
                    let extendee = parse_selector(target);
                    let ext = parse_selector(extender);
                    let result = selector_ops::extend_selector(&sel, &extendee, &ext);
                    Ok(Some(Value::String(result.to_string(), false)))
                }
                _ => Err(SassError::Eval(format!(
                    "$selector: {} is not a string.",
                    args[0]
                ))),
            }
        }
        "selector-replace" => {
            let params = selector_param_names("selector-replace");
            match args.len() < params.len() {
                true => {
                    let missing = params[args.len()];
                    return Err(SassError::Eval(format!("Missing argument ${missing}.")));
                }
                false => {}
            }
            match args.len() > params.len() {
                true => return Err(SassError::Eval(format!(
                    "Only {} arguments allowed, but {} {} passed.",
                    params.len(),
                    args.len(),
                    match args.len() == 1 { true => "was", false => "were" }
                ))),
                false => {}
            }
            match args {
                [
                    Value::String(selector, _),
                    Value::String(original, _),
                    Value::String(replacement, _),
                ] => {
                    let sel = parse_selector(selector);
                    let orig = parse_selector(original);
                    let repl = parse_selector(replacement);
                    let result = selector_ops::replace_selector(&sel, &orig, &repl);
                    Ok(Some(Value::String(result.to_string(), false)))
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
