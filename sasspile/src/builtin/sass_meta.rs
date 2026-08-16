//! sass:meta module — reflective/meta-programming functions.

use crate::eval::error::EvalError;
use crate::eval::evaluator::EvalContext;
use crate::parser::Expr;
use crate::value::{Quoted, Value};

/// Dispatch a sass:meta function call.
pub fn call(
    func: &str,
    args: &[Expr],
    ctx: &mut EvalContext<'_>,
) -> Result<Option<Value>, EvalError> {
    match func {
        "type-of" => type_of(args, ctx).map(Some),
        "unit" => unit(args, ctx).map(Some),
        "content-exists" => content_exists(args, ctx).map(Some),
        "function-exists" => function_exists(args, ctx).map(Some),
        "variable-exists" => variable_exists(args, ctx).map(Some),
        "global-variable-exists" => global_variable_exists(args, ctx).map(Some),
        "mixin-exists" => mixin_exists(args, ctx).map(Some),
        "get-function" => get_function(args, ctx).map(Some),
        "call" => meta_call(args, ctx).map(Some),
        "keywords" => keywords(args, ctx).map(Some),
        "inspect" => inspect(args, ctx).map(Some),
        _ => Ok(None),
    }
}

/// Get the type of a value as a string.
fn type_of(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let val = ctx.eval_expr(&args[0])?;
    let type_name = match &val {
        Value::Number(..) => "number",
        Value::String(..) => "string",
        Value::Boolean(..) => "bool",
        Value::Null => "null",
        Value::Color(..) => "color",
        Value::List(..) => "list",
        Value::Map(..) => "map",
        Value::ArgList(..) => "arglist",
        Value::Function(..) => "function",
        Value::Calculation(..) => "calculation",
        Value::Error(..) => "error",
    };
    Ok(Value::String(type_name.to_string(), Quoted::Quoted))
}

/// Get the unit of a number as a string.
fn unit(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let val = ctx.eval_expr(&args[0])?;
    match &val {
        Value::Number(n) => {
            let unit_str = format!("{:?}", n.unit);
            Ok(Value::String(unit_str, Quoted::Quoted))
        }
        _ => Err(EvalError::type_error("number", val.type_name())),
    }
}

/// Check if content block is passed.
fn content_exists(_args: &[Expr], _ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    // In sasslipe, we don't have @content blocks yet, so always return false.
    Ok(Value::Boolean(false))
}

/// Check if a function exists (built-in or user-defined).
fn function_exists(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let name = eval_string_name("function-exists", args, ctx)?;
    // Check built-in and user-defined.
    let found = is_builtin_function(&name) || ctx.definitions.has_function(&name);
    Ok(Value::Boolean(found))
}

/// Check if a variable exists in the current scope.
fn variable_exists(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let name = eval_string_name("variable-exists", args, ctx)?;
    // Look in symbol table.
    Ok(Value::Boolean(ctx.symbols.lookup(&name).is_some()))
}

/// Check if a variable exists in the global scope.
fn global_variable_exists(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let name = eval_string_name("global-variable-exists", args, ctx)?;
    // Check the global scope only.
    let found = ctx.symbols.depth() == 1 && ctx.symbols.lookup(&name).is_some();
    Ok(Value::Boolean(found))
}

/// Check if a mixin exists.
fn mixin_exists(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let name = eval_string_name("mixin-exists", args, ctx)?;
    Ok(Value::Boolean(ctx.definitions.has_mixin(&name)))
}

/// Get a function reference by name.
fn get_function(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let name = eval_string_name("get-function", args, ctx)?;
    let quoted = if args.len() >= 2 {
        let val = ctx.eval_expr(&args[1])?;
        match &val {
            Value::Boolean(b) => *b,
            _ => false,
        }
    } else {
        false
    };
    if quoted {
        // Quote the name for safety.
        return Ok(Value::String(name, Quoted::Quoted));
    }
    Ok(Value::Function(name))
}

/// Call a function dynamically by name.
fn meta_call(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let name = eval_string_name("call", args, ctx)?;
    let call_args: Vec<Expr> = args[1..].to_vec();
    crate::eval::functions::call(&name, &call_args, ctx)
}

/// Get keywords from an argument list.
fn keywords(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let val = ctx.eval_expr(&args[0])?;
    match &val {
        Value::ArgList(items) => {
            // Last n items are keyword args.
            let keys: Vec<Value> = items.to_vec();
            Ok(Value::List(keys, crate::value::Separator::Comma))
        }
        _ => Err(EvalError::type_error("arglist", val.type_name())),
    }
}

/// Inspect a value and return its string representation.
fn inspect(args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<Value, EvalError> {
    let val = ctx.eval_expr(&args[0])?;
    match &val {
        Value::String(s, _) => Ok(Value::String(s.clone(), Quoted::Quoted)),
        _ => Ok(Value::String(val.to_css_string(), Quoted::Quoted)),
    }
}

/// Evaluate a string literal argument.
fn eval_string_name(name: &str, args: &[Expr], ctx: &mut EvalContext<'_>) -> Result<String, EvalError> {
    if args.is_empty() {
        return Err(EvalError::ArityMismatch(name.into(), "1+".into(), 0));
    }
    let val = ctx.eval_expr(&args[0])?;
    match &val {
        Value::String(s, _) => Ok(s.clone()),
        _ => Err(EvalError::type_error("string", val.type_name())),
    }
}

/// Check if a name is a built-in function (Phase 5 + 6).
fn is_builtin_function(name: &str) -> bool {
    matches!(
        name,
        "unquote"
            | "quote"
            | "length"
            | "nth"
            | "abs"
            | "round"
            | "ceil"
            | "floor"
            | "min"
            | "max"
    )
}
