//! sass:list — list manipulation builtins.

use crate::reactive::Value;
use crate::Error;

fn get_list(v: &Value) -> std::result::Result<&Vec<Value>, Error> {
    match v {
        Value::List(items) => Ok(items),
        _ => Err(Error::msg(format!("list: expected list, got: {v:?}"))),
    }
}

fn get_list_ref(args: &[Value], idx: usize) -> Option<&Vec<Value>> {
    args.get(idx).and_then(|v| get_list(v).ok())
}

/// `list.length($list)` — number of items in the list.
pub fn length(args: &[Value]) -> std::result::Result<Value, Error> {
    let list = get_list_ref(args, 0).map(|v| v.as_slice()).unwrap_or(&[]);
    Ok(Value::Number(list.len() as f64))
}

/// `list.nth($list, $n)` — get the nth item (1-based).
pub fn nth(args: &[Value]) -> std::result::Result<Value, Error> {
    let empty: Vec<Value> = Vec::new();
    let list = get_list_ref(args, 0).unwrap_or(&empty);
    let n = args.get(1).map(|v| match v { Value::Number(n) => *n as usize, _ => 1 }).unwrap_or(1);

    if n == 0 || n > list.len() {
        return Err(Error::msg(format!("list.nth: index {n} out of bounds for list of length {}", list.len())));
    }

    Ok(list[n - 1].clone())
}

/// `list.append($list, $val, $separator)` — append a value to a list.
pub fn append(args: &[Value]) -> std::result::Result<Value, Error> {
    let empty: Vec<Value> = Vec::new();
    let list = get_list_ref(args, 0).unwrap_or(&empty);
    let val = args.get(1).cloned().unwrap_or(Value::Null);

    let mut new_list = list.clone();
    new_list.push(val);
    Ok(Value::List(new_list))
}

/// `list.index($list, $value)` — find index of a value (1-based or None).
pub fn index(args: &[Value]) -> std::result::Result<Value, Error> {
    let empty: Vec<Value> = Vec::new();
    let list = get_list_ref(args, 0).unwrap_or(&empty);
    let target = args.get(1).ok_or_else(|| Error::msg("list.index: missing target value"))?;

    match list.iter().position(|v| v == target) {
        Some(pos) => Ok(Value::Number((pos + 1) as f64)),
        None => Ok(Value::Null),
    }
}

/// `list.join($list, $separator, $bracketed)` — combine list items into a string.
pub fn join(args: &[Value]) -> std::result::Result<Value, Error> {
    let empty: Vec<Value> = Vec::new();
    let list = get_list_ref(args, 0).unwrap_or(&empty);
    let sep = args.get(1).map(|v| match v { Value::String(s) => s.as_str(), _ => "," }).unwrap_or(",");

    let parts: Vec<String> = list.iter().map(|v| match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => format!("{n}"),
        Value::Null => String::new(),
        Value::List(_) | Value::Map(_) => format!("{v:?}"),
    }).collect();

    Ok(Value::String(parts.join(sep)))
}
