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
use crate::resolver::ModuleResolver;
use crate::value::{SassString, Value};
use super::func::call_user_function;

/// CSS calculation function names that should not have their arguments
/// evaluated as Sass arithmetic.
const CSS_CALC_FUNCS: &[&str] = &["calc", "min", "max", "clamp"];

/// Evaluate an expression to a Value.
pub fn eval_expr(
    expr: &Expr,
    env: &mut Env,
    parent_sel: &[String],
    resolver: &mut dyn ModuleResolver,
) -> Result<Value, SassError> {
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
                    let lv = eval_expr(left, env, parent_sel, resolver)?;
                    if lv.is_truthy() {
                        eval_expr(right, env, parent_sel, resolver)
                    } else {
                        Ok(lv)
                    }
                }
                BinOp::Or => {
                    let lv = eval_expr(left, env, parent_sel, resolver)?;
                    if lv.is_truthy() {
                        Ok(lv)
                    } else {
                        eval_expr(right, env, parent_sel, resolver)
                    }
                }
                _ => {
                    let lv = eval_expr(left, env, parent_sel, resolver)?;
                    let rv = eval_expr(right, env, parent_sel, resolver)?;
                    apply_binop(op, &lv, &rv, &SourcePos::default())
                }
            }
        }
        Expr::UnaryOp { op, operand } => {
            let val = eval_expr(operand, env, parent_sel, resolver)?;
            apply_unaryop(op, &val, &SourcePos::default())
        }
        Expr::FunctionCall { name, args, namespace } => {
            eval_function_call(name, args, namespace.as_deref(), env, parent_sel, resolver)
        }
        Expr::Paren(inner) => eval_expr(inner, env, parent_sel, resolver),
        Expr::ListExpr { items, separator, bracketed } => {
            let mut values = Vec::new();
            for item in items {
                values.push(eval_expr(item, env, parent_sel, resolver)?);
            }
            Ok(Value::List(crate::value::SassList::new(values, separator.clone(), *bracketed)))
        }
        Expr::MapExpr(entries) => {
            let mut map = crate::value::SassMap::new();
            for (k, v) in entries {
                let key = eval_expr(k, env, parent_sel, resolver)?;
                let val = eval_expr(v, env, parent_sel, resolver)?;
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
                        let val = eval_expr(e, env, parent_sel, resolver)?;
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
fn eval_if_lazy(
    args: &[Arg],
    env: &mut Env,
    parent_sel: &[String],
    resolver: &mut dyn ModuleResolver,
) -> Result<Value, SassError> {
    let span = tracing::debug_span!("eval_if_lazy", stage = "eval", module = "if");
    let _enter = span.enter();

    if args.len() < 3 {
        return Err(SassError::eval(
            "if() requires 3 arguments: $condition, $if-true, $if-false",
            SourcePos::default(),
        ));
    }
    let cond = eval_expr(&args[0].value, env, parent_sel, resolver)?;
    if cond.is_truthy() {
        eval_expr(&args[1].value, env, parent_sel, resolver)
    } else {
        eval_expr(&args[2].value, env, parent_sel, resolver)
    }
}

fn eval_function_call(
    name: &str,
    args: &[Arg],
    namespace: Option<&str>,
    env: &mut Env,
    parent_sel: &[String],
    resolver: &mut dyn ModuleResolver,
) -> Result<Value, SassError> {
    if let Some(ns) = namespace {
        if let Some(func) = env.get_module_function(ns, name).cloned() {
            return call_user_function(&func, args, env, parent_sel, resolver);
        }
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
        return call_user_function(&func, args, env, parent_sel, resolver);
    }

    if name == "if" && namespace.is_none() && env.get_builtin("if").is_some() {
        return eval_if_lazy(args, env, parent_sel, resolver);
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
        let val = eval_expr(&arg.value, env, parent_sel, resolver)?;
        parts.push(val.to_string());
    }
    Ok(Value::String(SassString::unquoted(format!(
        "{}({})", name, parts.join(", ")
    ))))
}

/// Serialize an expression to a CSS string *without* performing Sass
/// arithmetic.  Used for `calc()`, `min()`, `max()`, `clamp()` arguments.
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
                        result.push_str(&expr_to_css_string(e, env, parent_sel));
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
