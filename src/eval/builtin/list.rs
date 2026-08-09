//! sass:list 内建函数。

use crate::error::{Result, SassError};
use crate::parse::ast::{Separator, Value};

/// 断言列表参数。
fn assert_list(arg: &Value) -> Result<&Vec<Value>> {
    match arg {
        Value::List(items, _) => Ok(items),
        _ => Err(SassError::TypeError {
            expected: "list".to_string(),
            actual: format!("{arg}"),
        }),
    }
}

fn first_arg(args: &[Value]) -> Result<&Value> {
    args.first()
        .ok_or_else(|| SassError::EvalError("函数需要至少 1 个参数".to_string()))
}

pub fn length(args: &[Value]) -> Result<Value> {
    let list = assert_list(first_arg(args)?)?;
    Ok(Value::Number(list.len() as f64, None))
}

pub fn nth(args: &[Value]) -> Result<Value> {
    let list = assert_list(
        args.first()
            .ok_or_else(|| SassError::EvalError("nth 需要 2 个参数".to_string()))?,
    )?;
    let n = assert_number(
        args.get(1)
            .ok_or_else(|| SassError::EvalError("nth 需要索引".to_string()))?,
    )?;
    let idx = n.0 as usize;
    if idx == 0 || idx > list.len() {
        return Err(SassError::EvalError(format!("索引 {idx} 超出范围")));
    }
    Ok(list[idx - 1].clone())
}

pub fn append(args: &[Value]) -> Result<Value> {
    let list = assert_list(
        args.first()
            .ok_or_else(|| SassError::EvalError("append 需要 2-3 个参数".to_string()))?,
    )?;
    let value = args
        .get(1)
        .ok_or_else(|| SassError::EvalError("append 需要追加值".to_string()))?;
    let sep = if args.len() >= 3 {
        match args.get(2).unwrap() {
            Value::String(s, false) if s == "comma" => Separator::Comma,
            Value::String(s, false) if s == "space" => Separator::Space,
            Value::String(s, false) if s == "slash" => Separator::Slash,
            _ => Separator::Comma,
        }
    } else {
        Separator::Space
    };
    let mut new_list = list.clone();
    new_list.push(value.clone());
    Ok(Value::List(new_list, sep))
}

pub fn join(args: &[Value]) -> Result<Value> {
    let list1 = assert_list(
        args.first()
            .ok_or_else(|| SassError::EvalError("join 需要 2-3 个参数".to_string()))?,
    )?;
    let list2 = assert_list(
        args.get(1)
            .ok_or_else(|| SassError::EvalError("join 需要第二个列表".to_string()))?,
    )?;
    let sep = if args.len() >= 3 {
        match args.get(2).unwrap() {
            Value::String(s, false) if s == "comma" => Separator::Comma,
            Value::String(s, false) if s == "space" => Separator::Space,
            Value::String(s, false) if s == "slash" => Separator::Slash,
            _ => Separator::Comma,
        }
    } else {
        Separator::Space
    };
    let mut new_list = list1.clone();
    new_list.extend(list2.iter().cloned());
    Ok(Value::List(new_list, sep))
}

pub fn index(args: &[Value]) -> Result<Value> {
    let list = assert_list(
        args.first()
            .ok_or_else(|| SassError::EvalError("index 需要 2 个参数".to_string()))?,
    )?;
    let value = args
        .get(1)
        .ok_or_else(|| SassError::EvalError("index 需要查找值".to_string()))?;
    match list.iter().position(|v| v == value) {
        Some(idx) => Ok(Value::Number((idx + 1) as f64, None)),
        None => Ok(Value::String("null".to_string(), false)),
    }
}

pub fn separator(args: &[Value]) -> Result<Value> {
    let list = first_arg(args)?;
    match list {
        Value::List(_, Separator::Comma) => Ok(Value::String("comma".to_string(), false)),
        Value::List(_, Separator::Space) => Ok(Value::String("space".to_string(), false)),
        Value::List(_, Separator::Slash) => Ok(Value::String("slash".to_string(), false)),
        _ => Ok(Value::String("space".to_string(), false)),
    }
}

pub fn set_nth(args: &[Value]) -> Result<Value> {
    let list = assert_list(
        args.first()
            .ok_or_else(|| SassError::EvalError("set-nth 需要 3 个参数".to_string()))?,
    )?;
    let n = assert_number(
        args.get(1)
            .ok_or_else(|| SassError::EvalError("set-nth 需要索引".to_string()))?,
    )?;
    let value = args
        .get(2)
        .ok_or_else(|| SassError::EvalError("set-nth 需要新值".to_string()))?;
    let idx = n.0 as usize;
    if idx == 0 || idx > list.len() {
        return Err(SassError::EvalError(format!("索引 {idx} 超出范围")));
    }
    let mut new_list = list.clone();
    new_list[idx - 1] = value.clone();
    Ok(Value::List(new_list, Separator::Space))
}

pub fn sl_separator(args: &[Value]) -> Result<Value> {
    let _ = args; // sass-list 的 separator 函数别名
    Ok(Value::String("space".to_string(), false))
}

/// 断言数值参数（内部使用）。
fn assert_number(arg: &Value) -> Result<(f64, Option<String>)> {
    match arg {
        Value::Number(n, unit) => Ok((*n, unit.clone())),
        _ => Err(SassError::TypeError {
            expected: "number".to_string(),
            actual: "other".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length() {
        let list = Value::List(
            vec![
                Value::Number(1.0, None),
                Value::Number(2.0, None),
                Value::Number(3.0, None),
            ],
            Separator::Comma,
        );
        assert_eq!(length(&[list]).unwrap(), Value::Number(3.0, None));
    }

    #[test]
    fn test_nth() {
        let list = Value::List(
            vec![
                Value::Number(1.0, None),
                Value::Number(2.0, None),
                Value::Number(3.0, None),
            ],
            Separator::Comma,
        );
        assert_eq!(
            nth(&[list.clone(), Value::Number(2.0, None)]).unwrap(),
            Value::Number(2.0, None)
        );
    }

    #[test]
    fn test_append() {
        let list = Value::List(
            vec![Value::Number(1.0, None), Value::Number(2.0, None)],
            Separator::Comma,
        );
        let result = append(&[list, Value::Number(3.0, None)]).unwrap();
        match result {
            Value::List(items, _) => assert_eq!(items.len(), 3),
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_index() {
        let list = Value::List(
            vec![
                Value::Number(1.0, None),
                Value::Number(2.0, None),
                Value::Number(3.0, None),
            ],
            Separator::Comma,
        );
        assert_eq!(
            index(&[list.clone(), Value::Number(2.0, None)]).unwrap(),
            Value::Number(2.0, None)
        );
        assert_eq!(
            index(&[list, Value::Number(99.0, None)]).unwrap(),
            Value::String("null".to_string(), false)
        );
    }
}
