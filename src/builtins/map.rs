//! sass:map built-in module.
//!
//! Implements: get, set, merge, deep-merge, remove, deep-remove,
//! keys, values, has-key.

use crate::ast::Arg;
use crate::env::Env;
use crate::error::{SassError, SourcePos};
use crate::value::{SassMap, SassList, Value};
use crate::ast::ListSeparator;
use super::helpers::*;

/// Register all map builtins.
pub fn register(env: &mut Env) {
    let span = tracing::debug_span!("register_map", stage = "init", module = "map");
    let _enter = span.enter();

    env.register_builtin("map-get".into(), map_get);
    env.register_builtin("map-set".into(), map_set);
    env.register_builtin("map-merge".into(), map_merge);
    env.register_builtin("map-deep-merge".into(), map_deep_merge);
    env.register_builtin("map-remove".into(), map_remove);
    env.register_builtin("map-deep-remove".into(), map_deep_remove);
    env.register_builtin("map-keys".into(), map_keys);
    env.register_builtin("map-values".into(), map_values);
    env.register_builtin("map-has-key".into(), map_has_key);
}

fn get_args(args: &[Arg], env: &mut Env) -> Result<Vec<Value>, SassError> {
    eval_args(args, env, &[])
}

fn map_get(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.len() < 2 {
        return Err(SassError::eval("map-get: expected 2 arguments", SourcePos::default()));
    }
    let m = expect_map(&vals[0], "map-get")?;
    Ok(m.get(&vals[1]).cloned().unwrap_or(Value::Null))
}

fn map_set(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.len() < 3 {
        return Err(SassError::eval("map-set: expected 3 arguments", SourcePos::default()));
    }
    let mut m = expect_map(&vals[0], "map-set")?.clone();
    m.insert(vals[1].clone(), vals[2].clone());
    Ok(Value::Map(m))
}

fn map_merge(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.len() < 2 {
        return Err(SassError::eval("map-merge: expected 2 arguments", SourcePos::default()));
    }
    // In Sass, an empty list `()` is equivalent to an empty map.
    // Treat empty lists as empty maps for map-merge.
    let m1 = match &vals[0] {
        Value::Map(m) => m.clone(),
        Value::List(l) if l.items.is_empty() => crate::value::SassMap::new(),
        _ => return Err(SassError::type_err(
            format!("map-merge: expected map, got {}", vals[0].type_name()),
            SourcePos::default(),
        )),
    };
    let m2 = match &vals[1] {
        Value::Map(m) => m.clone(),
        Value::List(l) if l.items.is_empty() => crate::value::SassMap::new(),
        _ => return Err(SassError::type_err(
            format!("map-merge: expected map, got {}", vals[1].type_name()),
            SourcePos::default(),
        )),
    };
    let mut result = m1;
    for (k, v) in &m2.entries {
        result.insert(k.clone(), v.clone());
    }
    Ok(Value::Map(result))
}

fn map_deep_merge(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.len() < 2 {
        return Err(SassError::eval("map-deep-merge: expected 2 arguments", SourcePos::default()));
    }
    // In Sass, an empty list `()` is equivalent to an empty map.
    let m1 = match &vals[0] {
        Value::Map(m) => m.clone(),
        Value::List(l) if l.items.is_empty() => SassMap::new(),
        _ => return Err(SassError::type_err(
            format!("map-deep-merge: expected map, got {}", vals[0].type_name()),
            SourcePos::default(),
        )),
    };
    let m2 = match &vals[1] {
        Value::Map(m) => m.clone(),
        Value::List(l) if l.items.is_empty() => SassMap::new(),
        _ => return Err(SassError::type_err(
            format!("map-deep-merge: expected map, got {}", vals[1].type_name()),
            SourcePos::default(),
        )),
    };
    let result = deep_merge_maps(m1, &m2);
    Ok(Value::Map(result))
}

fn map_remove(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.is_empty() {
        return Err(SassError::eval("map-remove: expected at least 1 argument", SourcePos::default()));
    }
    let mut m = expect_map(&vals[0], "map-remove")?.clone();
    for key in &vals[1..] {
        m.remove(key);
    }
    Ok(Value::Map(m))
}

fn map_deep_remove(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.len() < 2 {
        return Err(SassError::eval("map-deep-remove: expected at least 2 arguments", SourcePos::default()));
    }
    let mut m = expect_map(&vals[0], "map-deep-remove")?.clone();
    let key = &vals[1];
    let rest = &vals[2..];
    deep_remove(&mut m, key, rest);
    Ok(Value::Map(m))
}

fn map_keys(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let m = expect_map(&vals[0], "map-keys")?;
    let keys: Vec<Value> = m.keys().into_iter().cloned().collect();
    Ok(Value::List(SassList::new(keys, ListSeparator::Comma, false)))
}

fn map_values(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let m = expect_map(&vals[0], "map-values")?;
    let values: Vec<Value> = m.values().into_iter().cloned().collect();
    Ok(Value::List(SassList::new(values, ListSeparator::Comma, false)))
}

fn map_has_key(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.len() < 2 {
        return Err(SassError::eval("map-has-key: expected 2 arguments", SourcePos::default()));
    }
    let m = expect_map(&vals[0], "map-has-key")?;
    let key = &vals[1];
    if m.has_key(key) {
        return Ok(Value::Bool(true));
    }
    // Deep check: follow key path
    if vals.len() > 2 {
        let mut current = m.get(key);
        for k in &vals[2..] {
            match current {
                Some(Value::Map(inner)) => {
                    current = inner.get(k);
                }
                _ => return Ok(Value::Bool(false)),
            }
        }
        return Ok(Value::Bool(current.is_some()));
    }
    Ok(Value::Bool(false))
}

/// Recursively merge two maps (deep merge).
fn deep_merge_maps(mut m1: SassMap, m2: &SassMap) -> SassMap {
    for (k, v) in &m2.entries {
        let existing = m1.get(k).cloned();
        match (existing, v) {
            (Some(Value::Map(inner)), Value::Map(m2_val)) => {
                let merged = deep_merge_maps(inner, m2_val);
                m1.insert(k.clone(), Value::Map(merged));
            }
            _ => {
                m1.insert(k.clone(), v.clone());
            }
        }
    }
    m1
}

/// Deep remove following a key path.
fn deep_remove(m: &mut SassMap, key: &Value, rest: &[Value]) {
    if rest.is_empty() {
        m.remove(key);
        return;
    }
    if let Some(Value::Map(inner)) = m.get(key).cloned() {
        let mut inner_map = inner;
        deep_remove(&mut inner_map, &rest[0], &rest[1..]);
        m.insert(key.clone(), Value::Map(inner_map));
    }
}
