//! list 内建函数。

use crate::error::{Result, SassError};
use crate::eval::value::{Value, Separator};
use crate::eval::env::Env;
use crate::parse::ast::Arg;
use crate::eval::eval_value;

pub fn dispatch(field: &str, args: &[Arg], env: &Env) -> Result<Value> {
    let args: Vec<Value> = args.iter().map(|a| eval_value(&a.value, env)).collect();
    match field {
        "length" => match &args[..] {
            [v] => {
                let len = match v {
                    Value::List(items, _, _) => items.len(),
                    Value::ArgList(items) => items.len(),
                    Value::Map(pairs) => pairs.len(),
                    Value::Null => 0,
                    _ => 1,
                };
                Ok(Value::Number(len as f64, None))
            }
            _ => Err(SassError::eval("length() expects one argument")),
        },
        "nth" => match &args[..] {
            [v, Value::Number(n, _)] => {
                let items = match v {
                    Value::List(items, _, _) => items.clone(),
                    Value::ArgList(items) => items.clone(),
                    other => vec![other.clone()],
                };
                let idx = if *n > 0.0 { *n as usize } else { (items.len() as f64 + n + 1.0) as usize };
                items.get(idx.saturating_sub(1))
                    .cloned()
                    .ok_or_else(|| SassError::eval(format!("list index {n} out of range")))
            }
            _ => Err(SassError::eval("nth() expects a list and index")),
        },
        "join" => match &args[..] {
            [a, b, sep @ ..] => {
                let mut items = list_items(a);
                items.extend(list_items(b));
                let separator = match sep.first() {
                    Some(Value::String(s, _)) | Some(Value::Ident(s)) if s == "comma" => Separator::Comma,
                    Some(Value::String(s, _)) | Some(Value::Ident(s)) if s == "space" => Separator::Space,
                    Some(Value::String(s, _)) | Some(Value::Ident(s)) if s == "slash" => Separator::Slash,
                    _ => Separator::Space,
                };
                Ok(Value::List(items, separator, false))
            }
            _ => Err(SassError::eval("join() expects two lists")),
        },
        "append" => match &args[..] {
            [list, val, sep @ ..] => {
                let mut items = list_items(list);
                items.push(val.clone());
                let separator = match sep.first() {
                    Some(Value::String(s, _)) | Some(Value::Ident(s)) if s == "comma" => Separator::Comma,
                    Some(Value::String(s, _)) | Some(Value::Ident(s)) if s == "slash" => Separator::Slash,
                    _ => Separator::Space,
                };
                Ok(Value::List(items, separator, false))
            }
            _ => Err(SassError::eval("append() expects a list and a value")),
        },
        "index" => match &args[..] {
            [list, val] => {
                let items = list_items(list);
                for (i, item) in items.iter().enumerate() {
                    if item.equals(val) {
                        return Ok(Value::Number((i + 1) as f64, None));
                    }
                }
                Ok(Value::Null)
            }
            _ => Err(SassError::eval("index() expects a list and value")),
        },
        "is_bracketed" => match &args[..] {
            [Value::List(_, _, bracketed)] => Ok(Value::Bool(*bracketed)),
            _ => Ok(Value::Bool(false)),
        },
        "separator" => match &args[..] {
            [Value::List(_, sep, _)] => Ok(match sep {
                Separator::Comma => Value::String("comma".to_string(), crate::lex::token::QuoteStyle::None),
                Separator::Space => Value::String("space".to_string(), crate::lex::token::QuoteStyle::None),
                Separator::Slash => Value::String("slash".to_string(), crate::lex::token::QuoteStyle::None),
                Separator::Undecided => Value::String("space".to_string(), crate::lex::token::QuoteStyle::None),
            }),
            _ => Ok(Value::String("space".to_string(), crate::lex::token::QuoteStyle::None)),
        },
        "set_nth" => match &args[..] {
            [list, Value::Number(n, _), val] => {
                let mut items = list_items(list);
                let idx = (*n as usize).saturating_sub(1).min(items.len().saturating_sub(1));
                if idx < items.len() {
                    items[idx] = val.clone();
                }
                Ok(Value::List(items, Separator::Space, false))
            }
            _ => Err(SassError::eval("set-nth() expects a list, index, and value")),
        },
        "zip" => {
            let lists: Vec<Vec<Value>> = args.iter().map(list_items).collect();
            if lists.is_empty() {
                return Ok(Value::List(Vec::new(), Separator::Comma, false));
            }
            let min_len = lists.iter().map(|l| l.len()).min().unwrap_or(0);
            let result: Vec<Value> = (0..min_len).map(|i| {
                let items: Vec<Value> = lists.iter().map(|l| l[i].clone()).collect();
                Value::List(items, Separator::Space, false)
            }).collect();
            Ok(Value::List(result, Separator::Comma, false))
        }
        _ => Err(SassError::eval(format!("Unknown list function: {field}"))),
    }
}

fn list_items(v: &Value) -> Vec<Value> {
    match v {
        Value::List(items, _, _) => items.clone(),
        Value::ArgList(items) => items.clone(),
        Value::Null => Vec::new(),
        other => vec![other.clone()],
    }
}
