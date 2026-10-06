//! Evaluator — dispatches AST nodes to their `SassOp` implementations.

use std::sync::Arc;

use rxrust::prelude::*;
use tracing::info_span;

use crate::reactive::{AstNode, AstStream, CssStream, EvalContext, SassOp, ScopeId};

/// Evaluate an AST stream by dispatching each node to its `SassOp`.
///
/// `flat_map` merges each node's operator output into a single `AstStream`.
/// After dispatch, remaining `AstNode::Css` variants are extracted into a
/// `CssStream` for downstream consumption via `lower_to_css`.
pub fn evaluate(stream: AstStream, ctx: Arc<EvalContext>) -> AstStream {
    let span = info_span!("evaluator", scope_id = ctx.scope_id);
    let _guard = span.enter();

    stream
        .flat_map(move |node| {
            let node_span = info_span!("visit", scope_id = ctx.scope_id);
            let _guard = node_span.enter();
            let _ = node_span;
            let op = node.into_operator(ctx.clone());
            op
        })
        .box_it()
}

/// Lower an evaluated `AstStream` to a `CssStream`.
///
/// Filters out non-CSS nodes (intermediate forms like remaining
/// `AstNode::VariableDecl` or block nodes that have been processed) and
/// unwraps `AstNode::Css(stmt)` into `stmt`.
pub fn lower_to_css(stream: AstStream) -> CssStream {
    stream
        .filter_map(move |node| match node {
            AstNode::Css(stmt) => Some(stmt),
            _ => None,
        })
        .box_it()
}

/// Convenience: evaluate + lower in one step.
pub fn evaluate_to_css(stream: AstStream, ctx: Arc<EvalContext>) -> CssStream {
    lower_to_css(evaluate(stream, ctx))
}

/// Scope pre-analysis pass.
///
/// Assigns monotonically increasing `scope_id` values to AST nodes as they
/// enter nested contexts. Format: `parent_idx * 1000 + local_counter`.
pub fn pre_analysis(items: Vec<AstNode>, parent_scope: ScopeId, counter: &mut u64) -> Vec<AstNode> {
    let span = info_span!("pre_analysis", parent_scope);
    let _guard = span.enter();

    items
        .into_iter()
        .map(|node| assign_scope(node, parent_scope, counter))
        .collect()
}

fn assign_scope(node: AstNode, parent: ScopeId, counter: &mut u64) -> AstNode {
    match node {
        AstNode::For { var, from, through, body } => {
            *counter += 1;
            let scope_id = parent * 1000 + *counter;
            let mut inner_counter = 0;
            let analyzed_body = pre_analysis(body, scope_id, &mut inner_counter);
            AstNode::For { var, from, through, body: analyzed_body }
        }
        AstNode::If { cond, then_branch, else_branch } => {
            *counter += 1;
            let scope_id = parent * 1000 + *counter;
            let mut c_then = 0;
            let mut c_else = 0;
            AstNode::If {
                cond,
                then_branch: pre_analysis(then_branch, scope_id, &mut c_then),
                else_branch: pre_analysis(else_branch, scope_id, &mut c_else),
            }
        }
        AstNode::RuleSet { selector, inner } => {
            *counter += 1;
            let scope_id = parent * 1000 + *counter;
            let mut inner_counter = 0;
            let inner = pre_analysis(inner, scope_id, &mut inner_counter);
            AstNode::RuleSet { selector, inner }
        }
        other => other,
    }
}
