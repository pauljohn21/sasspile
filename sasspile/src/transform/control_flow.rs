//! Control flow expansion — @if, @for, @each, @while.
//!
//! Evaluates conditions/loops at compile time using EvalContext
//! and produces expanded AST with the selected/iterated body.

use tracing::instrument;

use crate::eval::EvalContext;
use crate::parser::{Expr, Node};
use crate::value::Value;
use crate::{Result, SassError};

use super::{TransformCtx, MAX_LOOP_ITERATIONS};

/// Expand @if/@else: evaluate condition and keep the appropriate branch.
#[instrument(skip(ctx, condition, then_body, else_body))]
pub(crate) fn expand_if(
    ctx: &mut TransformCtx,
    condition: &Expr,
    then_body: &[Node],
    else_body: Option<&[Node]>,
) -> Result<Vec<Node>> {
    let cond_val = {
        let mut eval_ctx = EvalContext::new(&mut ctx.symbols, &ctx.definitions);
        eval_ctx
            .eval_expr(condition)
            .map_err(|e| SassError::Compile(format!("@if condition: {e}")))?
    };

    let cond_bool = match cond_val {
        Value::Boolean(b) => b,
        Value::Null => false,
        other => other.to_bool(),
    };

    if cond_bool {
        ctx.symbols.push_local();
        let expanded = super::expand::expand_nodes(ctx, then_body)?;
        ctx.symbols.pop();
        Ok(expanded)
    } else if let Some(else_body) = else_body {
        super::expand::expand_nodes(ctx, else_body)
    } else {
        Ok(vec![])
    }
}

/// Expand @for: iterate from start through/to end.
#[instrument(skip(ctx, var, start, end, body))]
pub(crate) fn expand_for(
    ctx: &mut TransformCtx,
    var: &str,
    start: &Expr,
    end: &Expr,
    inclusive: bool,
    body: &[Node],
) -> Result<Vec<Node>> {
    let (start_val, end_val) = {
        let mut eval_ctx = EvalContext::new(&mut ctx.symbols, &ctx.definitions);
        let start_val = eval_ctx
            .eval_expr(start)
            .map_err(|e| SassError::Compile(format!("@for start: {e}")))?;
        let end_val = eval_ctx
            .eval_expr(end)
            .map_err(|e| SassError::Compile(format!("@for end: {e}")))?;
        (start_val, end_val)
    };

    let start_num = value_to_i64(&start_val)?;
    let end_num = value_to_i64(&end_val)?;

    let var_name = var.trim_start_matches('$');
    let mut result = Vec::new();

    if start_num <= end_num {
        let loop_end = if inclusive { end_num + 1 } else { end_num };
        for i in start_num..loop_end {
            result.extend(expand_for_iteration(ctx, var_name, i, body)?);
        }
    } else {
        // Reverse iteration.
        let loop_end = if inclusive { end_num - 1 } else { end_num };
        let mut i = start_num;
        while i >= loop_end {
            result.extend(expand_for_iteration(ctx, var_name, i, body)?);
            if i == 0 {
                break;
            }
            i -= 1;
        }
    }

    Ok(result)
}

fn expand_for_iteration(
    ctx: &mut TransformCtx,
    var_name: &str,
    value: i64,
    body: &[Node],
) -> Result<Vec<Node>> {
    // Push a scope for the loop variable.
    ctx.symbols.push_local();

    let entry = crate::semantic::SymbolEntry::mutable(
        Some(Value::Number(crate::value::Number::unitless(value as f64))),
        crate::source::SourceSpan::new(0, 0),
    );
    ctx.symbols.define_current(var_name.to_string(), entry);

    let expanded = super::expand::expand_nodes(ctx, body)?;

    ctx.symbols.pop();
    Ok(expanded)
}

/// Expand @each: iterate over a list.
#[instrument(skip(ctx, vars, list, body))]
pub(crate) fn expand_each(
    ctx: &mut TransformCtx,
    vars: &[String],
    list: &Expr,
    body: &[Node],
) -> Result<Vec<Node>> {
    let list_val = {
        let mut eval_ctx = EvalContext::new(&mut ctx.symbols, &ctx.definitions);
        eval_ctx
            .eval_expr(list)
            .map_err(|e| SassError::Compile(format!("@each list: {e}")))?
    };

    let items = match list_val {
        Value::List(items, _) => items,
        Value::Map(entries) => entries
            .into_iter()
            .map(|(k, v)| Value::List(vec![k, v], crate::value::Separator::Space))
            .collect(),
        single => vec![single],
    };

    let mut result = Vec::new();

    for item in items {
        ctx.symbols.push_local();

        if vars.len() == 1 {
            // Single variable — bind the whole item.
            let var_name = vars[0].trim_start_matches('$');
            let entry = crate::semantic::SymbolEntry::mutable(
                Some(item),
                crate::source::SourceSpan::new(0, 0),
            );
            ctx.symbols.define_current(var_name.to_string(), entry);
        } else {
            // Multiple variables — destructure a list.
            if let Value::List(parts, _) = &item {
                for (i, var) in vars.iter().enumerate() {
                    let val = parts.get(i).cloned().unwrap_or(Value::Null);
                    let var_name = var.trim_start_matches('$');
                    let entry = crate::semantic::SymbolEntry::mutable(
                        Some(val),
                        crate::source::SourceSpan::new(0, 0),
                    );
                    ctx.symbols.define_current(var_name.to_string(), entry);
                }
            } else {
                // Bind first var to the value, rest to null.
                let var_name = vars[0].trim_start_matches('$');
                let entry = crate::semantic::SymbolEntry::mutable(
                    Some(item),
                    crate::source::SourceSpan::new(0, 0),
                );
                ctx.symbols.define_current(var_name.to_string(), entry);
            }
        }

        let expanded = super::expand::expand_nodes(ctx, body)?;
        result.extend(expanded);

        ctx.symbols.pop();
    }

    Ok(result)
}

/// Expand @while: loop until condition is false.
#[instrument(skip(ctx, condition, body))]
pub(crate) fn expand_while(
    ctx: &mut TransformCtx,
    condition: &Expr,
    body: &[Node],
) -> Result<Vec<Node>> {
    let mut result = Vec::new();
    let mut iterations = 0;

    loop {
        if iterations >= MAX_LOOP_ITERATIONS {
            return Err(SassError::Compile(format!(
                "@while exceeded maximum iterations ({MAX_LOOP_ITERATIONS})"
            )));
        }

        let cond_val = {
            let mut eval_ctx = EvalContext::new(&mut ctx.symbols, &ctx.definitions);
            eval_ctx
                .eval_expr(condition)
                .map_err(|e| SassError::Compile(format!("@while condition: {e}")))?
        };

        let cond_bool = match cond_val {
            Value::Boolean(b) => b,
            Value::Null => false,
            other => other.to_bool(),
        };

        if !cond_bool {
            break;
        }

        ctx.symbols.push_local();
        let expanded = super::expand::expand_nodes(ctx, body)?;
        result.extend(expanded);
        ctx.symbols.pop();

        iterations += 1;
    }

    Ok(result)
}

/// Convert a Value to i64 for loop iteration.
fn value_to_i64(val: &Value) -> Result<i64> {
    match val {
        Value::Number(n) => Ok(n.value as i64),
        Value::String(s, _) => {
            s.parse::<i64>()
                .map_err(|_| SassError::Compile(format!(
                    "expected number for loop range, got string: {s}"
                )))
        }
        _ => Err(SassError::Compile(format!(
            "expected number for loop range, got: {:?}",
            val.type_name()
        ))),
    }
}
