//! string 内建函数。

use crate::error::{Result, SassError};
use crate::eval::value::Value;
use crate::eval::env::Env;
use crate::parse::ast::Arg;
use crate::eval::eval_value;
use crate::lex::token::QuoteStyle;

pub fn dispatch(field: &str, args: &[Arg], env: &Env) -> Result<Value> {
    let args: Vec<Value> = args.iter().map(|a| eval_value(&a.value, env)).collect();
    match field {
        "length" => match &args[..] {
            [Value::String(s, _)] | [Value::Ident(s)] => Ok(Value::Number(s.chars().count() as f64, None)),
            _ => Err(SassError::eval("str-length() expects a string")),
        },
        "quote" => match &args[..] {
            [v] => {
                let s = v.to_css_string();
                Ok(Value::String(s, QuoteStyle::Double))
            }
            _ => Err(SassError::eval("quote() expects one argument")),
        },
        "unquote" => match &args[..] {
            [Value::String(s, _)] | [Value::Ident(s)] => Ok(Value::String(s.clone(), QuoteStyle::None)),
            [v] => Ok(v.clone()),
            _ => Err(SassError::eval("unquote() expects one argument")),
        },
        "to_upper_case" => match &args[..] {
            [Value::String(s, st)] => Ok(Value::String(s.to_uppercase(), *st)),
            [Value::Ident(s)] => Ok(Value::Ident(s.to_uppercase())),
            _ => Err(SassError::eval("to-upper-case() expects a string")),
        },
        "to_lower_case" => match &args[..] {
            [Value::String(s, st)] => Ok(Value::String(s.to_lowercase(), *st)),
            [Value::Ident(s)] => Ok(Value::Ident(s.to_lowercase())),
            _ => Err(SassError::eval("to-lower-case() expects a string")),
        },
        "index" => match &args[..] {
            [Value::String(s, _), Value::String(sub, _)] => {
                if let Some(pos) = s.find(sub.as_str()) {
                    Ok(Value::Number((pos + 1) as f64, None))
                } else {
                    Ok(Value::Null)
                }
            }
            _ => Err(SassError::eval("str-index() expects two strings")),
        },
        "insert" => match &args[..] {
            [Value::String(s, st), Value::String(insert, _), Value::Number(idx, _)] => {
                // 使用字符迭代器而非字节索引
                let chars: Vec<char> = s.chars().collect();
                let i = (*idx as usize).saturating_sub(1).min(chars.len());
                let mut result: String = chars[..i].iter().collect();
                result.push_str(insert);
                result.extend(chars[i..].iter());
                Ok(Value::String(result, *st))
            }
            _ => Err(SassError::eval("str-insert() expects string, string, number")),
        },
        "slice" => match &args[..] {
            [Value::String(s, st), Value::Number(start, _), Value::Number(end, _)] => {
                let start = if *start >= 0.0 { *start as usize } else { (s.chars().count() as f64 + start) as usize };
                let end = if *end >= 0.0 { *end as usize } else { (s.chars().count() as f64 + end + 1.0) as usize };
                let result: String = s.chars().skip(start.saturating_sub(1)).take(end.saturating_sub(start.saturating_sub(1))).collect();
                Ok(Value::String(result, *st))
            }
            [Value::String(s, st), Value::Number(start, _)] => {
                let start = if *start >= 0.0 { *start as usize } else { (s.chars().count() as f64 + start) as usize };
                let result: String = s.chars().skip(start.saturating_sub(1)).collect();
                Ok(Value::String(result, *st))
            }
            _ => Err(SassError::eval("str-slice() expects string and numbers")),
        },
        "split" => match &args[..] {
            [Value::String(s, st), Value::String(sep, _)] => {
                let parts: Vec<Value> = s.split(sep.as_str())
                    .map(|p| Value::String(p.to_string(), *st))
                    .collect();
                Ok(Value::List(parts, crate::eval::value::Separator::Comma, false))
            }
            _ => Err(SassError::eval("string.split() expects two strings")),
        },
        _ => Err(SassError::eval(format!("Unknown string function: {field}"))),
    }
}
