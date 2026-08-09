//! sass:string 内建函数。

use crate::error::{Result, SassError};
use crate::parse::ast::Value;

/// 获取第一个字符串参数。
fn assert_string(arg: &Value) -> Result<&str> {
    match arg {
        Value::String(s, _) => Ok(s),
        _ => Err(SassError::TypeError {
            expected: "string".to_string(),
            actual: format!("{arg}"),
        }),
    }
}

/// 断言数值参数。
fn assert_number(arg: &Value) -> Result<(f64, Option<String>)> {
    match arg {
        Value::Number(n, unit) => Ok((*n, unit.clone())),
        _ => Err(SassError::TypeError {
            expected: "number".to_string(),
            actual: "other".to_string(),
        }),
    }
}

fn first_arg(args: &[Value]) -> Result<&Value> {
    args.first()
        .ok_or_else(|| SassError::EvalError("函数需要至少 1 个参数".to_string()))
}

pub fn length(args: &[Value]) -> Result<Value> {
    let s = assert_string(first_arg(args)?)?;
    Ok(Value::Number(s.chars().count() as f64, None))
}

pub fn index(args: &[Value]) -> Result<Value> {
    let s = assert_string(
        args.first()
            .ok_or_else(|| SassError::EvalError("index 需要 2 个参数".to_string()))?,
    )?;
    let sub = assert_string(
        args.get(1)
            .ok_or_else(|| SassError::EvalError("index 需要 2 个参数".to_string()))?,
    )?;
    match s.find(sub) {
        Some(idx) => Ok(Value::Number((s[..idx].chars().count() + 1) as f64, None)),
        None => Ok(Value::String("null".to_string(), false)),
    }
}

pub fn slice(args: &[Value]) -> Result<Value> {
    let s = assert_string(
        args.first()
            .ok_or_else(|| SassError::EvalError("slice 需要 2-3 个参数".to_string()))?,
    )?;
    let start_at = assert_number(
        args.get(1)
            .ok_or_else(|| SassError::EvalError("slice 需要起始位置".to_string()))?,
    )?;
    let start = (start_at.0 as isize - 1).max(0) as usize;
    let chars: Vec<char> = s.chars().collect();
    if start >= chars.len() {
        return Ok(Value::String(String::new(), true));
    }
    let end_at = if args.len() >= 3 {
        assert_number(args.get(2).unwrap())?.0 as usize
    } else {
        chars.len()
    };
    let end = end_at.min(chars.len());
    let result: String = chars[start..end].iter().collect();
    Ok(Value::String(result, true))
}

pub fn to_upper_case(args: &[Value]) -> Result<Value> {
    let s = assert_string(first_arg(args)?)?;
    Ok(Value::String(s.to_uppercase(), true))
}

pub fn to_lower_case(args: &[Value]) -> Result<Value> {
    let s = assert_string(first_arg(args)?)?;
    Ok(Value::String(s.to_lowercase(), true))
}

pub fn insert(args: &[Value]) -> Result<Value> {
    let s = assert_string(
        args.first()
            .ok_or_else(|| SassError::EvalError("insert 需要 3 个参数".to_string()))?,
    )?;
    let insert_str = assert_string(
        args.get(1)
            .ok_or_else(|| SassError::EvalError("insert 需要插入字符串".to_string()))?,
    )?;
    let index = assert_number(
        args.get(2)
            .ok_or_else(|| SassError::EvalError("insert 需要位置".to_string()))?,
    )?;
    let idx = (index.0 as isize - 1).max(0) as usize;
    let chars: Vec<char> = s.chars().collect();
    if idx >= chars.len() {
        return Ok(Value::String(format!("{s}{insert_str}"), true));
    }
    let mut result = String::new();
    result.extend(&chars[..idx]);
    result.push_str(insert_str);
    result.extend(&chars[idx..]);
    Ok(Value::String(result, true))
}

pub fn unique_id(args: &[Value]) -> Result<Value> {
    let _ = args; // 无参数
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    Ok(Value::String(format!("u{id:08x}"), false))
}

pub fn quote(args: &[Value]) -> Result<Value> {
    let s = assert_string(first_arg(args)?)?;
    Ok(Value::String(format!("\"{s}\""), false))
}

pub fn unquote(args: &[Value]) -> Result<Value> {
    let s = assert_string(first_arg(args)?)?;
    let trimmed = s.trim_matches('"').trim_matches('\'');
    Ok(Value::String(trimmed.to_string(), false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length() {
        assert_eq!(
            length(&[Value::String("hello".to_string(), false)]).unwrap(),
            Value::Number(5.0, None)
        );
    }

    #[test]
    fn test_to_upper_case() {
        assert_eq!(
            to_upper_case(&[Value::String("hello".to_string(), false)]).unwrap(),
            Value::String("HELLO".to_string(), true)
        );
    }

    #[test]
    fn test_to_lower_case() {
        assert_eq!(
            to_lower_case(&[Value::String("HELLO".to_string(), false)]).unwrap(),
            Value::String("hello".to_string(), true)
        );
    }

    #[test]
    fn test_slice() {
        assert_eq!(
            slice(&[
                Value::String("hello world".to_string(), false),
                Value::Number(1.0, None),
                Value::Number(5.0, None)
            ])
            .unwrap(),
            Value::String("hello".to_string(), true)
        );
    }

    #[test]
    fn test_quote() {
        assert_eq!(
            quote(&[Value::String("hello".to_string(), false)]).unwrap(),
            Value::String("\"hello\"".to_string(), false)
        );
    }

    #[test]
    fn test_unquote() {
        assert_eq!(
            unquote(&[Value::String("\"hello\"".to_string(), false)]).unwrap(),
            Value::String("hello".to_string(), false)
        );
    }
}
