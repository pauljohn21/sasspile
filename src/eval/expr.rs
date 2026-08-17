//! Expression evaluator — evaluates AST expressions to Values.

use crate::ast::*;
use crate::env::Env;
use crate::error::{SassError, SourcePos};
use crate::operators::{apply_binop, apply_unaryop};
use crate::value::{SassString, Value};

/// Evaluate an expression to a Value.
pub fn eval_expr(expr: &Expr, env: &mut Env, parent_sel: &[String]) -> Result<Value, SassError> {
    match expr {
        Expr::Literal(v) => Ok(v.clone()),
        Expr::Variable(name) => {
            env.get_var(name).cloned().ok_or_else(|| {
                SassError::eval(format!("Undefined variable: ${}", name), SourcePos::default())
            })
        }
        Expr::Operation { op, left, right } => {
            let lv = eval_expr(left, env, parent_sel)?;
            let rv = eval_expr(right, env, parent_sel)?;
            apply_binop(op, &lv, &rv, &SourcePos::default())
        }
        Expr::UnaryOp { op, operand } => {
            let val = eval_expr(operand, env, parent_sel)?;
            apply_unaryop(op, &val, &SourcePos::default())
        }
        Expr::FunctionCall { name, args, namespace } => {
            eval_function_call(name, args, namespace.as_deref(), env, parent_sel)
        }
        Expr::Paren(inner) => eval_expr(inner, env, parent_sel),
        Expr::ListExpr { items, separator, bracketed } => {
            let mut values = Vec::new();
            for item in items {
                values.push(eval_expr(item, env, parent_sel)?);
            }
            Ok(Value::List(crate::value::SassList::new(values, separator.clone(), *bracketed)))
        }
        Expr::MapExpr(entries) => {
            let mut map = crate::value::SassMap::new();
            for (k, v) in entries {
                let key = eval_expr(k, env, parent_sel)?;
                let val = eval_expr(v, env, parent_sel)?;
                map.insert(key, val);
            }
            Ok(Value::Map(map))
        }
        Expr::Interpolation(parts) => {
            let mut result = String::new();
            for part in parts {
                match part {
                    InterpPart::Literal(s) => result.push_str(s),
                    InterpPart::Expr(e) => {
                        let val = eval_expr(e, env, parent_sel)?;
                        result.push_str(&val.to_string());
                    }
                }
            }
            Ok(Value::String(SassString::unquoted(result)))
        }
        Expr::ParentSelector => {
            match parent_sel.last() {
                Some(s) => Ok(Value::String(SassString::unquoted(s.clone()))),
                None => Ok(Value::Null),
            }
        }
    }
}

fn eval_function_call(
    name: &str,
    args: &[Arg],
    namespace: Option<&str>,
    env: &mut Env,
    parent_sel: &[String],
) -> Result<Value, SassError> {
    if let Some(ns) = namespace {
        if let Some(func) = env.get_module_function(ns, name).cloned() {
            return call_user_function(&func, args, env, parent_sel);
        }
        return Err(SassError::eval(
            format!("Function not found: {}.{}", ns, name),
            SourcePos::default(),
        ));
    }

    if let Some(func) = env.get_function(name).cloned() {
        return call_user_function(&func, args, env, parent_sel);
    }

    if let Some(builtin) = env.get_builtin(name).copied() {
        return builtin(args, env);
    }

    // Unknown function — return as unquoted string
    let mut parts = Vec::new();
    for arg in args {
        let val = eval_expr(&arg.value, env, parent_sel)?;
        parts.push(val.to_string());
    }
    Ok(Value::String(SassString::unquoted(format!(
        "{}({})", name, parts.join(", ")
    ))))
}

fn call_user_function(
    func: &crate::env::UserFunction,
    args: &[Arg],
    env: &mut Env,
    parent_sel: &[String],
) -> Result<Value, SassError> {
    // Save current env state, create child
    let mut func_env = Env::new_child(std::mem::replace(env, Env::new_global()));
    bind_params(&func.params, args, &mut func_env, env, parent_sel)?;

    let mut result = Value::Null;
    for stmt in &func.body {
        if let Stmt::ReturnStmt(expr) = stmt {
            result = eval_expr(expr, &mut func_env, parent_sel)?;
            break;
        }
    }

    // Restore env
    *env = *func_env.parent.take().unwrap();
    Ok(result)
}

pub fn bind_params(
    params: &[Param],
    args: &[Arg],
    func_env: &mut Env,
    env: &mut Env,
    parent_sel: &[String],
) -> Result<(), SassError> {
    for (i, param) in params.iter().enumerate() {
        if param.rest {
            let mut items = Vec::new();
            for arg in args.iter().skip(i) {
                items.push(eval_expr(&arg.value, env, parent_sel)?);
            }
            func_env.set_var(param.name.clone(), Value::List(crate::value::SassList::new(items, ListSeparator::Comma, false)), false, false);
            break;
        }

        let val = args.iter().find(|a| a.name.as_deref() == Some(param.name.as_str()))
            .or_else(|| args.get(i))
            .map(|a| &a.value);

        let value = if let Some(expr) = val {
            eval_expr(expr, env, parent_sel)?
        } else if let Some(default) = &param.default {
            eval_expr(default, env, parent_sel)?
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
) -> Result<Value, SassError> {
    call_user_function(func, args, env, parent_sel)
}
