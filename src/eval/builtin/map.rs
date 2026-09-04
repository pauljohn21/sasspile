//! Map 内建函数。
//!
//! 包含 map-get/map-keys/map-values/map-has-key/map-merge/map-remove/map-set/map-deep-remove。
//! 支持嵌套路径（map.get(map, k1, k2, ...)）和空列表/Null 作为空 map。

use super::super::{Env, Evaluator};
use crate::error::{Result, SassError};
use crate::parse::ast::*;

impl Evaluator {
    /// 将 Value 转换为 Map（空列表/Null 视为空 map）。
    pub(crate) fn value_to_map(v: &Value) -> Result<Vec<(Value, Value)>> {
        match v {
            Value::Map(pairs) => Ok(pairs.clone()),
            Value::Null => Ok(Vec::new()),
            Value::List(elements, _, _) if elements.is_empty() => Ok(Vec::new()),
            _ => Err(SassError::Eval(format!("{v} is not a map"))),
        }
    }

    /// 嵌套 map.merge: map.merge(map, k1, k2, ..., map2) — 将 map2 合并到 map[k1][k2]。
    pub(crate) fn nested_map_merge(
        map: &[(Value, Value)],
        keys: &[Value],
        map2: &[(Value, Value)],
    ) -> Result<Vec<(Value, Value)>> {
        match keys.is_empty() {
            true => {
                let result = map2.iter().fold(map.to_vec(), |mut acc, (k, v)| {
                    match acc
                        .iter_mut()
                        .find(|(ek, _)| crate::eval::value::values_eq(ek, k))
                    {
                        Some(entry) => entry.1 = v.clone(),
                        None => acc.push((k.clone(), v.clone())),
                    }
                    acc
                });
                Ok(result)
            }
            false => {
                let key = &keys[0];
                let remaining = &keys[1..];
                let (result, found) = map.iter().try_fold(
                    (Vec::<(Value, Value)>::new(), false),
                    |(mut acc, _found), (k, v)| -> Result<(Vec<(Value, Value)>, bool)> {
                        match crate::eval::value::values_eq(k, key) {
                            true => {
                                let inner_map = Self::value_to_map(v).unwrap_or_default();
                                let new_inner = match remaining.is_empty() {
                                    true => map2.iter().fold(inner_map, |mut merged, (mk, mv)| {
                                        match merged
                                            .iter_mut()
                                            .find(|(ek, _)| crate::eval::value::values_eq(ek, mk))
                                        {
                                            Some(entry) => entry.1 = mv.clone(),
                                            None => merged.push((mk.clone(), mv.clone())),
                                        }
                                        merged
                                    }),
                                    false => Self::nested_map_merge(&inner_map, remaining, map2)?,
                                };
                                acc.push((k.clone(), Value::Map(new_inner)));
                                Ok((acc, true))
                            }
                            false => {
                                acc.push((k.clone(), v.clone()));
                                Ok((acc, false))
                            }
                        }
                    },
                )?;
                match found {
                    true => Ok(result),
                    false => {
                        let inner = match remaining.is_empty() {
                            true => map2.to_vec(),
                            false => Self::nested_map_merge(&[], remaining, map2)?,
                        };
                        let mut result = result;
                        result.push((key.clone(), Value::Map(inner)));
                        Ok(result)
                    }
                }
            }
        }
    }

    /// 嵌套 map.set: map.set(map, k1, k2, ..., value) — 在 map[k1][k2] 设置 value。
    pub(crate) fn nested_map_set(
        map: &[(Value, Value)],
        keys: &[Value],
        value: Value,
    ) -> Result<Vec<(Value, Value)>> {
        match keys.is_empty() {
            true => Ok(map.to_vec()),
            false => match keys.len() == 1 {
                true => {
                    let key = &keys[0];
                    let mut result = map.to_vec();
                    match result
                        .iter_mut()
                        .find(|(ek, _)| crate::eval::value::values_eq(ek, key))
                    {
                        Some(entry) => entry.1 = value,
                        None => result.push((key.clone(), value)),
                    }
                    Ok(result)
                }
                false => {
                    let key = &keys[0];
                    let remaining = &keys[1..];
                    let (mut result, found) = map.iter().try_fold(
                        (Vec::<(Value, Value)>::new(), false),
                        |(mut acc, _found), (k, v)| -> Result<(Vec<(Value, Value)>, bool)> {
                            match crate::eval::value::values_eq(k, key) {
                                true => {
                                    let inner_map = Self::value_to_map(v).unwrap_or_default();
                                    let new_inner =
                                        Self::nested_map_set(&inner_map, remaining, value.clone())?;
                                    acc.push((k.clone(), Value::Map(new_inner)));
                                    Ok((acc, true))
                                }
                                false => {
                                    acc.push((k.clone(), v.clone()));
                                    Ok((acc, false))
                                }
                            }
                        },
                    )?;
                    match !found {
                        true => {
                            let inner = Self::nested_map_set(&[], remaining, value)?;
                            result.push((key.clone(), Value::Map(inner)));
                        }
                        false => {}
                    }
                    Ok(result)
                }
            },
        }
    }

    /// map 函数分派。返回 Ok(Some(value)) 表示已处理，Ok(None) 表示不匹配。
    pub(crate) fn call_map_builtin(name: &str, args: &[Value], env: &Env) -> Result<Option<Value>> {
        let result = match name {
            "map-get" => {
                match args.len() < 2 {
                    true => {
                        return Err(SassError::Eval(
                            "map-get requires (map, key) arguments".into(),
                        ))
                    }
                    false => {}
                }
                args[1..]
                    .iter()
                    .try_fold(args[0].clone(), |current, key| -> Result<Value> {
                        let pairs = Self::value_to_map(&current)?;
                        match pairs
                            .iter()
                            .find(|(k, _)| crate::eval::value::values_eq(k, key))
                        {
                            Some((_, v)) => Ok(v.clone()),
                            None => Ok(Value::Null),
                        }
                    })?
            }
            "map-keys" => {
                match args.len() != 1 {
                    true => return Err(SassError::Eval("map-keys requires 1 map argument".into())),
                    false => {}
                }
                let pairs = Self::value_to_map(&args[0])?;
                Value::List(
                    pairs.iter().map(|(k, _)| k.clone()).collect(),
                    Separator::Comma,
                    false,
                )
            }
            "map-values" => {
                match args.len() != 1 {
                    true => {
                        return Err(SassError::Eval("map-values requires 1 map argument".into()))
                    }
                    false => {}
                }
                let pairs = Self::value_to_map(&args[0])?;
                Value::List(
                    pairs.iter().map(|(_, v)| v.clone()).collect(),
                    Separator::Comma,
                    false,
                )
            }
            "map-has-key" => {
                match args.len() < 2 {
                    true => {
                        return Err(SassError::Eval(
                            "map-has-key requires (map, key) arguments".into(),
                        ))
                    }
                    false => {}
                }
                let mut current = args[0].clone();
                let mut found = true;
                for key in &args[1..] {
                    let Ok(p) = Self::value_to_map(&current) else {
                        found = false;
                        break;
                    };
                    let pairs = p;
                    match pairs
                        .iter()
                        .find(|(k, _)| crate::eval::value::values_eq(k, key))
                    {
                        Some((_, v)) => current = v.clone(),
                        None => {
                            found = false;
                            break;
                        }
                    }
                }
                Value::Bool(found)
            }
            "map-merge" => {
                match args.len() < 2 {
                    true => {
                        return Err(SassError::Eval(
                            "map-merge requires at least 2 arguments".into(),
                        ))
                    }
                    false => {}
                }
                let map1 = Self::value_to_map(&args[0])?;
                match args.len() > 2 {
                    true => {
                        let keys = &args[1..args.len() - 1];
                        let map2 = Self::value_to_map(&args[args.len() - 1])?;
                        let result = Self::nested_map_merge(&map1, keys, &map2)?;
                        return Ok(Some(Value::Map(result)));
                    }
                    false => {}
                }
                let map2 = Self::value_to_map(&args[1])?;
                let merged = map2.iter().fold(map1, |mut acc, (k, v)| {
                    match acc
                        .iter_mut()
                        .find(|(ek, _)| crate::eval::value::values_eq(ek, k))
                    {
                        Some(entry) => entry.1 = v.clone(),
                        None => acc.push((k.clone(), v.clone())),
                    }
                    acc
                });
                Value::Map(merged)
            }
            "map-remove" => {
                match args.is_empty() {
                    true => {
                        return Err(SassError::Eval(
                            "map-remove requires at least 1 argument".into(),
                        ))
                    }
                    false => {}
                }
                let pairs = Self::value_to_map(&args[0])?;
                match args.len() == 1 {
                    true => return Ok(Some(Value::Map(pairs))),
                    false => {}
                }
                let keys = &args[1..];
                let filtered: Vec<(Value, Value)> = pairs
                    .iter()
                    .filter(|(k, _)| !keys.iter().any(|key| crate::eval::value::values_eq(k, key)))
                    .cloned()
                    .collect();
                Value::Map(filtered)
            }
            "map-set" => {
                match args.len() < 3 {
                    true => {
                        return Err(SassError::Eval(
                            "map-set requires at least 3 arguments".into(),
                        ))
                    }
                    false => {}
                }
                let map = Self::value_to_map(&args[0])?;
                let keys = &args[1..args.len() - 1];
                let value = &args[args.len() - 1];
                let result = Self::nested_map_set(&map, keys, value.clone())?;
                Value::Map(result)
            }
            "map-deep-merge" => {
                match args.len() != 2 {
                    true => {
                        return Err(SassError::Eval(
                            "map-deep-merge requires 2 arguments".into(),
                        ))
                    }
                    false => {}
                }
                let map1 = Self::value_to_map(&args[0])?;
                let map2 = Self::value_to_map(&args[1])?;
                Value::Map(Self::deep_merge_maps(&map1, &map2))
            }
            "map-deep-remove" => {
                return Self::map_deep_remove(args, env).map(Some);
            }
            _ => return Ok(None),
        };
        Ok(Some(result))
    }

    /// 递归合并两个 map——当同一键的两个值都是 map 时递归合并。
    fn deep_merge_maps(map1: &[(Value, Value)], map2: &[(Value, Value)]) -> Vec<(Value, Value)> {
        map2.iter().fold(map1.to_vec(), |mut acc, (k2, v2)| {
            match acc
                .iter_mut()
                .find(|(k1, _)| crate::eval::value::values_eq(k1, k2))
            {
                Some(entry) => {
                    match (&entry.1, v2) {
                        (Value::Map(inner1), Value::Map(inner2)) => {
                            entry.1 = Value::Map(Self::deep_merge_maps(inner1, inner2));
                        }
                        _ => entry.1 = v2.clone(),
                    }
                }
                None => acc.push((k2.clone(), v2.clone())),
            }
            acc
        })
    }

    /// map-deep-remove 递归实现。
    fn map_deep_remove(args: &[Value], env: &Env) -> Result<Value> {
        match args {
            [Value::Map(pairs), key @ ..] => {
                let keys: Vec<&Value> = key.iter().collect();
                match keys.is_empty() {
                    true => return Ok(Value::Map(pairs.clone())),
                    false => {}
                }
                let target_key = keys[0];
                let remaining_keys = &keys[1..];
                let result: Vec<(Value, Value)> = pairs
                    .iter()
                    .filter(|(k, _)| !crate::eval::value::values_eq(k, target_key))
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                let (result, _) = pairs.iter().try_fold(
                    (result, false),
                    |(mut acc, _found), (k, v)| -> Result<(Vec<(Value, Value)>, bool)> {
                        match crate::eval::value::values_eq(k, target_key) {
                            true => match (remaining_keys.is_empty(), v) {
                                (true, _) => Ok((acc, true)),
                                (false, Value::Map(inner)) => {
                                    let new_inner = Self::call_builtin(
                                        "map-deep-remove",
                                        &[Value::Map(inner.clone()), remaining_keys[0].clone()],
                                        &std::collections::HashMap::new(),
                                        env,
                                    )?;
                                    acc.push((k.clone(), new_inner));
                                    Ok((acc, true))
                                }
                                (false, _) => {
                                    acc.push((k.clone(), v.clone()));
                                    Ok((acc, true))
                                }
                            },
                            false => Ok((acc, false)),
                        }
                    },
                )?;
                Ok(Value::Map(result))
            }
            [other, ..] => Ok(other.clone()),
            _ => Err(SassError::Eval(
                "map-deep-remove requires at least 1 argument".into(),
            )),
        }
    }
}
