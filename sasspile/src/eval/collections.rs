//! List and Map access operations.

use crate::eval::error::EvalError;
use crate::value::{Number, Value};

/// Access the nth element of a list (1-indexed, Sass semantics).
pub fn nth(list: &Value, n: &Value) -> Result<Value, EvalError> {
    let n_int = match n {
        Value::Number(num) => {
            if num.value <= 0.0 {
                return Err(EvalError::ListIndexOutOfBounds(num.value as usize, list_len(list)));
            }
            num.value as usize
        }
        _ => return Err(EvalError::type_error("number", n.type_name())),
    };

    match list {
        Value::List(items, _) => {
            if n_int == 0 || n_int > items.len() {
                return Err(EvalError::ListIndexOutOfBounds(n_int, items.len()));
            }
            Ok(items[n_int - 1].clone())
        }
        Value::Map(entries) => {
            if n_int == 0 || n_int > entries.len() {
                return Err(EvalError::ListIndexOutOfBounds(n_int, entries.len()));
            }
            let (k, v) = &entries[n_int - 1];
            // Return a 2-element list (key, value).
            Ok(Value::List(
                vec![k.clone(), v.clone()],
                crate::value::Separator::Space,
            ))
        }
        Value::String(s, q) => {
            let chars: Vec<char> = s.chars().collect();
            if n_int == 0 || n_int > chars.len() {
                return Err(EvalError::ListIndexOutOfBounds(n_int, chars.len()));
            }
            Ok(Value::String(chars[n_int - 1].to_string(), *q))
        }
        // Single value as a 1-element list.
        other => {
            if n_int == 1 {
                Ok(other.clone())
            } else {
                Err(EvalError::ListIndexOutOfBounds(n_int, 1))
            }
        }
    }
}

/// Get a value from a map by key.
pub fn map_get(map: &Value, key: &Value) -> Result<Value, EvalError> {
    match map {
        Value::Map(entries) => {
            for (k, v) in entries {
                if k == key {
                    return Ok(v.clone());
                }
            }
            Err(EvalError::MapKeyNotFound(key.to_string_value()))
        }
        _ => Err(EvalError::type_error("map", map.type_name())),
    }
}

/// Check if a map contains a key.
pub fn map_has(map: &Value, key: &Value) -> Result<Value, EvalError> {
    match map {
        Value::Map(entries) => {
            let found = entries.iter().any(|(k, _)| k == key);
            Ok(Value::Boolean(found))
        }
        _ => Err(EvalError::type_error("map", map.type_name())),
    }
}

/// Get the length of a value (list, map, or string).
pub fn len(val: &Value) -> Value {
    match val {
        Value::List(items, _) => Value::Number(Number::unitless(items.len() as f64)),
        Value::Map(entries) => Value::Number(Number::unitless(entries.len() as f64)),
        Value::String(s, _) => Value::Number(Number::unitless(s.chars().count() as f64)),
        _ => Value::Number(Number::unitless(1.0)),
    }
}

/// Access helper: get length as usize.
fn list_len(val: &Value) -> usize {
    match val {
        Value::List(items, _) => items.len(),
        Value::Map(entries) => entries.len(),
        Value::String(s, _) => s.chars().count(),
        _ => 1,
    }
}
