//! map 内建函数。

use crate::error::{Result, SassError};
use crate::eval::value::Value;
use crate::eval::env::Env;
use crate::parse::ast::Arg;
use crate::eval::eval_value;

pub fn dispatch(field: &str, args: &[Arg], env: &Env) -> Result<Value> {
    let args: Vec<Value> = args.iter().map(|a| eval_value(&a.value, env)).collect();
    match field {
        "get" => match &args[..] {
            [Value::Map(pairs), key] => {
                for (k, v) in pairs {
                    if k.equals(key) {
                        return Ok(v.clone());
                    }
                }
                Ok(Value::Null)
            }
            _ => Err(SassError::eval("map-get() expects a map and key")),
        },
        "merge" => match &args[..] {
            [Value::Map(a), Value::Map(b)] => {
                let mut merged = a.clone();
                for (k, v) in b {
                    // 覆盖或插入
                    if let Some(pos) = merged.iter().position(|(ek, _)| ek.equals(k)) {
                        merged[pos].1 = v.clone();
                    } else {
                        merged.push((k.clone(), v.clone()));
                    }
                }
                Ok(Value::Map(merged))
            }
            _ => Err(SassError::eval("map-merge() expects two maps")),
        },
        "remove" => match &args[..] {
            [Value::Map(pairs), keys @ ..] => {
                let result: Vec<(Value, Value)> = pairs.iter()
                    .filter(|(k, _)| !keys.iter().any(|kk| kk.equals(k)))
                    .cloned()
                    .collect();
                Ok(Value::Map(result))
            }
            _ => Err(SassError::eval("map-remove() expects a map and keys")),
        },
        "keys" => match &args[..] {
            [Value::Map(pairs)] => {
                Ok(Value::List(pairs.iter().map(|(k, _)| k.clone()).collect(), crate::eval::value::Separator::Comma, false))
            }
            _ => Err(SassError::eval("map-keys() expects a map")),
        },
        "values" => match &args[..] {
            [Value::Map(pairs)] => {
                Ok(Value::List(pairs.iter().map(|(_, v)| v.clone()).collect(), crate::eval::value::Separator::Comma, false))
            }
            _ => Err(SassError::eval("map-values() expects a map")),
        },
        "has_key" => match &args[..] {
            [Value::Map(pairs), key] => {
                Ok(Value::Bool(pairs.iter().any(|(k, _)| k.equals(key))))
            }
            _ => Err(SassError::eval("map-has-key() expects a map and key")),
        },
        "deep_merge" => match &args[..] {
            [Value::Map(a), Value::Map(b)] => {
                let mut merged = a.clone();
                for (k, v) in b {
                    if let Some(pos) = merged.iter().position(|(ek, _)| ek.equals(k)) {
                        // 递归合并嵌套 Map
                        if let (Value::Map(merged_child), Value::Map(b_child)) = (&merged[pos].1, v) {
                            let mut result = merged_child.clone();
                            for (bk, bv) in b_child {
                                if let Some(p2) = result.iter().position(|(ek2, _)| ek2.equals(bk)) {
                                    result[p2].1 = bv.clone();
                                } else {
                                    result.push((bk.clone(), bv.clone()));
                                }
                            }
                            merged[pos].1 = Value::Map(result);
                        } else {
                            merged[pos].1 = v.clone();
                        }
                    } else {
                        merged.push((k.clone(), v.clone()));
                    }
                }
                Ok(Value::Map(merged))
            }
            _ => Err(SassError::eval("map.deep-merge() expects two maps")),
        },
        "deep_remove" => match &args[..] {
            [Value::Map(pairs), key] => {
                let result: Vec<(Value, Value)> = pairs.iter()
                    .filter(|(k, _)| !k.equals(key))
                    .cloned()
                    .collect();
                Ok(Value::Map(result))
            }
            _ => Err(SassError::eval("map.deep-remove() expects a map and key")),
        },
        _ => Err(SassError::eval(format!("Unknown map function: {field}"))),
    }
}
