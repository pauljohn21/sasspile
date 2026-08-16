//! sass:map module — map (key-value pair collection) operations.

use crate::eval::error::EvalError;
use crate::eval::evaluator::EvalContext;
use crate::parser::Expr;
use crate::value::{Separator, Value};

/// Dispatch a sass:map function call.
pub fn call(
    func: &str,
    args: &[Expr],
    ctx: &mut EvalContext<'_>,
) -> Result<Option<Value>, EvalError> {
    match func {
        "get" => get(args, ctx).map(Some),
        "merge" => merge(args, ctx).map(Some),
        "keys" => keys(args, ctx).map(Some),
        "values" => values(args, ctx).map(Some),
        "has-key" => has_key(args, ctx).map(Some),
        "remove" => remove(args, ctx).map(Some),
        "set" => set(args, ctx).map(Some),
        "deep-merge" => deep_merge(args, ctx).map(Some),
        "deep-get" => deep_get(args, ctx).map(Some),
        _ => Ok(None),
    }
}

/// Extract the first argument as a map.
fn eval_map(name: &str, args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Vec<(Value, Value)>, EvalError> {
    if args.is_empty() {
        return Err(EvalError::ArityMismatch(name.into(), "1+".into(), 0));
    }
    let val = ctx.eval_expr(&args[0])?;
    match &val {
        Value::Map(entries) => Ok(entries.clone()),
        _ => Err(EvalError::type_error("map", val.type_name())),
    }
}

/// Get a value by key.
fn get(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let entries = eval_map("get", args, ctx)?;
    let key = if args.len() >= 2 {
        ctx.eval_expr(&args[1])?
    } else {
        return Err(EvalError::ArityMismatch("get".into(), "2".into(), args.len()));
    };
    for (k, v) in &entries {
        if k == &key {
            return Ok(v.clone());
        }
    }
    Ok(Value::Null)
}

/// Set a key to a value.
fn set(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let mut entries = eval_map("set", args, ctx)?;
    let key = ctx.eval_expr(&args[1])?;
    let val = ctx.eval_expr(&args[2])?;
    // Replace if exists.
    if let Some(pos) = entries.iter().position(|(k, _)| k == &key) {
        entries[pos] = (key, val);
    } else {
        entries.push((key, val));
    }
    Ok(Value::Map(entries))
}

/// Merge two maps (shallow, second wins).
fn merge(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let map1 = eval_map("merge", args, ctx)?;
    let map2 = if args.len() >= 2 {
        eval_map("merge", &args[1..], ctx)?
    } else {
        return Err(EvalError::ArityMismatch("merge".into(), "2".into(), args.len()));
    };
    let mut result = map1;
    for (k, v) in &map2 {
        if let Some(pos) = result.iter().position(|(rk, _)| rk == k) {
            result[pos] = (k.clone(), v.clone());
        } else {
            result.push((k.clone(), v.clone()));
        }
    }
    Ok(Value::Map(result))
}

/// Deep merge: merges nested maps recursively.
fn deep_merge(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let map1 = eval_map("deep-merge", args, ctx)?;
    let map2 = if args.len() >= 2 {
        eval_map("deep-merge", &args[1..], ctx)?
    } else {
        return Err(EvalError::ArityMismatch("deep-merge".into(), "2".into(), args.len()));
    };
    let mut result = map1;
    for (k, v) in &map2 {
        if let Some(pos) = result.iter().position(|(rk, _)| rk == k) {
            // If both values are maps, recurse by wrapping and calling deep-merge.
            if let (Value::Map(sub1), Value::Map(sub2)) = (&result[pos].1, v) {
                let merged = merge_two_maps(sub1.clone(), sub2.clone());
                result[pos] = (k.clone(), Value::Map(merged));
            } else {
                result[pos] = (k.clone(), v.clone());
            }
        } else {
            result.push((k.clone(), v.clone()));
        }
    }
    Ok(Value::Map(result))
}

/// Helper: merge two map entry lists.
fn merge_two_maps(
    map1: Vec<(Value, Value)>,
    map2: Vec<(Value, Value)>,
) -> Vec<(Value, Value)> {
    let mut result = map1;
    for (k, v) in &map2 {
        if let Some(pos) = result.iter().position(|(rk, _)| rk == k) {
            if let (Value::Map(sub1), Value::Map(sub2)) = (&result[pos].1, v) {
                result[pos] = (k.clone(), Value::Map(merge_two_maps(sub1.clone(), sub2.clone())));
            } else {
                result[pos] = (k.clone(), v.clone());
            }
        } else {
            result.push((k.clone(), v.clone()));
        }
    }
    result
}

/// Deep get using a key path.
fn deep_get(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let entries = eval_map("deep-get", args, ctx)?;
    let path = if args.len() >= 2 {
        let val = ctx.eval_expr(&args[1])?;
        match &val {
            Value::List(items, _) => items.clone(),
            other => vec![other.clone()],
        }
    } else {
        return Err(EvalError::ArityMismatch("deep-get".into(), "2+".into(), args.len()));
    };
    let mut current = Value::Map(entries);
    for key in &path {
        match &current {
            Value::Map(entries) => {
                let mut found = None;
                for (k, v) in entries {
                    if k == key {
                        found = Some(v.clone());
                        break;
                    }
                }
                match found {
                    Some(v) => current = v,
                    None => return Ok(Value::Null),
                }
            }
            _ => return Ok(Value::Null),
        }
    }
    Ok(current)
}

/// Get all keys of a map as a list.
fn keys(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let entries = eval_map("keys", args, ctx)?;
    let keys: Vec<Value> = entries.into_iter().map(|(k, _)| k).collect();
    Ok(Value::List(keys, Separator::Comma))
}

/// Get all values of a map as a list.
fn values(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let entries = eval_map("values", args, ctx)?;
    let vals: Vec<Value> = entries.into_iter().map(|(_, v)| v).collect();
    Ok(Value::List(vals, Separator::Comma))
}

/// Check if a key exists in a map.
fn has_key(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let entries = eval_map("has-key", args, ctx)?;
    let key = if args.len() >= 2 {
        ctx.eval_expr(&args[1])?
    } else {
        return Err(EvalError::ArityMismatch("has-key".into(), "2".into(), args.len()));
    };
    Ok(Value::Boolean(entries.iter().any(|(k, _)| k == &key)))
}

/// Remove a key from a map.
fn remove(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let entries = eval_map("remove", args, ctx)?;
    let key = if args.len() >= 2 {
        ctx.eval_expr(&args[1])?
    } else {
        return Err(EvalError::ArityMismatch("remove".into(), "2+".into(), args.len()));
    };
    let filtered: Vec<(Value, Value)> = entries.into_iter().filter(|(k, _)| k != &key).collect();
    Ok(Value::Map(filtered))
}
