//! sass:string — string manipulation builtins.

use crate::reactive::Value;
use crate::Error;

fn get_string(v: &Value) -> std::result::Result<&str, Error> {
    match v {
        Value::String(s) => Ok(s),
        _ => Err(Error::msg(format!("string: expected string, got: {v:?}"))),
    }
}

/// `string.index($string, $substring)` — find index of substring (1-based or None).
/// Returns a Value::Number (1-based index) or Value::Null if not found.
pub fn index(args: &[Value]) -> std::result::Result<Value, Error> {
    let s = args.first().map(get_string).transpose()?.unwrap_or("");
    let sub = args.get(1).map(get_string).transpose()?.unwrap_or("");

    if sub.is_empty() {
        return Ok(Value::Null);
    }

    match s.find(sub) {
        Some(pos) => Ok(Value::Number((pos + 1) as f64)), // 1-based like Sass
        None => Ok(Value::Null),
    }
}

/// `string.length($string)` — number of characters.
pub fn length(args: &[Value]) -> std::result::Result<Value, Error> {
    let s = args.first().map(get_string).transpose()?.unwrap_or("");
    Ok(Value::Number(s.chars().count() as f64))
}

/// `string.slice($string, $start, $end)` — extract a substring (1-based, inclusive).
///
/// Sass semantics: both endpoints are 1-based and INCLUSIVE.
/// `string.slice("hello", 2, 4)` returns `"ell"`.
pub fn slice(args: &[Value]) -> std::result::Result<Value, Error> {
    let s: &str = args.first().map(get_string).transpose()?.unwrap_or("");
    let start =
        args.get(1).map(|v| match v { Value::Number(n) => *n as usize, _ => 1 }).unwrap_or(1);
    let end = args
        .get(2)
        .map(|v| match v { Value::Number(n) => *n as usize, _ => s.chars().count() })
        .unwrap_or_else(|| s.chars().count());

    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    if len == 0 || start == 0 {
        return Ok(Value::String(String::new()));
    }

    // Sass slice is 1-based, inclusive on both ends.
    // start=2, end=4 on "hello" → indices 1..=3 (0-based) → "ell"
    // Conversion: 0-based inclusive end = end - 1
    let start_idx = (start - 1).min(len - 1);
    let inclusive_end = (end.saturating_sub(1)).min(len - 1);

    if start_idx > inclusive_end {
        return Ok(Value::String(String::new()));
    }

    Ok(Value::String(chars[start_idx..=inclusive_end].iter().collect()))
}

/// `string.to-upper-case($string)` — uppercase.
pub fn to_upper_case(args: &[Value]) -> std::result::Result<Value, Error> {
    let s = args.first().map(get_string).transpose()?.unwrap_or("");
    Ok(Value::String(s.to_uppercase()))
}

/// `string.to-lower-case($string)` — lowercase.
pub fn to_lower_case(args: &[Value]) -> std::result::Result<Value, Error> {
    let s = args.first().map(get_string).transpose()?.unwrap_or("");
    Ok(Value::String(s.to_lowercase()))
}
