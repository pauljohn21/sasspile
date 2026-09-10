#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
//! List 内建函数。
//!
//! 包含 length/nth/append/join/index/separator/set-nth/is-bracketed/list-slash/zip。

use crate::error::{Result, SassError};
use crate::parse::ast::*;
use imbl::HashMap;

/// 返回每个 list 函数的参数名列表（按位置顺序）。
fn list_param_names(name: &str) -> &'static [&'static str] {
    match name {
        "length" | "list-length" => &["list"],
        "nth" => &["list", "n"],
        "append" => &["list", "val", "separator"],
        "join" => &["list1", "list2", "separator", "bracketed"],
        "index" => &["list", "value"],
        "list-separator" | "separator" => &["list"],
        "set-nth" => &["list", "n", "value"],
        "is-bracketed" => &["list"],
        "list-slash" => &[],
        "zip" => &[],
        _ => &[],
    }
}

/// 合并位置参数和命名参数（复用 string 模块的 `merge_args` 逻辑）。
fn merge_list_args(pos_args: &[Value], kw_args: &HashMap<String, Value>, name: &str) -> Vec<Value> {
    let param_names = list_param_names(name);
    let result: Vec<Value> = param_names
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
        true => {
            let mut r = result;
            r.extend_from_slice(&pos_args[param_names.len()..]);
            r
        }
        false => result,
    }
}

pub fn call(
    name: &str,
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
) -> Result<Option<Value>> {
    // join 的 $separator 命名参数校验（必须在 merge 之前，否则无法区分来源）
    if name == "join" {
        if let Some(sep_val) = kw_args.get("separator").or_else(|| kw_args.get("$separator")) {
            match sep_val {
                Value::String(s, _) => {
                    if !["comma", "space", "slash", "auto"].contains(&s.as_str()) {
                        return Err(SassError::Eval(format!(
                            "\"{s}\" is not a valid separator for join()."
                        )));
                    }
                }
                Value::Bool(_) | Value::Null => {}
                other => {
                    return Err(SassError::Eval(format!(
                        "$separator: {other} is not a valid separator argument."
                    )));
                }
            }
        }
    }
    let args = merge_list_args(pos_args, kw_args, name);
    let args = args.as_slice();
    match name {
        "length" | "list-length" => match args {
            [Value::List(es, _, _)] => Ok(Some(Value::Number(es.len() as f64, None))),
            [Value::Map(pairs)] => Ok(Some(Value::Number(pairs.len() as f64, None))),
            [_] => Ok(Some(Value::Number(1.0, None))),
            _ => Err(SassError::Eval("length requires 1 argument".into())),
        },
        "nth" => match args {
            [Value::List(es, _, _), Value::Number(n, _)] => {
                let len = es.len() as i64;
                let idx = *n as i64;
                let actual = match idx.cmp(&0) {
                    std::cmp::Ordering::Greater => (idx as usize).saturating_sub(1),
                    std::cmp::Ordering::Less => (len + idx) as usize,
                    std::cmp::Ordering::Equal => {
                        return Err(SassError::Eval(
                            "nth index 0 is invalid (starts from 1)".into(),
                        ));
                    }
                };
                Ok(Some(es.get(actual).cloned().ok_or_else(|| {
                    SassError::Eval(format!("nth index {idx} out of range"))
                })?))
            }
            [Value::Map(pairs), Value::Number(n, _)] => {
                let len = pairs.len() as i64;
                let idx = *n as i64;
                let actual = match idx.cmp(&0) {
                    std::cmp::Ordering::Greater => (idx as usize).saturating_sub(1),
                    std::cmp::Ordering::Less => (len + idx) as usize,
                    std::cmp::Ordering::Equal => {
                        return Err(SassError::Eval("nth index 0 is invalid".into()));
                    }
                };
                Ok(Some(
                    pairs
                        .get(actual)
                        .map(|(k, v)| {
                            Value::List(vec![k.clone(), v.clone()], Separator::Space, false)
                        })
                        .ok_or_else(|| SassError::Eval(format!("nth index {idx} out of range")))?,
                ))
            }
            [other, Value::Number(1.0, _)] => Ok(Some(other.clone())),
            [other, Value::Number(-1.0, _)] => Ok(Some(other.clone())),
            _ => Err(SassError::Eval("nth requires (list, n) arguments".into())),
        },
        "append" => match args {
            [Value::List(items, sep, bracketed), val] => {
                let mut new_items = items.clone();
                new_items.push(val.clone());
                // Undecided separator → Space when appending
                let new_sep = match sep {
                    Separator::Undecided => Separator::Space,
                    Separator::SlashLiteral => Separator::Slash,
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
                        Separator::SlashLiteral => Separator::Slash,
                        other => other.clone(),
                    },
                };
                let mut new_items = items.clone();
                new_items.push(val.clone());
                Ok(Some(Value::List(new_items, new_sep, *bracketed)))
            }
            [Value::Map(pairs), val] => {
                match pairs.is_empty() {
                    true => Ok(Some(Value::List(
                        vec![val.clone()],
                        Separator::Space,
                        false,
                    ))),
                    false => {
                    let items: Vec<Value> = pairs
                        .iter()
                        .map(|(k, v)| {
                            Value::List(vec![k.clone(), v.clone()], Separator::Space, false)
                        })
                        .collect();
                    let mut new_items = items;
                    new_items.push(val.clone());
                    Ok(Some(Value::List(new_items, Separator::Comma, false)))
                    }
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
            _ => Err(SassError::Eval("append requires 2-3 arguments".into())),
        },
        "join" => {
            match args.len() < 2 || args.len() > 4 {
                true => return Err(SassError::Eval("join requires 2-4 arguments".into())),
                false => {}
            }
            // 校验未知的命名参数（join 只接受 list1/list2/separator/bracketed）
            for key in kw_args.keys() {
                let bare = key.strip_prefix('$').unwrap_or(key.as_str());
                if !["list1", "list2", "separator", "bracketed"].contains(&bare) {
                    return Err(SassError::Eval(format!("Argument `{bare}` doesn't exist.")));
                }
            }
            // 校验 separator 类型——仅当 args[2] 确实是 separator 时检查
            // 当 separator 未提供而 bracketed 通过命名参数传入时，args[2] 可能是 bracketed 值
            if let Some(sep_val) = args.get(2) {
                if !matches!(sep_val, Value::String(_, _))
                    && !matches!(sep_val, Value::Bool(_))
                    && !matches!(sep_val, Value::Null)
                    && args.get(3).is_some()
                {
                    return Err(SassError::Eval(format!(
                        "$separator: {sep_val} is not a valid separator argument."
                    )));
                }
            }
            // 提取 list1 的 items 和 separator
            // 空 Map 的分隔符为 Undecided（与空列表一致），非空 Map 为 Comma
            let (a_items, a_sep, a_bracketed) = match &args[0] {
                Value::List(items, sep, br) => (items.clone(), sep.clone(), *br),
                Value::Map(pairs) => {
                    let items: Vec<Value> = pairs
                        .iter()
                        .map(|(k, v)| {
                            Value::List(vec![k.clone(), v.clone()], Separator::Space, false)
                        })
                        .collect();
                    let sep = if pairs.is_empty() { Separator::Undecided } else { Separator::Comma };
                    (items, sep, false)
                }
                other => (vec![other.clone()], Separator::Undecided, false),
            };
            // 提取 list2 的 items 和 separator
            let (b_items, b_sep, _) = match &args[1] {
                Value::List(items, sep, _) => (items.clone(), sep.clone(), false),
                Value::Map(pairs) => {
                    let items: Vec<Value> = pairs
                        .iter()
                        .map(|(k, v)| {
                            Value::List(vec![k.clone(), v.clone()], Separator::Space, false)
                        })
                        .collect();
                    let sep = if pairs.is_empty() { Separator::Undecided } else { Separator::Comma };
                    (items, sep, false)
                }
                other => (vec![other.clone()], Separator::Undecided, false),
            };
            // 解析 separator 和 bracketed 参数
            // 注意：当 separator 未提供而 bracketed 通过命名参数传入时，
            // args[2] 可能是 bracketed 值而非 separator
            let (sep, bracketed) = match args.get(3) {
                // 4+ 参数：args[2] = separator, args[3] = bracketed
                Some(bracketed_val) => {
                    let sep = match args.get(2).and_then(|v| match v {
                        Value::String(s, _) => Some(s.as_str()),
                        _ => None,
                    }) {
                        Some("comma") => Separator::Comma,
                        Some("space") => Separator::Space,
                        Some("slash") => Separator::Slash,
                        Some(s) => {
                            return Err(SassError::Eval(format!(
                                "\"{s}\" is not a valid separator for join()."
                            )));
                        }
                        None => {
                            // 无 separator → auto
                            let auto_sep = if a_sep == Separator::Undecided { b_sep } else { a_sep };
                            match auto_sep == Separator::SlashLiteral {
                                true => Separator::Slash,
                                false => auto_sep,
                            }
                        }
                    };
                    let bracketed = match bracketed_val {
                        Value::Bool(b) => *b,
                        Value::String(s, _) if s == "auto" => a_bracketed,
                        // truthy 非 bool 值（如标识符 e）视为 true
                        other => crate::eval::Evaluator::is_truthy(other),
                    };
                    (sep, bracketed)
                }
                // 3 或更少参数：args[2] 可能是 separator 或 bracketed
                // Null/非字符串非 bool → 视为 bracketed（falsy）
                None => match args.get(2) {
                    Some(Value::String(s, _)) => match s.as_str() {
                        "comma" => (Separator::Comma, a_bracketed),
                        "space" => (Separator::Space, a_bracketed),
                        "slash" => (Separator::Slash, a_bracketed),
                        // 非 separator 字符串 → 视为 bracketed 值
                        _ => {
                            let auto_sep = if a_sep == Separator::Undecided { b_sep } else { a_sep };
                            let auto_sep = match auto_sep == Separator::SlashLiteral {
                                true => Separator::Slash,
                                false => auto_sep,
                            };
                            // "auto" 表示自动检测；非空字符串视为 truthy
                            let bracketed = if s == "auto" { a_bracketed } else { !s.is_empty() };
                            (auto_sep, bracketed)
                        }
                    },
                    Some(Value::Bool(b)) => {
                        // args[2] 是 bool → bracketed
                        let auto_sep = if a_sep == Separator::Undecided { b_sep } else { a_sep };
                        let auto_sep = match auto_sep == Separator::SlashLiteral {
                            true => Separator::Slash,
                            false => auto_sep,
                        };
                        (auto_sep, *b)
                    }
                    Some(Value::Null) => {
                        // Null → bracketed = false
                        let auto_sep = if a_sep == Separator::Undecided { b_sep } else { a_sep };
                        let auto_sep = match auto_sep == Separator::SlashLiteral {
                            true => Separator::Slash,
                            false => auto_sep,
                        };
                        (auto_sep, false)
                    }
                    _ => {
                        // 无第三参数 → auto separator, 默认 bracketed
                        let auto_sep = if a_sep == Separator::Undecided { b_sep } else { a_sep };
                        let auto_sep = match auto_sep == Separator::SlashLiteral {
                            true => Separator::Slash,
                            false => auto_sep,
                        };
                        (auto_sep, a_bracketed)
                    }
                },
            };
            let mut items = a_items;
            items.extend(b_items);
            // join() 结果分隔符不得为 Undecided——默认为 Space
            let sep = match sep {
                Separator::Undecided => Separator::Space,
                other => other,
            };
            Ok(Some(Value::List(items, sep, bracketed)))
        }
        "index" => match args {
            [Value::List(items, _, _), needle] => {
                for (i, item) in items.iter().enumerate() {
                    match crate::eval::value::values_eq(item, needle) {
                        true => return Ok(Some(Value::Number((i + 1) as f64, None))),
                        false => {}
                    }
                }
                Ok(Some(Value::Null))
            }
            [Value::Map(pairs), needle] => {
                // Map index: 将每个键值对转为 [k, v] 列表与 needle 比较
                for (i, (k, v)) in pairs.iter().enumerate() {
                    let pair = Value::List(vec![k.clone(), v.clone()], Separator::Space, false);
                    match crate::eval::value::values_eq(&pair, needle) {
                        true => return Ok(Some(Value::Number((i + 1) as f64, None))),
                        false => {}
                    }
                }
                Ok(Some(Value::Null))
            }
            [other, needle] => {
                match crate::eval::value::values_eq(other, needle) {
                    true => Ok(Some(Value::Number(1.0, None))),
                    false => Ok(Some(Value::Null)),
                }
            }
            _ => Err(SassError::Eval("index requires 2 arguments".into())),
        },
        "list-separator" | "separator" => {
            match args.len() != 1 {
                true => return Err(SassError::Eval(format!(
                    "Only 1 argument allowed, but {} {} passed.",
                    args.len(),
                    match args.len() == 1 { true => "was", false => "were" }
                ))),
                false => {}
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
                Value::List(_, Separator::SlashLiteral, _) => {
                    Ok(Some(Value::String("slash".into(), false)))
                }
                Value::List(_, Separator::Undecided, _) => {
                    Ok(Some(Value::String("space".into(), false)))
                }
                Value::Map(pairs) => {
                    match pairs.is_empty() {
                        true => Ok(Some(Value::String("space".into(), false))),
                        false => Ok(Some(Value::String("comma".into(), false))),
                    }
                }
                _ => Ok(Some(Value::String("space".into(), false))),
            }
        }
        "set-nth" => match args {
            [list_input, Value::Number(n, _), val] => {
                // Coerce Map / non-list → list (per sass-spec)
                let (items, sep, bracketed) = match list_input {
                    Value::List(items, sep, br) => (items.clone(), sep.clone(), *br),
                    Value::Map(pairs) => (
                        pairs
                            .iter()
                            .map(|(k, v)| {
                                Value::List(vec![k.clone(), v.clone()], Separator::Space, false)
                            })
                            .collect(),
                        Separator::Comma,
                        false,
                    ),
                    other => (vec![other.clone()], Separator::Undecided, false),
                };
                let len = items.len() as i64;
                let idx = *n as i64;
                let actual = match idx.cmp(&0) {
                    std::cmp::Ordering::Greater => (idx as usize).saturating_sub(1),
                    std::cmp::Ordering::Less => (len + idx) as usize,
                    std::cmp::Ordering::Equal => {
                        return Err(SassError::Eval(format!("List index {idx} may not be 0.")));
                    }
                };
                let mut new_items = items;
                match actual < new_items.len() {
                    true => new_items[actual] = val.clone(),
                    false => return Err(SassError::Eval(format!(
                        "List index {idx} is out of bounds for list of length {len}"
                    ))),
                }
                Ok(Some(Value::List(new_items, sep, bracketed)))
            }
            _ => Err(SassError::Eval("set-nth requires 3 arguments".into())),
        },
        "is-bracketed" => match args.len() {
            1 => match &args[0] {
                Value::List(_, _, true) => Ok(Some(Value::Bool(true))),
                Value::Map(_) | Value::List(_, _, false) => Ok(Some(Value::Bool(false))),
                _ => Ok(Some(Value::Bool(false))),
            },
            0 => Err(SassError::Eval("Missing argument $number.".into())),
            n => Err(SassError::Eval(format!(
                "Only 1 argument allowed, but {n} were passed."
            ))),
        },
        "list-slash" => match args.len() {
            0 | 1 => Err(SassError::Eval("Missing argument $elements.".into())),
            _ => Ok(Some(Value::List(args.to_vec(), Separator::Slash, false))),
        },
        "zip" => {
            match args.is_empty() {
                true => return Ok(Some(Value::List(Vec::new(), Separator::Comma, false))),
                false => {}
            }
            // 将 Map 也转为列表（键值对变为子列表）
            // 单列表 zip：包装每个元素为单元素列表（符合 sass-spec）
            if args.len() == 1 {
                let items = match &args[0] {
                    Value::List(items, _, _) => items.clone(),
                    Value::Map(pairs) => pairs
                        .iter()
                        .map(|(k, v)| {
                            Value::List(vec![k.clone(), v.clone()], Separator::Space, false)
                        })
                        .collect(),
                    other => vec![other.clone()],
                };
                let wrapped: Vec<Value> = items
                    .into_iter()
                    .map(|v| Value::List(vec![v], Separator::Space, false))
                    .collect();
                return Ok(Some(Value::List(wrapped, Separator::Comma, false)));
            }
            // 多列表 zip：将每个参数转为列表（非列表值视为单元素列表）
            let lists: Vec<Vec<Value>> = args
                .iter()
                .map(|v| match v {
                    Value::List(items, _, _) => items.clone(),
                    Value::Map(pairs) => pairs
                        .iter()
                        .map(|(k, v)| {
                            Value::List(vec![k.clone(), v.clone()], Separator::Space, false)
                        })
                        .collect(),
                    other => vec![other.clone()],
                })
                .collect();
            let min_len = lists.iter().map(std::vec::Vec::len).min().unwrap_or(0);
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
