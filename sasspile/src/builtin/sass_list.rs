//! sass:list module — list manipulation functions.

use crate::eval::error::EvalError;
use crate::eval::evaluator::EvalContext;
use crate::parser::Expr;
use crate::value::{Number, Separator, Value};

/// Dispatch a sass:list function call.
pub fn call(
    func: &str,
    args: &[Expr],
    ctx: &mut EvalContext<'_>,
) -> Result<Option<Value>, EvalError> {
    match func {
        "length" => length(args, ctx).map(Some),
        "nth" => nth(args, ctx).map(Some),
        "set-nth" => set_nth(args, ctx).map(Some),
        "join" => join(args, ctx).map(Some),
        "append" => append(args, ctx).map(Some),
        "index" => index(args, ctx).map(Some),
        "separator" => separator(args, ctx).map(Some),
        "is-bracketed" => is_bracketed(args, ctx).map(Some),
        "zip" => zip(args, ctx).map(Some),
        "list-separator" => separator(args, ctx).map(Some),
        _ => Ok(None),
    }
}

/// Extract the first argument as a list.
fn eval_list(name: &str, args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<(Vec<Value>, Separator), EvalError> {
    if args.is_empty() {
        return Err(EvalError::ArityMismatch(name.into(), "1+".into(), 0));
    }
    let val = ctx.eval_expr(&args[0])?;
    match &val {
        Value::List(items, sep) => Ok((items.clone(), *sep)),
        _ => Err(EvalError::type_error("list", val.type_name())),
    }
}

/// Get the length of a list.
fn length(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let (items, _) = eval_list("length", args, ctx)?;
    Ok(Value::Number(Number::unitless(items.len() as f64)))
}

/// Get the nth element (1-indexed).
fn nth(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let (items, _) = eval_list("nth", args, ctx)?;
    let n = if args.len() >= 2 {
        let val = ctx.eval_expr(&args[1])?;
        match &val {
            Value::Number(num) => num.value as usize,
            _ => return Err(EvalError::type_error("number", val.type_name())),
        }
    } else {
        return Err(EvalError::ArityMismatch("nth".into(), "2".into(), args.len()));
    };
    if n == 0 || n > items.len() {
        return Err(EvalError::ListIndexOutOfBounds(n, items.len()));
    }
    Ok(items[n - 1].clone())
}

/// Set the nth element (1-indexed), returning a new list.
fn set_nth(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let (items, sep) = eval_list("set-nth", args, ctx)?;
    let n = if args.len() >= 2 {
        let val = ctx.eval_expr(&args[1])?;
        match &val {
            Value::Number(num) => num.value as usize,
            _ => return Err(EvalError::type_error("number", val.type_name())),
        }
    } else {
        return Err(EvalError::ArityMismatch("set-nth".into(), "3".into(), args.len()))
    };
    let new_val = if args.len() >= 3 {
        ctx.eval_expr(&args[2])?
    } else {
        return Err(EvalError::ArityMismatch("set-nth".into(), "3".into(), args.len()))
    };
    if n == 0 || n > items.len() {
        return Err(EvalError::ListIndexOutOfBounds(n, items.len()));
    }
    let mut new_items = items;
    new_items[n - 1] = new_val;
    Ok(Value::List(new_items, sep))
}

/// Join two lists.
fn join(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let (list1, sep1) = eval_list("join", args, ctx)?;
    let (list2, _) = if args.len() >= 2 {
        eval_list("join", &args[1..], ctx)?
    } else {
        return Err(EvalError::ArityMismatch("join".into(), "2+".into(), args.len()));
    };
    // Determine separator: explicit or use first list's.
    let sep = if args.len() >= 3 {
        let sep_val = ctx.eval_expr(&args[2])?;
        match &sep_val {
            Value::String(s, _) => match s.as_str() {
                "comma" => Separator::Comma,
                "space" => Separator::Space,
                "slash" => Separator::Slash,
                _ => sep1,
            },
            Value::Null => sep1,
            _ => sep1,
        }
    } else {
        sep1
    };
    let mut result = list1;
    result.extend(list2);
    Ok(Value::List(result, sep))
}

/// Append an element to a list.
fn append(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let (items, sep) = eval_list("append", args, ctx)?;
    let new_val = if args.len() >= 2 {
        ctx.eval_expr(&args[1])?
    } else {
        return Err(EvalError::ArityMismatch("append".into(), "2+".into(), args.len()));
    };
    // Bracketed argument.
    let bracketed = if args.len() >= 3 {
        let val = ctx.eval_expr(&args[2])?;
        match &val {
            Value::Boolean(b) => *b,
            _ => false,
        }
    } else {
        false
    };
    let mut result = items;
    result.push(new_val);
    let mut list = Value::List(result, sep);
    if bracketed {
        // Wrap for bracketed.
        list = Value::List(vec![list], Separator::Space);
    }
    Ok(list)
}

/// Find index of a value in a list (1-indexed, 0 if not found).
fn index(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let (items, _) = eval_list("index", args, ctx)?;
    let needle = if args.len() >= 2 {
        ctx.eval_expr(&args[1])?
    } else {
        return Err(EvalError::ArityMismatch("index".into(), "2".into(), args.len()));
    };
    let found = items.iter().position(|v| v == &needle);
    Ok(Value::Number(Number::unitless(match found {
        Some(i) => (i + 1) as f64,
        None => 0.0,
    })))
}

/// Get the separator of a list as a string.
fn separator(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let (_, sep) = eval_list("separator", args, ctx)?;
    let s = match sep {
        Separator::Comma => "comma",
        Separator::Space => "space",
        Separator::Slash => "slash",
        Separator::Undecided => "space",
    };
    Ok(Value::String(s.to_string(), crate::value::Quoted::Quoted))
}

/// Check if a list is bracketed.
fn is_bracketed(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let val = ctx.eval_expr(&args[0])?;
    match &val {
        Value::List(items, _) => {
            // Check if this is a single-item list containing a list.
            Ok(Value::Boolean(items.len() == 1 && matches!(&items[0], Value::List(_, _))))
        }
        _ => Err(EvalError::type_error("list", val.type_name())),
    }
}

/// Zip multiple lists together.
fn zip(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    // Collect all lists.
    let mut all_lists: Vec<(Vec<Value>, Separator)> = Vec::new();
    for arg in args {
        let val = ctx.eval_expr(arg)?;
        match &val {
            Value::List(items, sep) => all_lists.push((items.clone(), *sep)),
            _ => return Err(EvalError::type_error("list", val.type_name())),
        }
    }
    if all_lists.is_empty() {
        return Ok(Value::List(vec![], Separator::Space));
    }
    // Find min length.
    let min_len = all_lists.iter().map(|(l, _)| l.len()).min().unwrap_or(0);
    let mut result = Vec::new();
    for i in 0..min_len {
        let zipped: Vec<Value> = all_lists.iter().map(|(l, _)| l[i].clone()).collect();
        result.push(Value::List(zipped, Separator::Space));
    }
    Ok(Value::List(result, Separator::Comma))
}
