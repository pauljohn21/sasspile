//! Single `SassOp for AstNode` implementation — dispatches each AST node
//! variant to its operator logic.
//!
//! Every branch returns an `AstStream` (= `SharedBoxedObservable<AstNode>`).
//! Most branches build a `Vec<AstNode>` with native Rust iterator combinators
//! and lift it via `Shared::from_iter`. Only event-style operations use
//! `Shared::create` directly. All types are `Send + Sync`.

use std::sync::Arc;

use rxrust::prelude::*;
use tracing::{debug_span, info_span};

use crate::reactive::pipeline::compile::collect_css;
use crate::reactive::{AstNode, AstStream, CssStream, EvalContext, FnDef, MixinDef, SassOp};

const MAX_WHILE_ITERATIONS: i64 = 10_000;

// ─────────────────────────── empty stream helper ────────────────────────────

fn empty_ast_stream() -> AstStream {
    Shared::create(|subscriber| {
        subscriber.complete();
    })
    .box_it()
}

// ─────────────────────────── SassOp implementation ───────────────────────────

impl SassOp for AstNode {
    fn into_operator(self, ctx: Arc<EvalContext>) -> AstStream {
        let span = info_span!("sass_op", node = ?self, scope_id = ctx.scope_id);
        let _guard = span.enter();

        match self {
            // VariableDecl: emit Bind event, return empty (no CSS output)
            AstNode::VariableDecl { name, value } => {
                let span = debug_span!("variable_decl", %name, scope_id = ctx.scope_id);
                let _guard = span.enter();
                let scope_id = ctx.scope_id;
                let value_num = match value {
                    crate::reactive::Value::Number(n) => n as u64,
                    _ => 0,
                };
                let mut var_subject = ctx.bus.var_events();
                var_subject.next(crate::reactive::ValueEvent::Bind {
                    scope_id,
                    name: name.clone(),
                    value: value_num,
                });
                tracing::info!(%name, value = value_num, "variable bind");
                empty_ast_stream()
            }

            // StyleDecl: directly lower to CssStmt::Decl
            AstNode::StyleDecl { property, value } => {
                let span = debug_span!("style_decl", %property, %value, scope_id = ctx.scope_id);
                let _guard = span.enter();
                Shared::of(AstNode::Css(crate::reactive::CssStmt::Decl { property, value }))
                    .box_it()
            }

            // RuleSet: evaluate inner nodes, collect CSS, wrap in Rule
            AstNode::RuleSet { selector, inner } => {
                let span = debug_span!("rule_set", %selector, scope_id = ctx.scope_id);
                let _guard = span.enter();
                let child_ctx = Arc::new(ctx.child_scope(1));
                let css_vec = extract_css(&inner, child_ctx);
                Shared::of(AstNode::Css(crate::reactive::CssStmt::Rule {
                    selector,
                    inner: css_vec,
                }))
                .box_it()
            }

            // @media: evaluate inner nodes, wrap in Media
            AstNode::Media { query, inner } => {
                let span = debug_span!("media", %query, scope_id = ctx.scope_id);
                let _guard = span.enter();
                let child_ctx = Arc::new(ctx.child_scope(1));
                let css_vec = extract_css(&inner, child_ctx);
                Shared::of(AstNode::Css(crate::reactive::CssStmt::Media {
                    query,
                    inner: css_vec,
                }))
                .box_it()
            }

            // @supports: evaluate inner nodes, wrap in Supports
            AstNode::Supports { query, inner } => {
                let span = debug_span!("supports", %query, scope_id = ctx.scope_id);
                let _guard = span.enter();
                let child_ctx = Arc::new(ctx.child_scope(1));
                let css_vec = extract_css(&inner, child_ctx);
                Shared::of(AstNode::Css(crate::reactive::CssStmt::Supports {
                    query,
                    inner: css_vec,
                }))
                .box_it()
            }

            // @if: select branch, expand to Vec, then from_iter
            AstNode::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let span = debug_span!("if", scope_id = ctx.scope_id);
                let _guard = span.enter();
                let child_ctx = Arc::new(ctx.child_scope(1));
                let selected = if eval_cond(&cond) {
                    tracing::debug!("selecting then branch");
                    then_branch
                } else {
                    tracing::debug!("selecting else branch");
                    else_branch
                };
                let css_vec = extract_css(&selected, child_ctx);
                Shared::from_iter(css_vec.into_iter().map(AstNode::Css)).box_it()
            }

            // @for: flat_map over range, collect to Vec, then from_iter
            AstNode::For {
                from, through, body, ..
            } => {
                let span = debug_span!("for", %from, %through, scope_id = ctx.scope_id);
                let _guard = span.enter();
                let child_ctx = Arc::new(ctx.child_scope(1));
                let iterations = ((through - from).round() as i64 + 1).max(0);

                let nodes: Vec<AstNode> = (0..iterations)
                    .flat_map(|_| {
                        extract_css(&body, child_ctx.clone())
                            .into_iter()
                            .map(AstNode::Css)
                    })
                    .collect();
                Shared::from_iter(nodes).box_it()
            }

            // @each: flat_map over list (list resolution pending)
            AstNode::Each { .. } => {
                let span = debug_span!("each", scope_id = ctx.scope_id);
                let _guard = span.enter();
                tracing::debug!("each: pass-through (list resolution pending)");
                empty_ast_stream()
            }

            // @while: bounded iteration via from_iter
            AstNode::While { cond, body } => {
                let span = debug_span!("while", scope_id = ctx.scope_id);
                let _guard = span.enter();
                let child_ctx = Arc::new(ctx.child_scope(1));

                let nodes: Vec<AstNode> = std::iter::repeat_with(|| {
                    extract_css(&body, child_ctx.clone())
                        .into_iter()
                        .map(AstNode::Css)
                })
                .take(MAX_WHILE_ITERATIONS as usize)
                .take_while(|_| eval_cond(&cond))
                .flatten()
                .collect();
                Shared::from_iter(nodes).box_it()
            }

            // @warn: tap — emit tracing event, no CSS output
            AstNode::Warn { message } => {
                let span = debug_span!("warn", %message, scope_id = ctx.scope_id);
                let _guard = span.enter();
                tracing::warn!(%message, scope_id = ctx.scope_id, "@warn directive");
                empty_ast_stream()
            }

            // @debug: tap — emit tracing event, no CSS output
            AstNode::Debug { expr: _, message } => {
                let span = debug_span!("debug", %message, scope_id = ctx.scope_id);
                let _guard = span.enter();
                tracing::debug!(%message, scope_id = ctx.scope_id, "@debug directive");
                empty_ast_stream()
            }

            // @mixin: register definition, no CSS output
            AstNode::Mixin { name, params, body } => {
                let span = debug_span!("mixin_def", %name, scope_id = ctx.scope_id);
                let _guard = span.enter();
                ctx.bus.register_mixin(MixinDef { name, params, body });
                empty_ast_stream()
            }

            // @include: expand registered mixin body, then from_iter
            AstNode::MixinCall { name, args } => {
                let span = debug_span!("mixin_call", %name, ?args, scope_id = ctx.scope_id);
                let _guard = span.enter();
                let child_ctx = Arc::new(ctx.child_scope(1));
                let expanded = match ctx.bus.lookup_mixin(&name) {
                    Some(def) => {
                        tracing::debug!(
                            params = ?def.params,
                            body_len = def.body.len(),
                            "mixin found"
                        );
                        let css_vec = extract_css(&def.body, child_ctx);
                        css_vec.into_iter().map(AstNode::Css).collect::<Vec<_>>()
                    }
                    None => {
                        tracing::warn!("mixin not found");
                        Vec::new()
                    }
                };
                Shared::from_iter(expanded).box_it()
            }

            // @use: load module via module_events
            AstNode::UseRule { path } => {
                let span = debug_span!("use_rule", %path, scope_id = ctx.scope_id);
                let _guard = span.enter();
                let mut module_subject = ctx.bus.module_events();
                module_subject.next(crate::reactive::ModuleEvent::Load { name: path });
                empty_ast_stream()
            }

            // @function: register callable function
            AstNode::FunctionDecl { name, params, body } => {
                let span = debug_span!("function_def", %name, scope_id = ctx.scope_id);
                let _guard = span.enter();
                ctx.bus.register_fn(FnDef { name, params, body });
                empty_ast_stream()
            }

            // Fallback: passthrough the node as-is
            other => {
                let span = debug_span!("fallback", node = ?other, scope_id = ctx.scope_id);
                let _guard = span.enter();
                tracing::trace!(node = ?other, "passthrough operator");
                Shared::of(other).box_it()
            }
        }
    }
}

// ─────────────────────────── Helper functions ──────────────────────────────

/// Simplified condition evaluation.
///
/// Treats `AstNode::Placeholder` as false (default/nodes without
/// explicit boolean context). Any other node is a truthy expression.
fn eval_cond(cond: &AstNode) -> bool {
    !matches!(cond, AstNode::Placeholder)
}

/// Evaluate a batch of AST nodes to CSS statements synchronously.
///
/// Builds an `AstStream` from the node slice, evaluates it to `CssStream`,
/// then delegates to `collect_css` for terminal aggregation.
fn extract_css(nodes: &[AstNode], ctx: Arc<EvalContext>) -> Vec<crate::reactive::CssStmt> {
    let span = debug_span!("extract_css", node_count = nodes.len(), scope_id = ctx.scope_id);
    let _guard = span.enter();

    // Build AstStream from nodes, evaluate to CssStream
    let stream: AstStream = Shared::from_iter(nodes.to_vec()).box_it();
    let css_stream: CssStream = crate::reactive::evaluate_to_css(stream, ctx);

    // Delegate to collect_css helper (uses rxrust's collect operator)
    collect_css(css_stream)
}
