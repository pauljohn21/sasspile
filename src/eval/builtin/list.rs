//! List 内建函数。
//!
//! 包含 length/nth/append/join/index/separator/set-nth/is-bracketed/list-slash/zip。

use super::super::Evaluator;
use crate::error::{Result, SassError};
use crate::parse::ast::*;

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
                let actual = if idx > 0 {
                    (idx as usize).saturating_sub(1)
                } else if idx < 0 {
                    (len + idx) as usize
                } else {
                    return Err(SassError::Eval("nth 索引 0 無効（1 から開始）".into()));
                };
                Ok(Some(es.get(actual).cloned().ok_or_else(|| {
                    SassError::Eval(format!("nth 索引 {idx} 超出範囲"))
                })?))
            }
            [Value::Map(pairs), Value::Number(n, _)] => {
                let len = pairs.len() as i64;
                let idx = *n as i64;
                let actual = if idx > 0 {
                    (idx as usize).saturating_sub(1)
                } else if idx < 0 {
                    (len + idx) as usize
                } else {
                    return Err(SassError::Eval("nth 索引 0 无効".into()));
                };
                Ok(Some(
                    pairs
                        .get(actual)
                        .map(|(k, v)| {
                            Value::List(vec![k.clone(), v.clone()], Separator::Space, false)
                        })
                        .ok_or_else(|| SassError::Eval(format!("nth 索引 {idx} 超出范围")))?,
                ))
            }
            [other, Value::Number(1.0, _)] => Ok(Some(other.clone())),
            [other, Value::Number(-1.0, _)] => Ok(Some(other.clone())),
            _ => Err(SassError::Eval("nth 需要 (list, n) 参数".into())),
        },
        "append" => match args {
            [Value::List(items, sep, bracketed), val] => {
                let mut new_items = items.clone();
                new_items.push(val.clone());
                // Undecided separator → Space when appending
                let new_sep = match sep {
                    Separator::Undecided => Separator::Space,
                    other => other.clone(),
                };
                Ok(Some(Value::List(new_items, new_sep, *bracketed)))
            }
            [Value::List(items, sep, bracketed), val, Value::String(s, _)] => {
                let new_sep = match s.as_str() {
                    "comma" => Separator::Comma,
                    "space" => Separator::Space,
                    "slash" => Separator::Slash,
                    _ => match sep {
                        Separator::Undecided => Separator::Space,
                        other => other.clone(),
                    },
                };
                let mut new_items = items.clone();
                new_items.push(val.clone());
                Ok(Some(Value::List(new_items, new_sep, *bracketed)))
            }
            [Value::Map(pairs), val] => {
                if pairs.is_empty() {
                    // 空映射 = 空列表 → 返回单元素 space 列表
                    Ok(Some(Value::List(vec![val.clone()], Separator::Space, false)))
                } else {
                    // 非空 Map → comma-separated list of space-separated pairs
                    let items: Vec<Value> = pairs
                        .iter()
                        .map(|(k, v)| Value::List(vec![k.clone(), v.clone()], Separator::Space, false))
                        .collect();
                    let mut new_items = items;
                    new_items.push(val.clone());
                    Ok(Some(Value::List(new_items, Separator::Comma, false)))
                }
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
        "join" => {
            if args.len() < 2 || args.len() > 4 {
                return Err(SassError::Eval("join 需要 2-4 个参数".into()));
            }
            // 提取 list1 的 items 和 separator
            let (a_items, a_sep, a_bracketed) = match &args[0] {
                Value::List(items, sep, br) => (items.clone(), sep.clone(), *br),
                Value::Map(pairs) => {
                    let items: Vec<Value> = pairs.iter().map(|(k, v)| {
                        Value::List(vec![k.clone(), v.clone()], Separator::Space, false)
                    }).collect();
                    (items, Separator::Comma, false)
                }
                other => (vec![other.clone()], Separator::Undecided, false),
            };
            // 提取 list2 的 items 和 separator
            let (b_items, b_sep, _) = match &args[1] {
                Value::List(items, sep, _) => (items.clone(), sep.clone(), false),
                Value::Map(pairs) => {
                    let items: Vec<Value> = pairs.iter().map(|(k, v)| {
                        Value::List(vec![k.clone(), v.clone()], Separator::Space, false)
                    }).collect();
                    (items, Separator::Comma, false)
                }
                other => (vec![other.clone()], Separator::Undecided, false),
            };
            // 解析 separator 参数
            let sep = if let Some(Value::String(s, _)) = args.get(2) {
                match s.as_str() {
                    "comma" => Separator::Comma,
                    "space" => Separator::Space,
                    "slash" => Separator::Slash,
                    "auto" | _ => {
                        if a_sep == Separator::Undecided { b_sep } else { a_sep }
                    }
                }
            } else {
                // 无 separator 参数 → auto
                if a_sep == Separator::Undecided { b_sep } else { a_sep }
            };
            // 解析 bracketed 参数
            let bracketed = if let Some(Value::Bool(b)) = args.get(3) {
                *b
            } else if let Some(Value::String(s, _)) = args.get(3) {
                s == "auto"
            } else {
                // auto → 使用 list1 的 bracketed
                a_bracketed
            };
            let mut items = a_items;
            items.extend(b_items);
            Ok(Some(Value::List(items, sep, bracketed)))
        }
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
                if Evaluator::values_eq(other, needle) {
                    Ok(Some(Value::Number(1.0, None)))
                } else {
                    Ok(Some(Value::Null))
                }
            }
            _ => Err(SassError::Eval("index 需要 2 个参数".into())),
        },
        "list-separator" | "separator" => {
            if args.len() != 1 {
                return Err(SassError::Eval(format!(
                    "Only 1 argument allowed, but {} {} passed.",
                    args.len(),
                    if args.len() == 1 { "was" } else { "were" }
                )));
            }
            match &args[0] {
                Value::List(_, Separator::Comma, _) => {
                    Ok(Some(Value::String("comma".into(), false)))
                }
                Value::List(_, Separator::Space, _) => {
                    Ok(Some(Value::String("space".into(), false)))
                }
                Value::List(_, Separator::Slash, _) => {
                    Ok(Some(Value::String("slash".into(), false)))
                }
                Value::List(_, Separator::Undecided, _) => {
                    Ok(Some(Value::String("space".into(), false)))
                }
                Value::Map(pairs) => {
                    if pairs.is_empty() {
                        Ok(Some(Value::String("space".into(), false)))
                    } else {
                        Ok(Some(Value::String("comma".into(), false)))
                    }
                }
                _ => Ok(Some(Value::String("space".into(), false))),
            }
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
            [a, b] => Ok(Some(Value::List(
                vec![a.clone(), b.clone()],
                Separator::Slash,
                false,
            ))),
            _ => Err(SassError::Eval("list-slash 需要 2 个参数".into())),
        },
        "zip" => {
            if args.len() < 2 {
                return Err(SassError::Eval("zip 需要 2+ 个列表参数".into()));
            }
            // 将每个参数转为列表（非列表值视为单元素列表）
            let lists: Vec<Vec<Value>> = args
                .iter()
                .map(|v| match v {
                    Value::List(items, _, _) => items.clone(),
                    other => vec![other.clone()],
                })
                .collect();
            let min_len = lists.iter().map(|l| l.len()).min().unwrap_or(0);
            let pairs: Vec<Value> = (0..min_len)
                .map(|i| {
                    Value::List(
                        lists.iter().map(|l| l[i].clone()).collect(),
                        Separator::Space,
                        false,
                    )
                })
                .collect();
            Ok(Some(Value::List(pairs, Separator::Comma, false)))
        }
        _ => Ok(None),
    }
}
