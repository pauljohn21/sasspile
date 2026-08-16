//! sass:string module — string manipulation functions.

use crate::eval::error::EvalError;
use crate::eval::evaluator::EvalContext;
use crate::parser::Expr;
use crate::value::{Number, Quoted, Value};

/// Dispatch a sass:string function call.
pub fn call(
    func: &str,
    args: &[Expr],
    ctx: &mut EvalContext<'_>,
) -> Result<Option<Value>, EvalError> {
    match func {
        "unquote" => unquote(args, ctx).map(Some),
        "quote" => string_quote(args, ctx).map(Some),
        "length" => length(args, ctx).map(Some),
        "index" => string_index(args, ctx).map(Some),
        "insert" => insert(args, ctx).map(Some),
        "slice" => slice(args, ctx).map(Some),
        "upper-case" => upper_case(args, ctx).map(Some),
        "lower-case" => lower_case(args, ctx).map(Some),
        "to-upper-case" => upper_case(args, ctx).map(Some),
        "to-lower-case" => lower_case(args, ctx).map(Some),
        "unique-id" => unique_id(args, ctx).map(Some),
        "str-slice" => slice(args, ctx).map(Some),
        _ => Ok(None),
    }
}

/// Extract a string from arguments.
fn eval_string(name: &str, args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<String, EvalError> {
    if args.is_empty() {
        return Err(EvalError::ArityMismatch(name.into(), "1+".into(), 0));
    }
    let val = ctx.eval_expr(&args[0])?;
    Ok(val.to_string_value())
}

/// Unquote a string (returns string with Unquoted quoting).
fn unquote(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let s = eval_string("unquote", args, ctx)?;
    Ok(Value::String(s, Quoted::Unquoted))
}

/// Quote a string.
fn string_quote(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let s = eval_string("quote", args, ctx)?;
    Ok(Value::String(s, Quoted::Quoted))
}

/// Get string length (character count).
fn length(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let s = eval_string("length", args, ctx)?;
    Ok(Value::Number(Number::unitless(s.chars().count() as f64)))
}

/// Find index of substring (1-indexed, 0 if not found).
fn string_index(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let s = eval_string("str-index", args, ctx)?;
    let needle = if args.len() >= 2 {
        eval_string("str-index", &args[1..], ctx)?
    } else {
        return Err(EvalError::ArityMismatch("str-index".into(), "2".into(), args.len()));
    };
    match s.find(&needle) {
        Some(pos) => Ok(Value::Number(Number::unitless(s[..pos].chars().count() as f64 + 1.0))),
        None => Ok(Value::Number(Number::unitless(0.0))),
    }
}

/// Insert a substring at a given index.
fn insert(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let s = eval_string("str-insert", args, ctx)?;
    let insert_str = if args.len() >= 2 {
        eval_string("str-insert", &args[1..], ctx)?
    } else {
        return Err(EvalError::ArityMismatch("str-insert".into(), "3".into(), args.len()))
    };
    let index = if args.len() >= 3 {
        let val = ctx.eval_expr(&args[2])?;
        match &val {
            Value::Number(n) => n.value as usize,
            _ => return Err(EvalError::type_error("number", val.type_name())),
        }
    } else {
        return Err(EvalError::ArityMismatch("str-insert".into(), "3".into(), args.len()))
    };
    // Insert at character index (0-indexed from start).
    let char_count = s.chars().count();
    let idx = if index > char_count { char_count } else { index };
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i == idx {
            result.push_str(&insert_str);
        }
        result.push(ch);
    }
    if idx >= char_count {
        result.push_str(&insert_str);
    }
    Ok(Value::String(result, Quoted::Quoted))
}

/// Slice a string from start to end.
fn slice(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let s = eval_string("str-slice", args, ctx)?;
    let start = if args.len() >= 2 {
        let val = ctx.eval_expr(&args[1])?;
        match &val {
            Value::Number(n) => n.value as usize,
            _ => return Err(EvalError::type_error("number", val.type_name())),
        }
    } else {
        return Err(EvalError::ArityMismatch("str-slice".into(), "2+".into(), args.len()))
    };
    let end = if args.len() >= 3 {
        let val = ctx.eval_expr(&args[2])?;
        match &val {
            Value::Number(n) => n.value as i64 as isize,
            _ => return Err(EvalError::type_error("number", val.type_name())),
        }
    } else {
        -1
    };
    // 1-indexed, inclusive.
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as isize;
    let s_idx = if start == 0 { 0 } else { (start - 1) as isize };
    let e_idx = if end < 0 { len } else { end };
    let s_idx = s_idx.max(0).min(len) as usize;
    let e_idx = e_idx.max(0).min(len) as usize;
    if s_idx >= e_idx || s_idx >= chars.len() {
        return Ok(Value::String(String::new(), Quoted::Quoted));
    }
    let result: String = chars[s_idx..e_idx].iter().collect();
    Ok(Value::String(result, Quoted::Quoted))
}

/// Convert to upper case.
fn upper_case(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let s = eval_string("upper-case", args, ctx)?;
    Ok(Value::String(s.to_uppercase(), Quoted::Quoted))
}

/// Convert to lower case.
fn lower_case(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let s = eval_string("lower-case", args, ctx)?;
    Ok(Value::String(s.to_lowercase(), Quoted::Quoted))
}

/// Generate a unique ID.
fn unique_id(_args: &[Expr], _ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    Ok(Value::String(format!("u{:x}", id), Quoted::Unquoted))
}
