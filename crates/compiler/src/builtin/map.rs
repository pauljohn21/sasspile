//! sass:map — map (key-value) manipulation builtins.

use crate::reactive::Value;
use crate::Error;

fn get_map(v: &Value) -> std::result::Result<&Vec<(String, Value)>, Error> {
    match v {
        Value::Map(entries) => Ok(entries),
        _ => Err(Error::msg(format!("map: expected map, got: {v:?}"))),
    }
}

fn get_map_ref(args: &[Value], idx: usize) -> Option<&Vec<(String, Value)>> {
    args.get(idx).and_then(|v| get_map(v).ok())
}

fn get_key(v: &Value) -> std::result::Result<&str, Error> {
    match v {
        Value::String(s) => Ok(s),
        _ => Err(Error::msg(format!("map: expected string key, got: {v:?}"))),
    }
}

/// `map.get($map, $key)` — retrieve a value by key.
pub fn get(args: &[Value]) -> std::result::Result<Value, Error> {
    let empty: Vec<(String, Value)> = Vec::new();
    let map = get_map_ref(args, 0).unwrap_or(&empty);
    let key = args.get(1).map(get_key).transpose()?.unwrap_or("");

    match map.iter().find(|(k, _)| k == key) {
        Some((_, v)) => Ok(v.clone()),
        None => Ok(Value::Null),
    }
}

/// `map.has-key($map, $key)` — check if key exists.
pub fn has_key(args: &[Value]) -> std::result::Result<Value, Error> {
    let empty: Vec<(String, Value)> = Vec::new();
    let map = get_map_ref(args, 0).unwrap_or(&empty);
    let key = args.get(1).map(get_key).transpose()?.unwrap_or("");

    Ok(Value::Number(if map.iter().any(|(k, _)| k == key) { 1.0 } else { 0.0 }))
}

/// `map.keys($map)` — return list of all keys.
pub fn keys(args: &[Value]) -> std::result::Result<Value, Error> {
    let empty: Vec<(String, Value)> = Vec::new();
    let map = get_map_ref(args, 0).unwrap_or(&empty);
    Ok(Value::List(
        map.iter().map(|(k, _)| Value::String(k.clone())).collect(),
    ))
}

/// `map.values($map)` — return list of all values.
pub fn values(args: &[Value]) -> std::result::Result<Value, Error> {
    let empty: Vec<(String, Value)> = Vec::new();
    let map = get_map_ref(args, 0).unwrap_or(&empty);
    Ok(Value::List(
        map.iter().map(|(_, v)| v.clone()).collect(),
    ))
}

/// `map.merge($map1, $map2)` — merge two maps (second overwrites first on conflict).
pub fn merge(args: &[Value]) -> std::result::Result<Value, Error> {
    let empty: Vec<(String, Value)> = Vec::new();
    let map1 = get_map_ref(args, 0).unwrap_or(&empty);
    let map2 = get_map_ref(args, 1).unwrap_or(&empty);

    let mut result: Vec<(String, Value)> = map1.clone();
    for (k, v) in map2.iter() {
        if let Some(existing) = result.iter_mut().find(|(ek, _)| ek == k) {
            existing.1 = v.clone();
        } else {
            result.push((k.clone(), v.clone()));
        }
    }
    Ok(Value::Map(result))
}

/// `map.remove($map, $key)` — remove an entry by key.
pub fn remove(args: &[Value]) -> std::result::Result<Value, Error> {
    let empty: Vec<(String, Value)> = Vec::new();
    let map = get_map_ref(args, 0).unwrap_or(&empty);
    let key = args.get(1).map(get_key).transpose()?.unwrap_or("");

    Ok(Value::Map(
        map.iter().filter(|(k, _)| k != key).cloned().collect(),
    ))
}
