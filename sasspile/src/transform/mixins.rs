//! Mixin expansion — @include resolves to mixin body with parameter binding.
//!
//! Handles parameter passing, @content replacement, parent selector
//! propagation, and recursion detection.

use tracing::instrument;

use crate::eval::EvalContext;
use crate::parser::{Expr, Node, Rule};
use crate::semantic::SymbolEntry;
use crate::{Result, SassError};

use super::{TransformCtx, MAX_CALL_DEPTH};

/// Expand an @include directive at the current position.
#[instrument(skip(ctx, name, args, include_body))]
pub(crate) fn expand_include(
    ctx: &mut TransformCtx,
    name: &str,
    args: &[Expr],
    include_body: &[Node],
) -> Result<Vec<Node>> {
    // Recursion depth check.
    if ctx.call_depth >= MAX_CALL_DEPTH {
        return Err(SassError::Compile(format!(
            "maximum call depth ({MAX_CALL_DEPTH}) exceeded for mixin '{name}'"
        )));
    }

    // Look up the mixin definition.
    let mixin_entry = ctx
        .definitions
        .get_mixin(name)
        .ok_or_else(|| SassError::Compile(format!("undefined mixin: {name}")))?
        .clone();

    // Check arity (positional only).
    let required = mixin_entry.required_params;
    let total = mixin_entry.total_params;
    if args.len() < required {
        return Err(SassError::Compile(format!(
            "mixin '{name}' requires at least {required} arguments, got {}",
            args.len()
        )));
    }
    if args.len() > total {
        return Err(SassError::Compile(format!(
            "mixin '{name}' takes at most {total} arguments, got {}",
            args.len()
        )));
    }

    // Evaluate arguments.
    let arg_values = {
        let mut eval_ctx = EvalContext::new(&mut ctx.symbols, &ctx.definitions);
        let vals: std::result::Result<Vec<_>, _> =
            args.iter().map(|e| eval_ctx.eval_expr(e)).collect();
        vals.map_err(|e| SassError::Compile(format!("arg eval: {e}")))?
    };

    // Evaluate default values for missing args.
    let mut bound_values = Vec::new();
    for (i, param) in mixin_entry.definition.params.iter().enumerate() {
        if let Some(val) = arg_values.get(i) {
            bound_values.push(val.clone());
        } else if let Some(default) = &param.default {
            let mut eval_ctx = EvalContext::new(&mut ctx.symbols, &ctx.definitions);
            let default_val = eval_ctx
                .eval_expr(default)
                .map_err(|e| SassError::Compile(format!("default eval: {e}")))?;
            bound_values.push(default_val);
        } else {
            bound_values.push(crate::value::Value::Null);
        }
    }

    // Push param scope.
    ctx.symbols.push_param();

    // Bind parameters.
    for (i, param) in mixin_entry.definition.params.iter().enumerate() {
        let value = bound_values.get(i).cloned().unwrap_or(crate::value::Value::Null);
        let param_name = param.name.trim_start_matches('$').to_string();
        let entry = SymbolEntry::new(Some(value), crate::source::SourceSpan::new(0, 0));
        ctx.symbols.define_current(param_name, entry);
    }

    // Expand the mixin body with @content replacement.
    let expanded = expand_mixin_body(ctx, &mixin_entry.definition.body, include_body)?;

    // Pop param scope.
    ctx.symbols.pop();

    Ok(expanded)
}

/// Expand the mixin body, replacing @content with the include body.
fn expand_mixin_body(
    ctx: &mut TransformCtx,
    body: &[Node],
    include_body: &[Node],
) -> Result<Vec<Node>> {
    expand_nodes_with_content(ctx, body, include_body)
}

/// Expand nodes within a mixin context, replacing @content with include_body.
fn expand_nodes_with_content(
    ctx: &mut TransformCtx,
    nodes: &[Node],
    include_body: &[Node],
) -> Result<Vec<Node>> {
    let mut result = Vec::new();

    for node in nodes {
        match node {
            // @content — replace with the include body passed to @include.
            Node::AtRule(crate::parser::AtRule::Content) => {
                ctx.call_depth += 1;
                let expanded_content = expand_nodes_with_content(ctx, include_body, include_body)?;
                ctx.call_depth -= 1;
                result.extend(expanded_content);
            }
            // Nested @include — recurse with depth tracking.
            Node::AtRule(crate::parser::AtRule::Include(inner_include)) => {
                ctx.call_depth += 1;
                let expanded = expand_include(
                    ctx,
                    &inner_include.name,
                    &inner_include.args,
                    &inner_include.body,
                )?;
                ctx.call_depth -= 1;
                result.extend(expanded);
            }
            // Nested rule — expand selector and recursively process body with @content handling.
            Node::Rule(rule) => {
                let selector = super::expand::expand_selector(&rule.selector, ctx)?;
                ctx.symbols.push_local();
                let expanded_nodes = expand_nodes_with_content(ctx, &rule.nodes, include_body)?;
                ctx.symbols.pop();
                result.push(Node::Rule(Rule {
                    selector,
                    nodes: expanded_nodes,
                }));
            }
            // Recursively expand other nodes using the generic expander.
            _ => {
                let expanded_one = super::expand::expand_nodes(ctx, std::slice::from_ref(node))?;
                result.extend(expanded_one);
            }
        }
    }

    Ok(result)
}
