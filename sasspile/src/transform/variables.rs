//! Variable collection and replacement.
//!
//! Handles SCSS variable declarations (`$name: value`), scope-aware
//! replacement of `Expr::Variable` nodes, and interpolation resolution.

use tracing::instrument;

use crate::eval::EvalContext;
use crate::parser::{AtRule, Expr, Node};
use crate::semantic::SymbolEntry;
use crate::{Result, SassError};

use super::TransformCtx;

/// Collect variable definitions from a node list.
#[instrument(skip(ctx, nodes))]
pub(crate) fn collect_definitions(ctx: &mut TransformCtx, nodes: &[Node]) -> Result<()> {
    for node in nodes {
        collect_from_node(ctx, node)?;
    }
    Ok(())
}

/// Collect variable defs from a single node (recursively into rules).
fn collect_from_node(ctx: &mut TransformCtx, node: &Node) -> Result<()> {
    match node {
        Node::Declaration(decl) => {
            if decl.is_variable {
                let var_name = &decl.name;
                let mut eval_ctx = EvalContext::new(&mut ctx.symbols, &ctx.definitions);
                let value = eval_ctx
                    .eval_expr(&decl.value)
                    .map_err(|e| SassError::Compile(format!("variable eval error: {e}")))?;
                let entry = SymbolEntry::mutable(Some(value), decl.span);
                ctx.symbols.define_current(var_name.to_string(), entry);
            }
        }
        Node::Rule(rule) => {
            ctx.symbols.push_local();
            for inner in &rule.nodes {
                collect_from_node(ctx, inner)?;
            }
            ctx.symbols.pop();
        }
        Node::AtRule(AtRule::Media(media)) => {
            ctx.symbols.push_local();
            for node in &media.body {
                collect_from_node(ctx, node)?;
            }
            ctx.symbols.pop();
        }
        Node::AtRule(AtRule::Supports(supports)) => {
            ctx.symbols.push_local();
            for node in &supports.body {
                collect_from_node(ctx, node)?;
            }
            ctx.symbols.pop();
        }
        Node::AtRule(AtRule::If(stmt)) => {
            ctx.symbols.push_local();
            for node in &stmt.body {
                collect_from_node(ctx, node)?;
            }
            ctx.symbols.pop();
            if let Some(else_body) = &stmt.else_body {
                ctx.symbols.push_local();
                for node in else_body {
                    collect_from_node(ctx, node)?;
                }
                ctx.symbols.pop();
            }
        }
        Node::AtRule(AtRule::For(stmt)) => {
            for node in &stmt.body {
                collect_from_node(ctx, node)?;
            }
        }
        Node::AtRule(AtRule::Each(stmt)) => {
            for node in &stmt.body {
                collect_from_node(ctx, node)?;
            }
        }
        Node::AtRule(AtRule::While(stmt)) => {
            for node in &stmt.body {
                collect_from_node(ctx, node)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Evaluate a variable declaration value and store it in the symbol table.
pub(crate) fn eval_and_store(
    ctx: &mut TransformCtx,
    var_name: &str,
    value_expr: &Expr,
) -> Result<crate::value::Value> {
    let mut eval_ctx = EvalContext::new(&mut ctx.symbols, &ctx.definitions);
    let value = eval_ctx
        .eval_expr(value_expr)
        .map_err(|e| SassError::Compile(format!("variable '${var_name}' eval error: {e}")))?;
    let entry = SymbolEntry::mutable(Some(value.clone()), crate::source::SourceSpan::new(0, 0));
    ctx.symbols.define_current(var_name.to_string(), entry);
    Ok(value)
}

/// Expand an expression: replace all variable references with constant values.
pub(crate) fn expand_expr(ctx: &mut TransformCtx, expr: &Expr) -> Result<Expr> {
    match expr {
        Expr::Variable(name) => {
            // Variable names from the parser include the $ prefix.
            let lookup_name = name.trim_start_matches('$');
            match ctx.symbols.lookup(lookup_name) {
                Some(entry) => {
                    match &entry.value {
                        Some(val) => value_to_expr(val.clone()),
                        None => Err(SassError::Compile(format!(
                            "variable '${name}' has no value"
                        ))),
                    }
                }
                None => Err(SassError::Compile(format!(
                    "undefined variable: ${name}"
                ))),
            }
        }
        Expr::Interpolation(inner) => {
            let expanded = expand_expr(ctx, inner)?;
            // If the inner expression evaluates to a simple value, unwrap.
            match expanded {
                Expr::String(s) => Ok(Expr::String(s)),
                Expr::Number(v, u) => Ok(Expr::String(format_number(v, &u))),
                other => Ok(other),
            }
        }
        Expr::Binary(op, lhs, rhs) => {
            let l = expand_expr(ctx, lhs)?;
            let r = expand_expr(ctx, rhs)?;
            // Try constant folding for numeric ops.
            match (&l, &r) {
                (Expr::Number(a, u1), Expr::Number(b, u2)) if u1 == u2 => {
                    match op {
                        crate::parser::BinaryOp::Add => {
                            Ok(Expr::Number(a + b, u1.clone()))
                        }
                        crate::parser::BinaryOp::Sub => {
                            Ok(Expr::Number(a - b, u1.clone()))
                        }
                        crate::parser::BinaryOp::Mul => {
                            Ok(Expr::Number(a * b, u1.clone()))
                        }
                        crate::parser::BinaryOp::Div if *b != 0.0 => {
                            Ok(Expr::Number(a / b, u1.clone()))
                        }
                        _ => Ok(Expr::Binary(*op, Box::new(l), Box::new(r))),
                    }
                }
                _ => Ok(Expr::Binary(*op, Box::new(l), Box::new(r))),
            }
        }
        Expr::Unary(op, operand) => {
            let v = expand_expr(ctx, operand)?;
            match (&op, &v) {
                (crate::parser::UnaryOp::Neg, Expr::Number(n, u)) => {
                    Ok(Expr::Number(-n, u.clone()))
                }
                _ => Ok(Expr::Unary(*op, Box::new(v))),
            }
        }
        Expr::Call(name, args) => {
            let expanded_args: Result<Vec<Expr>> =
                args.iter().map(|a| expand_expr(ctx, a)).collect();
            // Try to evaluate the call to a constant if possible.
            match name.as_str() {
                "rgb" | "rgba" => eval_rgb_call(expanded_args?),
                _ => {
                    let mut eval_ctx = EvalContext::new(&mut ctx.symbols, &ctx.definitions);
                    match eval_ctx.eval_expr(expr) {
                        Ok(val) => value_to_expr(val),
                        Err(_) => Ok(Expr::Call(name.clone(), expanded_args?)),
                    }
                }
            }
        }
        Expr::List(items) => {
            let expanded: Result<Vec<Expr>> =
                items.iter().map(|i| expand_expr(ctx, i)).collect();
            Ok(Expr::List(expanded?))
        }
        Expr::SpaceList(items) => {
            let expanded: Result<Vec<Expr>> =
                items.iter().map(|i| expand_expr(ctx, i)).collect();
            Ok(Expr::SpaceList(expanded?))
        }
        Expr::SlashList(items) => {
            let expanded: Result<Vec<Expr>> =
                items.iter().map(|i| expand_expr(ctx, i)).collect();
            Ok(Expr::SlashList(expanded?))
        }
        Expr::Map(entries) => {
            let expanded: Result<Vec<(Expr, Expr)>> = entries
                .iter()
                .map(|(k, v)| Ok((expand_expr(ctx, k)?, expand_expr(ctx, v)?)))
                .collect();
            Ok(Expr::Map(expanded?))
        }
        Expr::Parens(inner) => {
            let expanded = expand_expr(ctx, inner)?;
            Ok(Expr::Parens(Box::new(expanded)))
        }
        Expr::NamedArg(name, value) => {
            let expanded = expand_expr(ctx, value)?;
            Ok(Expr::NamedArg(name.clone(), Box::new(expanded)))
        }
        Expr::Spread(inner) => {
            let expanded = expand_expr(ctx, inner)?;
            Ok(Expr::Spread(Box::new(expanded)))
        }
        // Passthrough for literals and bare identifiers.
        Expr::Number(..) | Expr::String(..) | Expr::Boolean(..) | Expr::Null | Expr::Color(..)
        | Expr::Url(..) | Expr::Identifier(..) => Ok(expr.clone()),
    }
}

/// Convert a Value back to an AST Expr for substitution.
fn value_to_expr(val: crate::value::Value) -> Result<Expr> {
    match val {
        crate::value::Value::Number(n) => {
            let unit = match &n.unit {
                crate::value::Unit::None => None,
                other => Some(format!("{other:?}").to_lowercase()),
            };
            Ok(Expr::Number(n.value, unit))
        }
        crate::value::Value::String(s, _) => Ok(Expr::String(s)),
        crate::value::Value::Boolean(b) => Ok(Expr::Boolean(b)),
        crate::value::Value::Null => Ok(Expr::Null),
        crate::value::Value::Color(c) => {
            // Convert SassColor back to Color literal if opaque.
            if c.is_opaque() {
                let rgb = (c.r as u32) << 16 | (c.g as u32) << 8 | (c.b as u32);
                Ok(Expr::Color(rgb))
            } else {
                Ok(Expr::String(c.to_string()))
            }
        }
        crate::value::Value::List(items, sep) => {
            let expanded: Result<Vec<Expr>> = items.into_iter().map(value_to_expr).collect();
            match sep {
                crate::value::Separator::Space => Ok(Expr::SpaceList(expanded?)),
                crate::value::Separator::Slash => Ok(Expr::SlashList(expanded?)),
                _ => Ok(Expr::List(expanded?)),
            }
        }
        crate::value::Value::Map(entries) => {
            let expanded: Result<Vec<(Expr, Expr)>> = entries
                .into_iter()
                .map(|(k, v)| Ok((value_to_expr(k)?, value_to_expr(v)?)))
                .collect();
            Ok(Expr::Map(expanded?))
        }
        _ => Err(SassError::Compile(format!(
            "cannot convert value to expr: {val:?}"
        ))),
    }
}

/// Format a number for interpolation.
fn format_number(val: f64, unit: &Option<String>) -> String {
    if val.fract().abs() < f64::EPSILON {
        // Integer-like.
        let int_val = val as i64;
        match unit {
            Some(u) => format!("{int_val}{u}"),
            None => format!("{int_val}"),
        }
    } else {
        match unit {
            Some(u) => format!("{val}{u}"),
            None => format!("{val}"),
        }
    }
}

/// Try to evaluate rgb/rgba function calls.
fn eval_rgb_call(args: Vec<Expr>) -> Result<Expr> {
    if args.len() == 3
        && let (Expr::Number(r, _), Expr::Number(g, _), Expr::Number(b, _)) =
            (&args[0], &args[1], &args[2])
    {
        let r = r.clamp(0.0, 255.0) as u32;
        let g = g.clamp(0.0, 255.0) as u32;
        let b = b.clamp(0.0, 255.0) as u32;
        return Ok(Expr::Color((r << 16) | (g << 8) | b));
    }
    Ok(Expr::Call("rgb".to_string(), args))
}
