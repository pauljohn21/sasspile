//! sass:string built-in module.
//!
//! Implements: quote, unquote, to-upper-case, to-lower-case,
//! length, index, insert, slice, split, unique-id.

use crate::ast::Arg;
use crate::env::Env;
use crate::error::{SassError, SourcePos};
use crate::value::{SassString, Value};
use super::helpers::*;

/// Register all string builtins.
pub fn register(env: &mut Env) {
    let span = tracing::debug_span!("register_string", stage = "init", module = "string");
    let _enter = span.enter();

    env.register_builtin("quote".into(), string_quote);
    env.register_builtin("unquote".into(), string_unquote);
    env.register_builtin("to-upper-case".into(), string_to_upper);
    env.register_builtin("to-lower-case".into(), string_to_lower);
    env.register_builtin("str-length".into(), string_length);
    env.register_builtin("string-length".into(), string_length);
    env.register_builtin("str-index".into(), string_index);
    env.register_builtin("string-index".into(), string_index);
    env.register_builtin("str-insert".into(), string_insert);
    env.register_builtin("string-insert".into(), string_insert);
    env.register_builtin("str-slice".into(), string_slice);
    env.register_builtin("string-slice".into(), string_slice);
    env.register_builtin("str-split".into(), string_split);
    env.register_builtin("string-split".into(), string_split);
    env.register_builtin("unique-id".into(), string_unique_id);
}

fn get_args(args: &[Arg], env: &mut Env) -> Result<Vec<Value>, SassError> {
    eval_args(args, env, &[])
}

fn string_quote(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let s = expect_string(&vals[0], "quote")?;
    Ok(quoted_str(&s.value))
}

fn string_unquote(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let s = expect_string(&vals[0], "unquote")?;
    Ok(unquoted_str(&s.value))
}

fn string_to_upper(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let s = expect_string(&vals[0], "to-upper-case")?;
    let upper = s.value.to_uppercase();
    Ok(Value::String(SassString { value: upper, quoted: s.quoted }))
}

fn string_to_lower(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let s = expect_string(&vals[0], "to-lower-case")?;
    let lower = s.value.to_lowercase();
    Ok(Value::String(SassString { value: lower, quoted: s.quoted }))
}

fn string_length(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    let s = expect_string(&vals[0], "length")?;
    // Unicode-aware: count chars, not bytes
    Ok(num(s.value.chars().count() as f64))
}

fn string_index(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.len() < 2 {
        return Err(SassError::eval("index: expected 2 arguments", SourcePos::default()));
    }
    let s = expect_string(&vals[0], "index")?;
    let needle = expect_string(&vals[1], "index")?;
    // Sass uses 1-based indexing
    let pos = s.value.find(&needle.value);
    match pos {
        Some(byte_pos) => {
            let char_pos = s.value[..byte_pos].chars().count() + 1;
            Ok(num(char_pos as f64))
        }
        None => Ok(Value::Null),
    }
}

fn string_insert(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.len() < 3 {
        return Err(SassError::eval("insert: expected 3 arguments", SourcePos::default()));
    }
    let s = expect_string(&vals[0], "insert")?;
    let insert_str = expect_string(&vals[1], "insert")?;
    let idx = expect_number(&vals[2], "insert")?;
    let index = idx.value as i64;
    let chars: Vec<char> = s.value.chars().collect();
    let len = chars.len() as i64;
    // Sass: negative index counts from end; index 0 inserts at start
    let pos = if index < 0 {
        (len + index + 1).max(0) as usize
    } else if index == 0 {
        0
    } else {
        (index - 1).min(len) as usize
    };
    let mut result: Vec<char> = Vec::with_capacity(chars.len() + insert_str.value.chars().count());
    result.extend(chars[..pos].iter().copied());
    result.extend(insert_str.value.chars());
    result.extend(chars[pos..].iter().copied());
    Ok(Value::String(SassString {
        value: result.into_iter().collect(),
        quoted: s.quoted,
    }))
}

fn string_slice(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.len() < 2 {
        return Err(SassError::eval("slice: expected at least 2 arguments", SourcePos::default()));
    }
    let s = expect_string(&vals[0], "slice")?;
    let start = expect_number(&vals[1], "slice")?;
    let end = if vals.len() >= 3 {
        expect_number(&vals[2], "slice")?.value as i64
    } else {
        i64::MAX
    };
    let chars: Vec<char> = s.value.chars().collect();
    let len = chars.len() as i64;
    let start_idx = normalize_index(start.value as i64, len);
    let end_idx = normalize_index(end, len).min(len.saturating_sub(1).max(0) as usize);
    let result: String = if start_idx > end_idx || chars.is_empty() {
        String::new()
    } else {
        // Sass str-slice end is inclusive, so add 1 for Rust's exclusive range
        chars[start_idx..=end_idx].iter().collect()
    };
    Ok(Value::String(SassString { value: result, quoted: s.quoted }))
}

fn string_split(args: &[Arg], env: &mut Env) -> Result<Value, SassError> {
    let vals = get_args(args, env)?;
    if vals.len() < 2 {
        return Err(SassError::eval("split: expected 2 arguments", SourcePos::default()));
    }
    let s = expect_string(&vals[0], "split")?;
    let sep = expect_string(&vals[1], "split")?;
    let items: Vec<Value> = if sep.value.is_empty() {
        s.value.chars().map(|c| quoted_str(&c.to_string())).collect()
    } else {
        s.value.split(&sep.value).map(|part| quoted_str(part)).collect()
    };
    Ok(Value::List(crate::value::SassList::new(
        items,
        crate::ast::ListSeparator::Comma,
        false,
    )))
}

fn string_unique_id(_args: &[Arg], _env: &mut Env) -> Result<Value, SassError> {
    thread_local! {
        static COUNTER: std::cell::Cell<u64> = std::cell::Cell::new(1);
    }
    COUNTER.with(|c| {
        let id = c.get();
        c.set(id + 1);
        Ok(unquoted_str(&format!("u{:x}", id)))
    })
}

/// Normalize a 1-based Sass index to a 0-based Rust index.
fn normalize_index(index: i64, len: i64) -> usize {
    if index < 0 {
        (len + index).max(0) as usize
    } else {
        (index - 1).max(0).min(len) as usize
    }
}
