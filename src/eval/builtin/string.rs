//! String 内建函数。
//!
//! 包含 str-length/to-upper-case/to-lower-case/unquote/quote/
//! str-slice/str-index/str-insert/str-split/unique-id。

use super::super::Evaluator;
use crate::error::{Result, SassError};
use crate::parse::ast::*;
use std::collections::HashMap;

/// 返回每个 string 函数的参数名列表（按位置顺序）。
/// 用于将命名参数（kw_args）按参数名映射到位置参数。
fn string_param_names(name: &str) -> &'static [&'static str] {
    match name {
        "str-length" => &["string"],
        "to-upper-case" => &["string"],
        "to-lower-case" => &["string"],
        "unquote" => &["string"],
        "quote" => &["string"],
        "str-slice" => &["string", "start-at", "end-at"],
        "str-index" => &["string", "substring"],
        "str-insert" => &["string", "insert", "index"],
        "str-split" => &["string", "separator", "limit"],
        "unique-id" => &[],
        _ => &[],
    }
}

/// 将位置参数和命名参数合并为统一的位置参数列表。
/// 按 `param_names` 顺序填充：先取 pos_args 对应位置，不足的从 kw_args 按参数名查找。
pub(crate) fn merge_args(
    pos_args: &[Value],
    kw_args: &HashMap<String, Value>,
    param_names: &[&str],
) -> Vec<Value> {
    let mut result = Vec::with_capacity(param_names.len());
    for (i, pname) in param_names.iter().enumerate() {
        if i < pos_args.len() {
            result.push(pos_args[i].clone());
        } else if let Some(v) = kw_args.get(*pname) {
            result.push(v.clone());
        } else if let Some(v) = kw_args.get(&format!("${pname}")) {
            result.push(v.clone());
        }
        // 如果两个地方都没有，就不添加——让函数自身处理缺少参数的情况
    }
    // 如果 pos_args 比 param_names 多（如 rest 参数），追加剩余
    if pos_args.len() > param_names.len() {
        result.extend_from_slice(&pos_args[param_names.len()..]);
    }
    result
}

impl Evaluator {
    /// String 函数分派。返回 Ok(Some(value)) 表示已处理，Ok(None) 表示不匹配。
    pub(crate) fn call_string_builtin(
        name: &str,
        pos_args: &[Value],
        kw_args: &HashMap<String, Value>,
    ) -> Result<Option<Value>> {
        let params = string_param_names(name);
        let args = merge_args(pos_args, kw_args, params);
        let result = match name {
            "str-length" => {
                if args.is_empty() {
                    return Err(SassError::Eval("Missing argument $string.".into()));
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
                        Value::Number(s.chars().count() as f64, None)
                    }
                    other => return Err(SassError::Eval(format!(
                        "$string: {} is not a string.", other
                    ))),
                }
            }
            "to-upper-case" => {
                if args.is_empty() {
                    return Err(SassError::Eval("Missing argument $string.".into()));
                }
                if args.len() > 1 {
                    return Err(SassError::Eval(format!(
                        "Only 1 argument allowed, but {} {} passed.",
                        args.len(),
                        if args.len() == 1 { "was" } else { "were" }
                    )));
                }
                match &args[0] {
                    Value::String(s, q) => {
                        // Dart Sass 只转换 ASCII a-z → A-Z
                        let uppered: String = s
                            .chars()
                            .map(|c| {
                                if c.is_ascii_lowercase() {
                                    c.to_ascii_uppercase()
                                } else {
                                    c
                                }
                            })
                            .collect();
                        Value::String(uppered, *q)
                    }
                    other => return Err(SassError::Eval(format!(
                        "$string: {} is not a string.", other
                    ))),
                }
            }
            "to-lower-case" => {
                if args.is_empty() {
                    return Err(SassError::Eval("Missing argument $string.".into()));
                }
                if args.len() > 1 {
                    return Err(SassError::Eval(format!(
                        "Only 1 argument allowed, but {} {} passed.",
                        args.len(),
                        if args.len() == 1 { "was" } else { "were" }
                    )));
                }
                match &args[0] {
                    Value::String(s, q) => {
                        // Dart Sass 只转换 ASCII A-Z → a-z
                        let lowered: String = s
                            .chars()
                            .map(|c| {
                                if c.is_ascii_uppercase() {
                                    c.to_ascii_lowercase()
                                } else {
                                    c
                                }
                            })
                            .collect();
                        Value::String(lowered, *q)
                    }
                    other => return Err(SassError::Eval(format!(
                        "$string: {} is not a string.", other
                    ))),
                }
            }
            "unquote" => {
                if args.is_empty() {
                    return Err(SassError::Eval("Missing argument $string.".into()));
                }
                if args.len() > 1 {
                    return Err(SassError::Eval(format!(
                        "Only 1 argument allowed, but {} {} passed.",
                        args.len(),
                        if args.len() == 1 { "was" } else { "were" }
                    )));
                }
                match &args[0] {
                    Value::String(s, _) => Value::String(s.clone(), false),
                    other => return Err(SassError::Eval(format!(
                        "$string: {} is not a string.", other
                    ))),
                }
            }
            "quote" => {
                if args.is_empty() {
                    return Err(SassError::Eval("Missing argument $string.".into()));
                }
                if args.len() > 1 {
                    return Err(SassError::Eval(format!(
                        "Only 1 argument allowed, but {} {} passed.",
                        args.len(),
                        if args.len() == 1 { "was" } else { "were" }
                    )));
                }
                match &args[0] {
                    Value::String(s, _) => Value::String(s.clone(), true),
                    other => return Err(SassError::Eval(format!(
                        "$string: {} is not a string.", other
                    ))),
                }
            }
            "str-slice" => Self::str_slice(&args)?,
            "str-index" => {
                if args.len() != 2 {
                    return Err(SassError::Eval("str-index requires 2 arguments".into()));
                }
                let s = match &args[0] {
                    Value::String(s, _) => s.clone(),
                    other => return Err(SassError::Eval(format!("$string: {} is not a string.", other))),
                };
                let needle = match &args[1] {
                    Value::String(needle, _) => needle.clone(),
                    other => return Err(SassError::Eval(format!("$substring: {} is not a string.", other))),
                };
                match s.find(&needle) {
                    Some(pos) => {
                        Value::Number((s[..pos].chars().count() + 1) as f64, None)
                    }
                    None => Value::Null,
                }
            }
            "str-insert" => Self::str_insert(&args)?,
            "str-split" => Self::str_split(&args)?,
            "unique-id" => {
                if !args.is_empty() {
                    return Err(SassError::Eval(format!(
                        "Only 0 arguments allowed, but {} {} passed.",
                        args.len(),
                        if args.len() == 1 { "was" } else { "were" }
                    )));
                }
                use std::sync::atomic::{AtomicU64, Ordering};
                static COUNTER: AtomicU64 = AtomicU64::new(1);
                let id = COUNTER.fetch_add(1, Ordering::SeqCst);
                Value::String(format!("u{id}"), false)
            }
            _ => return Ok(None),
        };
        Ok(Some(result))
    }

    /// str-slice($string, $start-at, $end-at: -1)
    fn str_slice(args: &[Value]) -> Result<Value> {
        if args.len() < 2 || args.len() > 3 {
            return Err(SassError::Eval("str-slice requires 2-3 arguments".into()));
        }
        let (s, q) = match &args[0] {
            Value::String(s, q) => (s.clone(), *q),
            other => return Err(SassError::Eval(format!("$string: {} is not a string.", other))),
        };
        let start = match &args[1] {
            Value::Number(n, u) => {
                if n.fract() != 0.0 {
                    return Err(SassError::Eval(format!("$start-at: {} is not an int.", n)));
                }
                if u.is_some() {
                    return Err(SassError::Eval(format!("$start-at: Expected {} to have no units.", n)));
                }
                *n as isize
            }
            other => return Err(SassError::Eval(format!("$start-at: {} is not a number.", other))),
        };
        let end = match args.get(2) {
            Some(Value::Number(n, u)) => {
                if n.fract() != 0.0 {
                    return Err(SassError::Eval(format!("$end-at: {} is not an int.", n)));
                }
                if u.is_some() {
                    return Err(SassError::Eval(format!("$end-at: Expected {} to have no units.", n)));
                }
                Some(*n as isize)
            }
            None => None,
            Some(other) => return Err(SassError::Eval(format!("$end-at: {} is not a number.", other))),
        };
        let chars: Vec<char> = s.chars().collect();
        let len = chars.len() as isize;
        let start_idx = if start < 0 {
            (len + start).max(0) as usize
        } else {
            (start - 1).max(0) as usize
        };
        let end_idx = match end {
            None => len as usize,
            Some(e) => {
                if e < 0 {
                    (len + e + 1).max(0) as usize
                } else {
                    e.min(len) as usize
                }
            }
        };
        let result: String = chars[start_idx.min(end_idx)..end_idx.min(len as usize)]
            .iter()
            .collect();
        Ok(Value::String(result, q))
    }

    /// str-insert($string, $insert, $index)
    fn str_insert(args: &[Value]) -> Result<Value> {
        if args.len() != 3 {
            return Err(SassError::Eval("str-insert requires 3 arguments".into()));
        }
        let (s, q) = match &args[0] {
            Value::String(s, q) => (s.clone(), *q),
            other => return Err(SassError::Eval(format!("$string: {} is not a string.", other))),
        };
        let insert = match &args[1] {
            Value::String(insert, _) => insert.clone(),
            other => return Err(SassError::Eval(format!("$insert: {} is not a string.", other))),
        };
        let idx = match &args[2] {
            Value::Number(n, u) => {
                if u.is_some() {
                    return Err(SassError::Eval(format!("$index: Expected {} to have no units.", n)));
                }
                if n.fract() != 0.0 {
                    return Err(SassError::Eval(format!("$index: {} is not an int.", n)));
                }
                *n as isize
            }
            other => return Err(SassError::Eval(format!("$index: {} is not a number.", other))),
        };
        let chars: Vec<char> = s.chars().collect();
        let len = chars.len() as isize;
        let pos = if idx >= 0 {
            (idx - 1).max(0).min(len) as usize
        } else {
            (len + idx + 1).max(0) as usize
        };
        let mut result: Vec<char> = chars[..pos].to_vec();
        result.extend(insert.chars());
        result.extend(chars[pos..].iter());
        Ok(Value::String(result.into_iter().collect(), q))
    }

    /// str-split($string, $separator, $limit: null)
    fn str_split(args: &[Value]) -> Result<Value> {
        if args.len() > 3 {
            return Err(SassError::Eval(format!(
                "Only 3 arguments allowed, but {} were passed.",
                args.len()
            )));
        }
        if args.len() < 2 {
            return Err(SassError::Eval("Missing argument $separator.".into()));
        }
        let (s, input_quoted) = match &args[0] {
            Value::String(s, q) => (s.clone(), *q),
            other => return Err(SassError::Eval(format!("$string: {} is not a string.", other))),
        };
        let sep = match &args[1] {
            Value::String(sep, _) => Some(sep.clone()),
            Value::Null => None,
            other => return Err(SassError::Eval(format!("$separator: {} is not a string.", other))),
        };
        let limit = match args.get(2) {
            Some(Value::Number(n, u)) => {
                if u.is_some() {
                    return Err(SassError::Eval(format!("$limit: Expected {} to have no units.", n)));
                }
                if n.fract() != 0.0 {
                    return Err(SassError::Eval(format!("$limit: {} is not an int.", n)));
                }
                if *n < 1.0 {
                    return Err(SassError::Eval(format!("$limit: Must be 1 or greater, was {}.", *n as i64)));
                }
                Some(*n as usize)
            }
            Some(Value::Null) | None => None,
            Some(other) => return Err(SassError::Eval(format!("$limit: {} is not a number.", other))),
        };

        let parts: Vec<String> = if s.is_empty() {
            Vec::new()
        } else if let Some(sep) = sep {
            if sep.is_empty() {
                s.chars().map(|c| c.to_string()).collect()
            } else if let Some(limit) = limit {
                s.splitn(limit + 1, &sep).map(|p| p.to_string()).collect()
            } else {
                s.split(&sep).map(|p| p.to_string()).collect()
            }
        } else {
            s.chars().map(|c| c.to_string()).collect()
        };
        Ok(Value::List(
            parts
                .into_iter()
                .map(|p| Value::String(p, input_quoted))
                .collect(),
            Separator::Comma,
            true,
        ))
    }
}
