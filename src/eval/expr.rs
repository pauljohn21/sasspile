//! Expression evaluator — evaluates AST expressions to Values.
//!
//! CSS calculation functions (`calc`, `min`, `max`, `clamp`) are handled
//! specially: their arguments are serialized to CSS strings *without*
//! performing Sass arithmetic, so that e.g. `calc(#{$w} * 2)` produces
//! `calc(var(--bs-border-width) * 2)` rather than trying to multiply a
//! string by a number.

use crate::ast::*;
use crate::env::Env;
use crate::error::{SassError, SourcePos};
use crate::operators::{apply_binop, apply_unaryop};
use crate::value::{SassString, Value};

/// CSS calculation function names that should not have their arguments
/// evaluated as Sass arithmetic.
const CSS_CALC_FUNCS: &[&str] = &["calc", "min", "max", "clamp"];

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
            // Short-circuit evaluation for And/Or
            match op {
                BinOp::And => {
                    let lv = eval_expr(left, env, parent_sel)?;
                    if lv.is_truthy() {
                        eval_expr(right, env, parent_sel)
                    } else {
                        Ok(lv)
                    }
                }
                BinOp::Or => {
                    let lv = eval_expr(left, env, parent_sel)?;
                    if lv.is_truthy() {
                        Ok(lv)
                    } else {
                        eval_expr(right, env, parent_sel)
                    }
                }
                _ => {
                    let lv = eval_expr(left, env, parent_sel)?;
                    let rv = eval_expr(right, env, parent_sel)?;
                    apply_binop(op, &lv, &rv, &SourcePos::default())
                }
            }
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
        Expr::NamespacedVariable { namespace, name } => {
            env.get_module_var(namespace, name).cloned().ok_or_else(|| {
                SassError::eval(format!("Undefined variable: {}.${}", namespace, name), SourcePos::default())
            })
        }
    }
}

/// Lazily evaluate `if($condition, $if-true, $if-false)`.
/// Only the condition is evaluated first; then only the matching
/// branch is evaluated. This mirrors Dart Sass `if()` semantics and
/// avoids errors from the unused branch (e.g. `unit($value)` when
/// `$value` is not a number).
fn eval_if_lazy(args: &[Arg], env: &mut Env, parent_sel: &[String]) -> Result<Value, SassError> {
    let span = tracing::debug_span!("eval_if_lazy", stage = "eval", module = "if");
    let _enter = span.enter();

    if args.len() < 3 {
        return Err(SassError::eval(
            "if() requires 3 arguments: $condition, $if-true, $if-false",
            SourcePos::default(),
        ));
    }
    let cond = eval_expr(&args[0].value, env, parent_sel)?;
    if cond.is_truthy() {
        eval_expr(&args[1].value, env, parent_sel)
    } else {
        eval_expr(&args[2].value, env, parent_sel)
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
        // Fallback: built-in module functions registered as "ns-func"
        // (e.g. map.deep-merge → map-deep-merge, color.adjust → color-adjust)
        let builtin_name = format!("{}-{}", ns, name);
        if let Some(builtin) = env.get_builtin(&builtin_name).copied() {
            return builtin(args, env);
        }
        return Err(SassError::eval(
            format!("Function not found: {}.{}", ns, name),
            SourcePos::default(),
        ));
    }

    if let Some(func) = env.get_function(name).cloned() {
        return call_user_function(&func, args, env, parent_sel);
    }

    // Special-case `if()` for lazy evaluation: only evaluate the
    // branch that matches the condition, avoiding side-effects or
    // errors from the unused branch (e.g. unit($value) when $value
    // is not a number).
    if name == "if" && namespace.is_none() && env.get_builtin("if").is_some() {
        return eval_if_lazy(args, env, parent_sel);
    }

    if let Some(builtin) = env.get_builtin(name).copied() {
        return builtin(args, env);
    }

    // CSS calculation functions — serialize args to CSS without arithmetic
    if CSS_CALC_FUNCS.contains(&name) {
        let css_args: Vec<String> = args
            .iter()
            .map(|a| expr_to_css_string(&a.value, env, parent_sel))
            .collect();
        return Ok(Value::String(SassString::unquoted(format!(
            "{}({})", name, css_args.join(", ")
        ))));
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

/// Serialize an expression to a CSS string *without* performing Sass
/// arithmetic.  This is used for `calc()`, `min()`, `max()`, `clamp()`
/// arguments where operators like `*` and `+` should be preserved as
/// CSS syntax rather than evaluated.
///
/// Variables and interpolation are still resolved to their values.
fn expr_to_css_string(expr: &Expr, env: &mut Env, parent_sel: &[String]) -> String {
    match expr {
        Expr::Literal(v) => crate::eval::value_to_css(v),
        Expr::Variable(name) => {
            match env.get_var(name) {
                Some(v) => crate::eval::value_to_css(v),
                None => format!("${}", name),
            }
        }
        Expr::Operation { op, left, right } => {
            let op_str = match op {
                BinOp::Add => " + ",
                BinOp::Sub => " - ",
                BinOp::Mul => " * ",
                BinOp::Div => " / ",
                BinOp::Mod => " % ",
                BinOp::Eq => " == ",
                BinOp::NotEq => " != ",
                BinOp::Lt => " < ",
                BinOp::LtEq => " <= ",
                BinOp::Gt => " > ",
                BinOp::GtEq => " >= ",
                BinOp::And => " and ",
                BinOp::Or => " or ",
            };
            format!(
                "{}{}{}",
                expr_to_css_string(left, env, parent_sel),
                op_str,
                expr_to_css_string(right, env, parent_sel)
            )
        }
        Expr::UnaryOp { op, operand } => {
            let prefix = match op {
                UnaryOp::Neg => "-",
                UnaryOp::Not => "not ",
            };
            format!("{}{}", prefix, expr_to_css_string(operand, env, parent_sel))
        }
        Expr::FunctionCall { name, args, namespace } => {
            let inner: Vec<String> = args
                .iter()
                .map(|a| expr_to_css_string(&a.value, env, parent_sel))
                .collect();
            if let Some(ns) = namespace {
                format!("{}.{}({})", ns, name, inner.join(", "))
            } else {
                format!("{}({})", name, inner.join(", "))
            }
        }
        Expr::Paren(inner) => {
            format!("({})", expr_to_css_string(inner, env, parent_sel))
        }
        Expr::ListExpr { items, separator, bracketed } => {
            let sep = match separator {
                ListSeparator::Comma => ", ",
                ListSeparator::Slash => " / ",
                _ => " ",
            };
            let parts: Vec<String> = items
                .iter()
                .map(|e| expr_to_css_string(e, env, parent_sel))
                .collect();
            if *bracketed {
                format!("[{}]", parts.join(sep))
            } else {
                parts.join(sep)
            }
        }
        Expr::Interpolation(parts) => {
            let mut result = String::new();
            for part in parts {
                match part {
                    InterpPart::Literal(s) => result.push_str(s),
                    InterpPart::Expr(e) => {
                        let val = eval_expr(e, env, parent_sel);
                        match val {
                            Ok(v) => result.push_str(&crate::eval::value_to_css(&v)),
                            Err(_) => result.push_str(&expr_to_css_string(e, env, parent_sel)),
                        }
                    }
                }
            }
            result
        }
        Expr::MapExpr(entries) => {
            let parts: Vec<String> = entries
                .iter()
                .map(|(k, v)| format!(
                    "{}: {}",
                    expr_to_css_string(k, env, parent_sel),
                    expr_to_css_string(v, env, parent_sel)
                ))
                .collect();
            format!("({})", parts.join(", "))
        }
        Expr::ParentSelector => {
            parent_sel.last().cloned().unwrap_or_default()
        }
        Expr::NamespacedVariable { namespace, name } => {
            match env.get_module_var(namespace, name) {
                Some(v) => crate::eval::value_to_css(v),
                None => format!("{}.${}", namespace, name),
            }
        }
    }
}

fn call_user_function(
    func: &crate::env::UserFunction,
    args: &[Arg],
    env: &mut Env,
    parent_sel: &[String],
) -> Result<Value, SassError> {
    // Pre-evaluate all arguments in the *caller's* environment before
    // creating the function's child scope.  This is critical because
    // `std::mem::replace(env, …)` would otherwise leave `env` pointing
    // at a temporary empty environment while `bind_params` evaluates
    // argument expressions (e.g. `$blue` in `tint-color($blue, 80%)`).
    //
    // Also expand spread args ($val...) — maps become named args,
    // lists become positional args.
    let expanded = expand_spread_args(args, env, parent_sel)?;

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
                Value::List(crate::value::SassList::new(items, ListSeparator::Comma, false)),
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
            eval_expr(default, env, parent_sel)?
        } else {
            Value::Null
        };
        evaluated.push((param.name.clone(), value));
    }

    // Now create the child environment and bind the pre-evaluated values.
    let parent = std::mem::replace(env, Env::new_global());
    let mut func_env = Env::new_child(parent);
    for (name, value) in evaluated {
        func_env.set_var(name, value, false, false);
    }

    let mut result = Value::Null;
    let mut dummy_output: Vec<super::CssRule> = Vec::new();
    let mut dummy_extends: Vec<super::ExtendEntry> = Vec::new();
    for stmt in &func.body {
        if let Stmt::ReturnStmt(expr) = stmt {
            result = eval_expr(expr, &mut func_env, parent_sel)?;
            break;
        }
        // Execute non-return statements (variable declarations, @each, @if, etc.)
        super::eval_stmt(stmt, &mut func_env, parent_sel, &mut dummy_output, &mut dummy_extends)?;
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
) -> Result<Vec<(Option<String>, Value)>, SassError> {
    let span = tracing::debug_span!("expand_spread_args", stage = "eval", module = "args");
    let _enter = span.enter();
    let mut result: Vec<(Option<String>, Value)> = Vec::new();

    for arg in args {
        let val = eval_expr(&arg.value, env, parent_sel)?;
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
