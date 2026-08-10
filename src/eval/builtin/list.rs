//! List 内建函数。
//!
//! 包含 length/nth/append/join/index/separator/set-nth/is-bracketed/list-slash/zip。

use crate::error::{Result, SassError};
use crate::parse::ast::*;
use super::super::Evaluator;

pub fn call(name: &str, args: &[Value]) -> Result<Option<Value>> {
    match name {
        "length" | "list-length" => match args {
            [Value::List(es, _, _)] => Ok(Some(Value::Number(es.len() as f64, None))),
            [Value::Map(pairs)] => Ok(Some(Value::Number(pairs.len() as f64, None))),
            [_] => Ok(Some(Value::Number(1.0, None))),
            _ => Err(SassError::Eval("length 需要 1 个参数".into())),
        },
        "nth" => match args {
            [Value::List(es, _, _), Value::Number(n, _)] => {
                let len = es.len() as i64;
                let idx = *n as i64;
                let actual = if idx > 0 { (idx as usize).saturating_sub(1) }
                else if idx < 0 { ((len + idx) as usize).saturating_sub(1) }
                else { return Err(SassError::Eval("nth 索引 0 无效（从 1 开始）".into())); };
                Ok(Some(es.get(actual).cloned().ok_or_else(|| SassError::Eval(format!("nth 索引 {idx} 超出范围")))?))
            }
            [Value::Map(pairs), Value::Number(n, _)] => {
                let len = pairs.len() as i64;
                let idx = *n as i64;
                let actual = if idx > 0 { (idx as usize).saturating_sub(1) }
                else if idx < 0 { ((len + idx) as usize).saturating_sub(1) }
                else { return Err(SassError::Eval("nth 索引 0 无效".into())); };
                Ok(Some(pairs.get(actual).map(|(k, v)| Value::List(vec![k.clone(), v.clone()], Separator::Space, false))
                .ok_or_else(|| SassError::Eval(format!("nth 索引 {idx} 超出范围")))?))
            }
            [other, Value::Number(1.0, _)] => Ok(Some(other.clone())),
            [other, Value::Number(-1.0, _)] => Ok(Some(other.clone())),
            _ => Err(SassError::Eval("nth 需要 (list, n) 参数".into())),
        },
        "append" => match args {
            [Value::List(items, sep, false), val] => {
                let mut new_items = items.clone();
                new_items.push(val.clone());
                Ok(Some(Value::List(new_items, sep.clone(), false)))
            }
            [Value::List(items, sep, false), val, Value::String(s, _)] => {
                let new_sep = match s.as_str() {
                    "comma" => Separator::Comma,
                    "space" => Separator::Space,
                    "slash" => Separator::Slash,
                    _ => sep.clone(),
                };
                let mut new_items = items.clone();
                new_items.push(val.clone());
                Ok(Some(Value::List(new_items, new_sep, false)))
            }
            [other, val] => {
                let items = match other {
                    Value::List(items, _, _) => {
                        let mut i = items.clone();
                        i.push(val.clone());
                        i
                    }
                    _ => vec![other.clone(), val.clone()],
                };
                Ok(Some(Value::List(items, Separator::Space, false)))
            }
            _ => Err(SassError::Eval("append 需要 2-3 个参数".into())),
        },
        "join" => match args {
            [Value::List(a, sa, false), Value::List(b, sb, false)] => {
                let sep = if a.is_empty() { sb.clone() } else { sa.clone() };
                let mut items = a.clone();
                items.extend(b.clone());
                Ok(Some(Value::List(items, sep, false)))
            }
            [Value::List(a, sa, false), Value::List(b, sb, false), Value::String(s, _)] => {
                let sep = match s.as_str() {
                    "comma" => Separator::Comma,
                    "space" => Separator::Space,
                    "slash" => Separator::Slash,
                    _ => if a.is_empty() { sb.clone() } else { sa.clone() },
                };
                let mut items = a.clone();
                items.extend(b.clone());
                Ok(Some(Value::List(items, sep, false)))
            }
            [a, b] => {
                let (a_items, a_sep) = match a {
                    Value::List(items, sep, _) => (items.clone(), sep.clone()),
                    _ => (vec![a.clone()], Separator::Undecided),
                };
                let (b_items, b_sep) = match b {
                    Value::List(items, sep, _) => (items.clone(), sep.clone()),
                    _ => (vec![b.clone()], Separator::Undecided),
                };
                let sep = if a_items.is_empty() { b_sep } else { a_sep };
                let mut items = a_items;
                items.extend(b_items);
                Ok(Some(Value::List(items, sep, false)))
            }
            _ => Err(SassError::Eval("join 需要 2-4 个参数".into())),
        },
        "index" => match args {
            [Value::List(items, _, _), needle] => {
                for (i, item) in items.iter().enumerate() {
                    if Evaluator::values_eq(item, needle) {
                        return Ok(Some(Value::Number((i + 1) as f64, None)));
                    }
                }
                Ok(Some(Value::Null))
            }
            [other, needle] => {
                if Evaluator::values_eq(other, needle) { Ok(Some(Value::Number(1.0, None))) }
                else { Ok(Some(Value::Null)) }
            }
            _ => Err(SassError::Eval("index 需要 2 个参数".into())),
        },
        "list-separator" | "separator" => match args {
            [Value::List(_, Separator::Comma, false)] => Ok(Some(Value::String("comma".into(), false))),
            [Value::List(_, Separator::Space, false)] => Ok(Some(Value::String("space".into(), false))),
            [Value::List(_, Separator::Slash, false)] => Ok(Some(Value::String("slash".into(), false))),
            _ => Ok(Some(Value::String("space".into(), false))),
        },
        "set-nth" => match args {
            [Value::List(items, sep, false), Value::Number(n, _), val] => {
                let idx = *n as usize;
                let mut new_items = items.clone();
                if idx >= 1 && idx <= new_items.len() {
                    new_items[idx - 1] = val.clone();
                }
                Ok(Some(Value::List(new_items, sep.clone(), false)))
            }
            _ => Err(SassError::Eval("set-nth 需要 3 个参数".into())),
        },
        "is-bracketed" => match args {
            [Value::List(_, _, true)] => Ok(Some(Value::Bool(true))),
            _ => Ok(Some(Value::Bool(false))),
        },
        "list-slash" => match args {
            [a, b] => Ok(Some(Value::List(vec![a.clone(), b.clone()], Separator::Slash, false))),
            _ => Err(SassError::Eval("list-slash 需要 2 个参数".into())),
        },
        "zip" => match args {
            [Value::List(a, _, _), Value::List(b, _, _)] => {
                let pairs: Vec<Value> = a.iter().zip(b.iter()).map(|(x, y)| {
                    Value::List(vec![x.clone(), y.clone()], Separator::Space, false)
                }).collect();
                Ok(Some(Value::List(pairs, Separator::Comma, false)))
            }
            _ => Err(SassError::Eval("zip 需要 2+ 个列表参数".into())),
        },
        _ => Ok(None),
    }
}
