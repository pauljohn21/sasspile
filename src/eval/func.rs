//! User function calls, parameter binding, and spread argument expansion.

use crate::ast::*;
use crate::env::Env;
use crate::error::SassError;
use crate::resolver::ModuleResolver;
use crate::value::{SassList, Value};
use super::expr::eval_expr;

/// Call a user-defined function.
///
/// Arguments are pre-evaluated in the caller's environment before creating
/// the function's child scope.  This is critical because
/// `std::mem::replace(env, …)` would otherwise leave `env` pointing at a
/// temporary empty environment while argument expressions are evaluated.
pub fn call_user_function(
    func: &crate::env::UserFunction,
    args: &[Arg],
    env: &mut Env,
    parent_sel: &[String],
    resolver: &mut dyn ModuleResolver,
) -> Result<Value, SassError> {
    // Pre-evaluate all arguments in the *caller's* environment before
    // creating the function's child scope.
    // Also expand spread args ($val...) — maps become named args,
    // lists become positional args.
    let expanded = expand_spread_args(args, env, parent_sel, resolver)?;

    // Separate into named and positional args
    let mut named: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    let mut positional: Vec<Value> = Vec::new();
    for (name, val) in expanded {
        if let Some(n) = name {
            named.insert(n, val);
        } else {
            positional.push(val);
        }
    }

    let mut evaluated: Vec<(String, Value)> = Vec::new();
    let mut pos_idx = 0;
    for (_i, param) in func.params.iter().enumerate() {
        if param.rest {
            let mut items = Vec::new();
            while pos_idx < positional.len() {
                items.push(positional[pos_idx].clone());
                pos_idx += 1;
            }
            evaluated.push((
                param.name.clone(),
                Value::List(SassList::new(items, ListSeparator::Comma, false)),
            ));
            break;
        }

        // Try named match first, then positional
        let value = if let Some(v) = named.get(&param.name) {
            v.clone()
        } else if pos_idx < positional.len() {
            let v = positional[pos_idx].clone();
            pos_idx += 1;
            v
        } else if let Some(default) = &param.default {
            eval_expr(default, env, parent_sel, resolver)?
        } else {
            Value::Null
        };
        evaluated.push((param.name.clone(), value));
    }

    // Now create the child environment and bind the pre-evaluated values.
    env.with_child_scope(|func_env| -> Result<Value, SassError> {
        for (name, value) in evaluated {
            func_env.set_var(name, value, false, false);
        }

        let mut result = Value::Null;
        let mut dummy_output: Vec<super::CssRule> = Vec::new();
        let mut dummy_extends: Vec<super::ExtendEntry> = Vec::new();
        let mut dummy_cache = super::ModuleCache::new();
        for stmt in &func.body {
            if let Stmt::ReturnStmt(expr) = stmt {
                result = eval_expr(expr, func_env, parent_sel, resolver)?;
                break;
            }
            // Execute non-return statements (variable declarations, @each, @if, etc.)
            super::eval_stmt(stmt, func_env, parent_sel, &mut dummy_output, &mut dummy_extends, resolver, &mut dummy_cache)?;
        }

        Ok(result)
    })
}

/// Bind parameters to arguments (legacy interface used by some builtins).
pub fn bind_params(
    params: &[Param],
    args: &[Arg],
    func_env: &mut Env,
    env: &mut Env,
    parent_sel: &[String],
    resolver: &mut dyn ModuleResolver,
) -> Result<(), SassError> {
    for (i, param) in params.iter().enumerate() {
        if param.rest {
            let mut items = Vec::new();
            for arg in args.iter().skip(i) {
                items.push(eval_expr(&arg.value, env, parent_sel, resolver)?);
            }
            func_env.set_var(param.name.clone(), Value::List(SassList::new(items, ListSeparator::Comma, false)), false, false);
            break;
        }

        let val = args.iter().find(|a| a.name.as_deref() == Some(param.name.as_str()))
            .or_else(|| args.get(i))
            .map(|a| &a.value);

        let value = if let Some(expr) = val {
            eval_expr(expr, env, parent_sel, resolver)?
        } else if let Some(default) = &param.default {
            eval_expr(default, env, parent_sel, resolver)?
        } else {
            Value::Null
        };

        func_env.set_var(param.name.clone(), value, false, false);
    }
    Ok(())
}

/// Call a user function with already-prepared args.
/// This is used by meta.call to invoke a function reference.
pub fn bind_params_and_call(
    func: &crate::env::UserFunction,
    args: &[Arg],
    env: &mut Env,
    parent_sel: &[String],
    resolver: &mut dyn ModuleResolver,
) -> Result<Value, SassError> {
    call_user_function(func, args, env, parent_sel, resolver)
}

/// Pre-evaluate spread arguments (`$val...`) into a flat list of
/// `(name, value)` pairs.
///
/// When `arg.spread` is true:
/// - **Map**: each `(key, value)` becomes a named arg (`key` must be a string)
/// - **List**: each element becomes a positional arg (name = None)
/// - **Other**: treated as a single-element list (one positional arg)
///
/// Non-spread args are passed through with their expression pre-evaluated.
pub fn expand_spread_args(
    args: &[Arg],
    env: &mut Env,
    parent_sel: &[String],
    resolver: &mut dyn ModuleResolver,
) -> Result<Vec<(Option<String>, Value)>, SassError> {
    let span = tracing::debug_span!("expand_spread_args", stage = "eval", module = "args");
    let _enter = span.enter();
    let mut result: Vec<(Option<String>, Value)> = Vec::new();

    for arg in args {
        let val = eval_expr(&arg.value, env, parent_sel, resolver)?;
        if arg.spread {
            match &val {
                Value::Map(m) => {
                    for (k, v) in &m.entries {
                        let name = match k {
                            Value::String(s) => Some(s.value.clone()),
                            _ => None,
                        };
                        result.push((name, v.clone()));
                    }
                }
                Value::List(l) => {
                    for item in &l.items {
                        result.push((None, item.clone()));
                    }
                }
                _ => {
                    result.push((arg.name.clone(), val));
                }
            }
        } else {
            result.push((arg.name.clone(), val));
        }
    }

    tracing::debug!(
        stage = "eval", module = "args",
        input_count = args.len(),
        output_count = result.len(),
        "spread args expanded"
    );
    Ok(result)
}
