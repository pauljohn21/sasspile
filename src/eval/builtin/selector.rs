//! Selector 内建函数。
//!
//! 包含 selector-append/nest/is-super/parse/simple-selectors/unify/extend。

use crate::error::{Result, SassError};
use crate::eval::selector::algorithms;
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
            // selector-nest 接受字符串或列表（解析后的选择器），返回组合后的选择器列表
            if args.is_empty() {
                return Ok(Some(Value::String(String::new(), false)));
            }
            // 将每个参数展平为选择器字符串列表
            let lists: Vec<Vec<String>> = args.iter().map(value_to_selector_strings).collect();
            // 笛卡尔积组合
            let mut result = vec![String::new()];
            for list in &lists {
                let mut next = Vec::new();
                for prefix in &result {
                    for sel in list {
                        if prefix.is_empty() {
                            next.push(sel.clone());
                        } else {
                            next.push(format!("{prefix} {sel}"));
                        }
                    }
                }
                result = next;
            }
            if result.len() == 1 {
                Ok(Some(Value::String(result[0].clone(), false)))
            } else {
                Ok(Some(Value::List(
                    result
                        .into_iter()
                        .map(|s| Value::String(s, false))
                        .collect(),
                    Separator::Comma,
                    false,
                )))
            }
        }
        "selector-is-superselector" => match args {
            [Value::String(a, _), Value::String(b, _)] => {
                algorithms::is_superselector(a, b).map(Some)
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
            [Value::String(a, _), Value::String(b, _)] => algorithms::unify(a, b).map(Some),
            _ => Ok(Some(Value::Null)),
        },
        "selector-extend" => {
            // selector-extend 接受字符串或列表（解析后的选择器）
            if args.len() == 3 {
                let selector = value_to_selector_string(&args[0]);
                let target = value_to_selector_string(&args[1]);
                let extender = value_to_selector_string(&args[2]);
                algorithms::extend(&selector, &target, &extender).map(Some)
            } else if args.len() == 2 {
                // 2 参数形式：(selector, extender)
                let selector = value_to_selector_string(&args[0]);
                let extender = value_to_selector_string(&args[1]);
                algorithms::extend(&selector, &selector, &extender).map(Some)
            } else {
                Err(SassError::Eval("selector-extend 需要 2-3 个参数".into()))
            }
        }
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
            _ => Err(SassError::Eval("selector-replace 需要 3 个参数".into())),
        },
        _ => Ok(None),
    }
}

/// 将 Value（字符串或列表）转换为选择器字符串列表。
fn value_to_selector_strings(val: &Value) -> Vec<String> {
    match val {
        Value::String(s, _) => vec![s.clone()],
        Value::List(items, _, _) => items.iter().map(value_to_selector_string).collect(),
        Value::Null => vec![],
        _ => vec![val.to_string()],
    }
}

/// 将 Value（字符串或列表）转换为逗号分隔的选择器字符串。
fn value_to_selector_string(val: &Value) -> String {
    match val {
        Value::String(s, _) => s.clone(),
        Value::List(items, _, _) => items
            .iter()
            .map(|v| match v {
                Value::String(s, _) => s.clone(),
                _ => v.to_string(),
            })
            .collect::<Vec<_>>()
            .join(", "),
        Value::Null => String::new(),
        _ => val.to_string(),
    }
}
