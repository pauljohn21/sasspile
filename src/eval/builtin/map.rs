//! sass:map 内建函数。

use crate::error::{Result, SassError};
use crate::parse::ast::{Separator, Value};

/// 断言 map 参数——内部表示为 Value::List 的键值对。
fn assert_map(arg: &Value) -> Result<&Vec<Value>> {
    match arg {
        Value::List(items, _) => {
            if items.len() % 2 != 0 {
                return Err(SassError::TypeError {
                    expected: "map (偶数个元素)".to_string(),
                    actual: "odd-length list".to_string(),
                });
            }
            Ok(items)
        }
        _ => Err(SassError::TypeError {
            expected: "map".to_string(),
            actual: format!("{arg}"),
        }),
    }
}

fn first_arg(args: &[Value]) -> Result<&Value> {
    args.first()
        .ok_or_else(|| SassError::EvalError("函数需要至少 1 个参数".to_string()))
}

/// 从列表构建 map（键值对）。
#[allow(dead_code)]
pub fn build_map(items: Vec<Value>) -> Value {
    Value::List(items, Separator::Comma)
}

pub fn get(args: &[Value]) -> Result<Value> {
    let map = assert_map(
        args.first()
            .ok_or_else(|| SassError::EvalError("map-get 需要 2 个参数".to_string()))?,
    )?;
    let key = args
        .get(1)
        .ok_or_else(|| SassError::EvalError("map-get 需要键".to_string()))?;
    for chunk in map.chunks(2) {
        if chunk[0] == *key {
            return Ok(chunk[1].clone());
        }
    }
    Ok(Value::String("null".to_string(), false))
}

pub fn keys(args: &[Value]) -> Result<Value> {
    let map = assert_map(first_arg(args)?)?;
    let keys: Vec<Value> = map.iter().step_by(2).cloned().collect();
    Ok(Value::List(keys, Separator::Comma))
}

pub fn values(args: &[Value]) -> Result<Value> {
    let map = assert_map(first_arg(args)?)?;
    let values: Vec<Value> = map.iter().skip(1).step_by(2).cloned().collect();
    Ok(Value::List(values, Separator::Comma))
}

pub fn has_key(args: &[Value]) -> Result<Value> {
    let map = assert_map(
        args.first()
            .ok_or_else(|| SassError::EvalError("map-has-key 需要 2 个参数".to_string()))?,
    )?;
    let key = args
        .get(1)
        .ok_or_else(|| SassError::EvalError("map-has-key 需要键".to_string()))?;
    Ok(Value::Bool(map.chunks(2).any(|chunk| chunk[0] == *key)))
}

pub fn merge(args: &[Value]) -> Result<Value> {
    let map1 = assert_map(
        args.first()
            .ok_or_else(|| SassError::EvalError("map-merge 需要 2 个参数".to_string()))?,
    )?;
    let map2 = assert_map(
        args.get(1)
            .ok_or_else(|| SassError::EvalError("map-merge 需要第二个 map".to_string()))?,
    )?;
    let mut result = map1.clone();
    // 合并：map2 的键覆盖 map1
    for chunk in map2.chunks(2) {
        let key = &chunk[0];
        let val = &chunk[1];
        if let Some(pos) = result.chunks(2).position(|c| c[0] == *key) {
            result[pos * 2 + 1] = val.clone();
        } else {
            result.push(key.clone());
            result.push(val.clone());
        }
    }
    Ok(Value::List(result, Separator::Comma))
}

pub fn remove(args: &[Value]) -> Result<Value> {
    let map = assert_map(first_arg(args)?)?;
    let keys_to_remove: Vec<&Value> = args.iter().skip(1).collect();
    let mut result = Vec::new();
    for chunk in map.chunks(2) {
        if !keys_to_remove.contains(&&chunk[0]) {
            result.push(chunk[0].clone());
            result.push(chunk[1].clone());
        }
    }
    Ok(Value::List(result, Separator::Comma))
}

pub fn deep_get(args: &[Value]) -> Result<Value> {
    let map = assert_map(
        args.first()
            .ok_or_else(|| SassError::EvalError("map-deep-get 需要 map + 多个键".to_string()))?,
    )?;
    let keys = &args[1..];
    if keys.is_empty() {
        return Ok(Value::String("null".to_string(), false));
    }
    let key = &keys[0];
    for chunk in map.chunks(2) {
        if chunk[0] == *key {
            if keys.len() == 1 {
                return Ok(chunk[1].clone());
            }
            // 递归查找
            let mut next_args = vec![chunk[1].clone()];
            next_args.extend_from_slice(&keys[1..]);
            return deep_get(&next_args);
        }
    }
    Ok(Value::String("null".to_string(), false))
}

pub fn deep_merge(args: &[Value]) -> Result<Value> {
    let map1 = assert_map(
        args.first()
            .ok_or_else(|| SassError::EvalError("map-deep-merge 需要 2 个参数".to_string()))?,
    )?;
    let map2 = assert_map(
        args.get(1)
            .ok_or_else(|| SassError::EvalError("map-deep-merge 需要第二个 map".to_string()))?,
    )?;
    let mut result = map1.clone();
    for chunk in map2.chunks(2) {
        let key = &chunk[0];
        let val = &chunk[1];
        if let Some(pos) = result.chunks(2).position(|c| c[0] == *key) {
            // 如果两边都是 map，递归合并
            if let (Value::List(_, _), Value::List(_, _)) = (&result[pos * 2 + 1], val) {
                let sub_merge = deep_merge(&[result[pos * 2 + 1].clone(), val.clone()])?;
                result[pos * 2 + 1] = sub_merge;
            } else {
                result[pos * 2 + 1] = val.clone();
            }
        } else {
            result.push(key.clone());
            result.push(val.clone());
        }
    }
    Ok(Value::List(result, Separator::Comma))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_map() -> Value {
        build_map(vec![
            Value::String("color".to_string(), false),
            Value::String("red".to_string(), true),
            Value::String("size".to_string(), false),
            Value::Number(16.0, Some("px".to_string())),
        ])
    }

    #[test]
    fn test_get() {
        let map = sample_map();
        assert_eq!(
            get(&[map, Value::String("color".to_string(), false)]).unwrap(),
            Value::String("red".to_string(), true)
        );
    }

    #[test]
    fn test_keys() {
        let map = sample_map();
        let result = keys(&[map]).unwrap();
        match result {
            Value::List(items, _) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], Value::String("color".to_string(), false));
                assert_eq!(items[1], Value::String("size".to_string(), false));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_values() {
        let map = sample_map();
        let result = values(&[map]).unwrap();
        match result {
            Value::List(items, _) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], Value::String("red".to_string(), true));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_has_key() {
        let map = sample_map();
        assert_eq!(
            has_key(&[map.clone(), Value::String("color".to_string(), false)]).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            has_key(&[map, Value::String("unknown".to_string(), false)]).unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn test_merge() {
        let map1 = build_map(vec![
            Value::String("a".to_string(), false),
            Value::Number(1.0, None),
        ]);
        let map2 = build_map(vec![
            Value::String("b".to_string(), false),
            Value::Number(2.0, None),
        ]);
        let result = merge(&[map1, map2]).unwrap();
        match result {
            Value::List(items, _) => assert_eq!(items.len(), 4),
            _ => panic!("Expected List"),
        }
    }
}
