//! sass:list built-in module.
//!
//! Implements: length, nth, set-nth, join, append, zip, index,
//! separator, is-bracketed, slash.

use crate::ast::{Arg, ListSeparator};
use crate::env::Env;
use crate::error::{SassError, SourcePos};
use crate::value::{SassList, Value};
use super::helpers::*;

/// Register all list builtins.
pub fn register(env: &mut Env) {
    let span = tracing::debug_span!("register_list", stage = "init", module = "list");
    let _enter = span.enter();

    env.register_builtin("length".into(), list_length);
    env.register_builtin("nth".into(), list_nth);
    env.register_builtin("set-nth".into(), list_set_nth);
    env.register_builtin("join".into(), list_join);
    env.register_builtin("append".into(), list_append);
    env.register_builtin("zip".into(), list_zip);
    env.register_builtin("index".into(), list_index);
    env.register_builtin("list-separator".into(), list_separator);
    env.register_builtin("is-bracketed".into(), list_is_bracketed);
    env.register_builtin("list-slash".into(), list_slash);
}

fn get_args(args: &[Arg], env: &mut Env) -> Result<Vec<Value>, SassError> {
    eval_args(args, env, &[])
}

fn list_length(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let list = expect_list(&vals[0]);
    Ok(num(list.len() as f64))
}

fn list_nth(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.len() < 2 {
        return Err(SassError::eval("nth: expected 2 arguments", SourcePos::default()));
    }
    let list = expect_list(&vals[0]);
    let idx = expect_number(&vals[1], "nth")?;
    let index = idx.value as i64;
    let len = list.len() as i64;
    let real_idx = if index < 0 {
        len + index
    } else {
        index - 1
    };
    if real_idx < 0 || real_idx >= len {
        return Err(SassError::eval(
            format!("nth: list index {} is out of bounds for list of length {}", index, len),
            SourcePos::default(),
        ));
    }
    Ok(list.items[real_idx as usize].clone())
}

fn list_set_nth(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.len() < 3 {
        return Err(SassError::eval("set-nth: expected 3 arguments", SourcePos::default()));
    }
    let mut list = expect_list(&vals[0]);
    let idx = expect_number(&vals[1], "set-nth")?;
    let new_val = vals[2].clone();
    let index = idx.value as i64;
    let len = list.len() as i64;
    let real_idx = if index < 0 {
        len + index
    } else {
        index - 1
    };
    if real_idx < 0 || real_idx >= len {
        return Err(SassError::eval(
            format!("set-nth: list index {} is out of bounds", index),
            SourcePos::default(),
        ));
    }
    list.items[real_idx as usize] = new_val;
    Ok(Value::List(list))
}

fn list_join(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.len() < 2 {
        return Err(SassError::eval("join: expected at least 2 arguments", SourcePos::default()));
    }
    let list1 = expect_list(&vals[0]);
    let list2 = expect_list(&vals[1]);
    let separator = if vals.len() >= 4 {
        match expect_string(&vals[3], "join")?.value.as_str() {
            "comma" => ListSeparator::Comma,
            "space" => ListSeparator::Space,
            "slash" => ListSeparator::Slash,
            _ => ListSeparator::Space,
        }
    } else if list1.separator != ListSeparator::Undetermined {
        list1.separator
    } else if list2.separator != ListSeparator::Undetermined {
        list2.separator
    } else {
        ListSeparator::Space
    };
    let bracketed = if vals.len() >= 5 {
        vals[4].is_truthy()
    } else {
        list1.bracketed
    };
    let mut items = list1.items;
    items.extend(list2.items);
    Ok(Value::List(SassList::new(items, separator, bracketed)))
}

fn list_append(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.len() < 2 {
        return Err(SassError::eval("append: expected at least 2 arguments", SourcePos::default()));
    }
    let mut list = expect_list(&vals[0]);
    let val = vals[1].clone();
    let separator = if vals.len() >= 3 {
        match expect_string(&vals[2], "append")?.value.as_str() {
            "comma" => ListSeparator::Comma,
            "space" => ListSeparator::Space,
            "slash" => ListSeparator::Slash,
            _ => list.separator,
        }
    } else if list.separator == ListSeparator::Undetermined {
        ListSeparator::Space
    } else {
        list.separator
    };
    list.items.push(val);
    list.separator = separator;
    Ok(Value::List(list))
}

fn list_zip(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.is_empty() {
        return Ok(Value::List(SassList::new(vec![], ListSeparator::Comma, false)));
    }
    let lists: Vec<SassList> = vals.iter().map(expect_list).collect();
    let min_len = lists.iter().map(|l| l.len()).min().unwrap_or(0);
    let mut result = Vec::new();
    for i in 0..min_len {
        let mut inner = Vec::new();
        for list in &lists {
            inner.push(list.items[i].clone());
        }
        result.push(Value::List(SassList::new(inner, ListSeparator::Space, false)));
    }
    Ok(Value::List(SassList::new(result, ListSeparator::Comma, false)))
}

fn list_index(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.len() < 2 {
        return Err(SassError::eval("index: expected 2 arguments", SourcePos::default()));
    }
    let list = expect_list(&vals[0]);
    let target = &vals[1];
    for (i, item) in list.items.iter().enumerate() {
        if item == target {
            return Ok(num((i + 1) as f64));
        }
    }
    Ok(Value::Null)
}

fn list_separator(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    // If multiple args, treat them as a comma-separated list
    if vals.len() > 1 {
        return Ok(unquoted_str("comma"));
    }
    let list = expect_list(&vals[0]);
    let sep = match list.separator {
        ListSeparator::Comma => "comma",
        ListSeparator::Space => "space",
        ListSeparator::Slash => "slash",
        ListSeparator::Undetermined => "space",
    };
    Ok(unquoted_str(sep))
}

fn list_is_bracketed(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let list = expect_list(&vals[0]);
    Ok(Value::Bool(list.bracketed))
}

fn list_slash(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let items: Vec<Value> = vals.iter().cloned().collect();
    Ok(Value::List(SassList::new(items, ListSeparator::Slash, false)))
}
