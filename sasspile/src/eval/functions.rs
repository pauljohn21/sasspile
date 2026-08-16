//! Function and mixin call dispatch.

use crate::eval::error::EvalError;
use crate::eval::evaluator::EvalContext;
use crate::parser::Expr;
use crate::value::{Quoted, Value};

/// Dispatch a function/mixin call.
pub fn call(name: &str, args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    // Try namespaced built-in first (module.func).
    if name.contains('.')
        && let Some(result) = crate::builtin::dispatch(name, args, ctx)?
    {
        return Ok(result);
    }

    // Core built-ins (Phase 5).
    if let Some(result) = try_core_builtin(name, args, ctx)? {
        return Ok(result);
    }

    // User-defined functions.
    if let Some(func) = ctx.definitions.get_function(name) {
        return ctx.call_user_function(func, args);
    }

    // CSS built-in functions (min, max, abs, round, ceil, floor).
    css_function(name, args, ctx)
}

/// Try core built-in functions (unquote, quote, length, nth).
fn try_core_builtin(
    name: &str,
    args: &[Expr],
    ctx: &mut EvalContext<'_>,
) -> Result<Option<Value>, EvalError> {
    match name {
        "unquote" => {
            let arg = eval_single_arg("unquote", args, ctx)?;
            Ok(Some(arg))
        }
        "quote" => {
            let arg = eval_single_arg("quote", args, ctx)?;
            Ok(Some(Value::String(arg.to_string_value(), Quoted::Quoted)))
        }
        "length" => {
            let arg = eval_single_arg("length", args, ctx)?;
            let len = match &arg {
                Value::List(items, _) => items.len(),
                Value::Map(entries) => entries.len(),
                Value::String(s, _) => s.chars().count(),
                _ => 1,
            };
            Ok(Some(Value::Number(crate::value::Number::unitless(len as f64))))
        }
        "nth" => {
            let list_expr = args.first().ok_or_else(|| {
                EvalError::ArityMismatch("nth".into(), "2".into(), args.len())
            })?;
            let n_expr = args.get(1).ok_or_else(|| {
                EvalError::ArityMismatch("nth".into(), "2".into(), args.len())
            })?;
            let list = ctx.eval_expr(list_expr)?;
            let n = ctx.eval_expr(n_expr)?;
            Ok(Some(crate::eval::collections::nth(&list, &n)?))
        }
        _ => Ok(None),
    }
}

/// Evaluate single-argument function's argument.
fn eval_single_arg(name: &str, args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::ArityMismatch(name.into(), "1".into(), args.len()));
    }
    ctx.eval_expr(&args[0])
}

/// Dispatch CSS built-in functions (min, max, calc, etc.).
fn css_function(name: &str, args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    match name {
        "min" | "max" => {
            let values: Result<Vec<_>, _> = args.iter().map(|a| ctx.eval_expr(a)).collect();
            let values = values?;
            compare_many(name, &values)
        }
        "abs" => {
            let arg = eval_single_arg("abs", args, ctx)?;
            if let Value::Number(n) = &arg {
                Ok(Value::Number(crate::value::Number::new(n.value.abs(), n.unit.clone())))
            } else {
                Err(EvalError::type_error("number", arg.type_name()))
            }
        }
        "round" => {
            let arg = eval_single_arg("round", args, ctx)?;
            if let Value::Number(n) = &arg {
                Ok(Value::Number(crate::value::Number::new(n.value.round(), n.unit.clone())))
            } else {
                Err(EvalError::type_error("number", arg.type_name()))
            }
        }
        "ceil" => {
            let arg = eval_single_arg("ceil", args, ctx)?;
            if let Value::Number(n) = &arg {
                Ok(Value::Number(crate::value::Number::new(n.value.ceil(), n.unit.clone())))
            } else {
                Err(EvalError::type_error("number", arg.type_name()))
            }
        }
        "floor" => {
            let arg = eval_single_arg("floor", args, ctx)?;
            if let Value::Number(n) = &arg {
                Ok(Value::Number(crate::value::Number::new(n.value.floor(), n.unit.clone())))
            } else {
                Err(EvalError::type_error("number", arg.type_name()))
            }
        }
        _ => Err(EvalError::UndefinedCallable(name.into())),
    }
}

/// Compare a list of numbers and return min or max.
fn compare_many(name: &str, values: &[Value]) -> Result<Value, EvalError> {
    if values.is_empty() {
        return Err(EvalError::ArityMismatch(name.into(), "at least 1".into(), 0));
    }

    let mut result = &values[0];
    for val in &values[1..] {
        match (result, val) {
            (Value::Number(a), Value::Number(b)) => {
                let should_replace = match name {
                    "min" => b.value < a.value,
                    "max" => b.value > a.value,
                    _ => false,
                };
                if should_replace {
                    result = val;
                }
            }
            _ => return Err(EvalError::TypeError("comparison requires numbers".into())),
        }
    }
    Ok(result.clone())
}
