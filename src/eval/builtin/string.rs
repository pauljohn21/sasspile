//! String 内建函数。
//!
//! 包含 str-length/to-upper-case/to-lower-case/unquote/quote/
//! str-slice/str-index/str-insert/str-split/unique-id。

use super::super::Evaluator;
use crate::error::{Result, SassError};
use crate::parse::ast::*;

impl Evaluator {
    /// String 函数分派。返回 Ok(Some(value)) 表示已处理，Ok(None) 表示不匹配。
    /// 支持关键字参数（如 string.to-lower-case($string: abc)）。
    pub(crate) fn call_string_builtin(
        name: &str,
        pos_args: &[Value],
        kw_args: &std::collections::HashMap<String, Value>,
    ) -> Result<Option<Value>> {
        let result = match name {
            "str-length" => {
                if pos_args.len() > 1 {
                    return Err(SassError::Eval(format!(
                        "Only 1 argument allowed, but {} were passed.",
                        pos_args.len()
                    )));
                }
                match Self::get_str_arg(pos_args, kw_args, 0, "string") {
                    Some(Value::String(s, _)) => Value::Number(s.chars().count() as f64, None),
                    Some(other) => return Err(SassError::Eval(format!("$string: {other} is not a string."))),
                    None => return Err(SassError::Eval("Missing argument $string.".into())),
                }
            }
            "to-upper-case" => {
                if pos_args.len() > 1 {
                    return Err(SassError::Eval(format!(
                        "Only 1 argument allowed, but {} were passed.",
                        pos_args.len()
                    )));
                }
                match Self::get_str_arg(pos_args, kw_args, 0, "string") {
                    Some(Value::String(s, q)) => {
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
                    Some(other) => return Err(SassError::Eval(format!("$string: {other} is not a string."))),
                    None => return Err(SassError::Eval("Missing argument $string.".into())),
                }
            }
            "to-lower-case" => {
                if pos_args.len() > 1 {
                    return Err(SassError::Eval(format!(
                        "Only 1 argument allowed, but {} were passed.",
                        pos_args.len()
                    )));
                }
                match Self::get_str_arg(pos_args, kw_args, 0, "string") {
                    Some(Value::String(s, q)) => {
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
                    Some(other) => return Err(SassError::Eval(format!("$string: {other} is not a string."))),
                    None => return Err(SassError::Eval("Missing argument $string.".into())),
                }
            }
            "unquote" => {
                if pos_args.len() > 1 {
                    return Err(SassError::Eval(format!(
                        "Only 1 argument allowed, but {} were passed.",
                        pos_args.len()
                    )));
                }
                match Self::get_str_arg(pos_args, kw_args, 0, "string") {
                    Some(Value::String(s, _)) => Value::String(s.clone(), false),
                    Some(other) => return Err(SassError::Eval(format!("$string: {other} is not a string."))),
                    None => return Err(SassError::Eval("Missing argument $string.".into())),
                }
            }
            "quote" => {
                if pos_args.len() > 1 {
                    return Err(SassError::Eval(format!(
                        "Only 1 argument allowed, but {} were passed.",
                        pos_args.len()
                    )));
                }
                match Self::get_str_arg(pos_args, kw_args, 0, "string") {
                    Some(Value::String(s, _)) => Value::String(s.clone(), true),
                    Some(other) => return Err(SassError::Eval(format!("$string: {other} is not a string."))),
                    None => return Err(SassError::Eval("Missing argument $string.".into())),
                }
            }
            "str-slice" => {
                if pos_args.len() > 3 {
                    return Err(SassError::Eval(format!(
                        "Only 3 arguments allowed, but {} were passed.",
                        pos_args.len()
                    )));
                }
                Self::str_slice(pos_args, kw_args)?
            }
            "str-index" => {
                if pos_args.len() > 2 {
                    return Err(SassError::Eval(format!(
                        "Only 2 arguments allowed, but {} were passed.",
                        pos_args.len()
                    )));
                }
                let s = match Self::get_str_arg(pos_args, kw_args, 0, "string") {
                    Some(Value::String(s, _)) => s.clone(),
                    Some(other) => return Err(SassError::Eval(format!("$string: {other} is not a string."))),
                    None => return Err(SassError::Eval("Missing argument $string.".into())),
                };
                let needle = match Self::get_str_arg(pos_args, kw_args, 1, "substring") {
                    Some(Value::String(needle, _)) => needle.clone(),
                    Some(other) => return Err(SassError::Eval(format!("$substring: {other} is not a string."))),
                    None => return Err(SassError::Eval("Missing argument $substring.".into())),
                };
                match s.find(&needle) {
                    Some(pos) => Value::Number((s[..pos].chars().count() + 1) as f64, None),
                    None => Value::Null,
                }
            }
            "str-insert" => {
                if pos_args.len() > 3 {
                    return Err(SassError::Eval(format!(
                        "Only 3 arguments allowed, but {} were passed.",
                        pos_args.len()
                    )));
                }
                Self::str_insert(pos_args, kw_args)?
            }
            "str-split" => {
                if pos_args.len() > 3 {
                    return Err(SassError::Eval(format!(
                        "Only 3 arguments allowed, but {} were passed.",
                        pos_args.len()
                    )));
                }
                Self::str_split(pos_args, kw_args)?
            }
            "unique-id" => {
                if !pos_args.is_empty() || !kw_args.is_empty() {
                    let n = pos_args.len() + kw_args.len();
                    return Err(SassError::Eval(format!(
                        "Only 0 arguments allowed, but {} {} passed.",
                        n,
                        if n == 1 { "was" } else { "were" }
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

    /// 辅助：从 pos_args 或 kw_args 按位置/名字取参数
    fn get_str_arg<'a>(
        pos_args: &'a [Value],
        kw_args: &'a std::collections::HashMap<String, Value>,
        pos: usize,
        kw: &str,
    ) -> Option<&'a Value> {
        pos_args.get(pos)
            .or_else(|| kw_args.get(kw))
            .or_else(|| kw_args.get(&format!("${kw}")))
    }

    /// str-slice($string, $start-at, $end-at: -1)
    fn str_slice(
        pos_args: &[Value],
        kw_args: &std::collections::HashMap<String, Value>,
    ) -> Result<Value> {
        let (s, q) = match Self::get_str_arg(pos_args, kw_args, 0, "string") {
            Some(Value::String(s, q)) => (s.clone(), *q),
            Some(other) => {
                return Err(SassError::Eval(format!(
                    "$string: {} is not a string.",
                    other
                )));
            }
            None => return Err(SassError::Eval("Missing argument $string.".into())),
        };
        let start = match Self::get_str_arg(pos_args, kw_args, 1, "start-at") {
            Some(Value::Number(n, u)) => {
                if n.fract() != 0.0 {
                    return Err(SassError::Eval(format!("$start-at: {} is not an int.", n)));
                }
                if u.is_some() {
                    return Err(SassError::Eval(format!(
                        "$start-at: Expected {} to have no units.",
                        n
                    )));
                }
                *n as isize
            }
            Some(other) => {
                return Err(SassError::Eval(format!(
                    "$start-at: {} is not a number.",
                    other
                )));
            }
            None => return Err(SassError::Eval("Missing argument $start-at.".into())),
        };
        let end = match Self::get_str_arg(pos_args, kw_args, 2, "end-at") {
            Some(Value::Number(n, u)) => {
                if n.fract() != 0.0 {
                    return Err(SassError::Eval(format!("$end-at: {} is not an int.", n)));
                }
                if u.is_some() {
                    return Err(SassError::Eval(format!(
                        "$end-at: Expected {} to have no units.",
                        n
                    )));
                }
                Some(*n as isize)
            }
            None => None,
            Some(other) => {
                return Err(SassError::Eval(format!(
                    "$end-at: {} is not a number.",
                    other
                )));
            }
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
    fn str_insert(
        pos_args: &[Value],
        kw_args: &std::collections::HashMap<String, Value>,
    ) -> Result<Value> {
        let (s, q) = match Self::get_str_arg(pos_args, kw_args, 0, "string") {
            Some(Value::String(s, q)) => (s.clone(), *q),
            Some(other) => {
                return Err(SassError::Eval(format!(
                    "$string: {} is not a string.",
                    other
                )));
            }
            None => return Err(SassError::Eval("Missing argument $string.".into())),
        };
        let insert = match Self::get_str_arg(pos_args, kw_args, 1, "insert") {
            Some(Value::String(insert, _)) => insert.clone(),
            Some(other) => {
                return Err(SassError::Eval(format!(
                    "$insert: {} is not a string.",
                    other
                )));
            }
            None => return Err(SassError::Eval("Missing argument $insert.".into())),
        };
        let idx = match Self::get_str_arg(pos_args, kw_args, 2, "index") {
            Some(Value::Number(n, u)) => {
                if u.is_some() {
                    return Err(SassError::Eval(format!(
                        "$index: Expected {} to have no units.",
                        n
                    )));
                }
                if n.fract() != 0.0 {
                    return Err(SassError::Eval(format!("$index: {} is not an int.", n)));
                }
                *n as isize
            }
            Some(other) => {
                return Err(SassError::Eval(format!(
                    "$index: {} is not a number.",
                    other
                )));
            }
            None => return Err(SassError::Eval("Missing argument $index.".into())),
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
    fn str_split(
        pos_args: &[Value],
        kw_args: &std::collections::HashMap<String, Value>,
    ) -> Result<Value> {
        let total = pos_args.len() + kw_args.len();
        if total == 0 || (pos_args.is_empty() && !kw_args.contains_key("string") && !kw_args.contains_key("$string")) {
            return Err(SassError::Eval("Missing argument $string.".into()));
        }
        // 检查是否只有 $string 没 $separator
        let has_string = !pos_args.is_empty() || kw_args.contains_key("string") || kw_args.contains_key("$string");
        let has_separator = pos_args.len() > 1 || kw_args.contains_key("separator") || kw_args.contains_key("$separator");
        if has_string && !has_separator {
            return Err(SassError::Eval("Missing argument $separator.".into()));
        }
        let (s, input_quoted) = match Self::get_str_arg(pos_args, kw_args, 0, "string") {
            Some(Value::String(s, q)) => (s.clone(), *q),
            Some(other) => {
                return Err(SassError::Eval(format!(
                    "$string: {} is not a string.",
                    other
                )));
            }
            None => return Err(SassError::Eval("Missing argument $string.".into())),
        };
        let sep = match Self::get_str_arg(pos_args, kw_args, 1, "separator") {
            Some(Value::String(sep, _)) => Some(sep.clone()),
            Some(Value::Null) => None,
            Some(other) => {
                return Err(SassError::Eval(format!(
                    "$separator: {} is not a string.",
                    other
                )));
            }
            None => None,
        };
        let limit = match Self::get_str_arg(pos_args, kw_args, 2, "limit") {
            Some(Value::Number(n, u)) => {
                if u.is_some() {
                    return Err(SassError::Eval(format!(
                        "$limit: Expected {} to have no units.",
                        n
                    )));
                }
                if n.fract() != 0.0 {
                    return Err(SassError::Eval(format!("$limit: {} is not an int.", n)));
                }
                if *n < 1.0 {
                    return Err(SassError::Eval(format!(
                        "$limit: Must be 1 or greater, was {}.",
                        *n as i64
                    )));
                }
                Some(*n as usize)
            }
            Some(Value::Null) | None => None,
            Some(other) => {
                return Err(SassError::Eval(format!(
                    "$limit: {} is not a number.",
                    other
                )));
            }
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
