//! Single `SassOp for AstNode` implementation — dispatches each AST node
//! variant to its operator logic.

use std::cell::RefCell;
use std::rc::Rc;

use rxrust::prelude::*;
use tracing::{debug_span, info_span};

use crate::reactive::{AstNode, AstStream, CssStmt, EvalContext, FnDef, MixinDef, SassOp};

impl SassOp for AstNode {
    fn into_operator(
        self,
        ctx: Rc<EvalContext>,
    ) -> Box<dyn Fn(AstStream) -> AstStream> {
        // Emit a tracing event per operator dispatch (task 10.2)
        let span = info_span!("sass_op", node = ?self, scope_id = ctx.scope_id);
        let _guard = span.enter();

        match self {
            // VariableDecl: emit Bind event, no CSS output
            AstNode::VariableDecl { name, value } => {
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
                Box::new(|stream| stream)
            }

            // StyleDecl: directly lower to CssStmt::Decl
            AstNode::StyleDecl { property, value } => {
                let span = debug_span!("style_decl", %property, %value, scope_id = ctx.scope_id);
                let _guard = span.enter();
                let css_node = AstNode::Css(CssStmt::Decl { property, value });
                Box::new(move |_| Local::of(css_node.clone()).box_it_clone())
            }

            // RuleSet: evaluate inner nodes, collect CSS, wrap in Rule
            AstNode::RuleSet { selector, inner } => {
                let span = debug_span!("rule_set", %selector, scope_id = ctx.scope_id);
                let _guard = span.enter();
                let child_ctx = Rc::new(ctx.child_scope(1));
                let css_vec = extract_css(&inner, child_ctx);
                let rule = AstNode::Css(CssStmt::Rule {
                    selector,
                    inner: css_vec,
                });
                Box::new(move |_| Local::of(rule.clone()).box_it_clone())
            }

            // @media: evaluate inner nodes, wrap in Media
            AstNode::Media { query, inner } => {
                let span = debug_span!("media", %query, scope_id = ctx.scope_id);
                let _guard = span.enter();
                let child_ctx = Rc::new(ctx.child_scope(1));
                let css_vec = extract_css(&inner, child_ctx);
                let media = AstNode::Css(CssStmt::Media {
                    query,
                    inner: css_vec,
                });
                Box::new(move |_| Local::of(media.clone()).box_it_clone())
            }

            // @supports: evaluate inner nodes, wrap in Supports
            AstNode::Supports { query, inner } => {
                let span = debug_span!("supports", %query, scope_id = ctx.scope_id);
                let _guard = span.enter();
                let child_ctx = Rc::new(ctx.child_scope(1));
                let css_vec = extract_css(&inner, child_ctx);
                let supports = AstNode::Css(CssStmt::Supports {
                    query,
                    inner: css_vec,
                });
                Box::new(move |_| Local::of(supports.clone()).box_it_clone())
            }

            // @if: select the first branch with a truthy condition
            AstNode::If { cond, then_branch, else_branch } => {
                let span = debug_span!("if", scope_id = ctx.scope_id);
                let _guard = span.enter();
                let child_ctx = Rc::new(ctx.child_scope(1));
                let selected = if eval_cond(&cond) {
                    tracing::debug!("selecting then branch");
                    then_branch
                } else {
                    tracing::debug!("selecting else branch");
                    else_branch
                };
                let css_vec = extract_css(&selected, child_ctx);
                let stream: AstStream =
                    Local::from_iter(css_vec.into_iter().map(AstNode::Css)).box_it_clone();
                Box::new(move |_| stream.clone())
            }

            // @for: flat_map over numeric range
            AstNode::For { from, through, body, .. } => {
                let span = debug_span!("for", %from, %through, scope_id = ctx.scope_id);
                let _guard = span.enter();
                let child_ctx = Rc::new(ctx.child_scope(1));
                let iterations = ((through - from).round() as i64 + 1).max(0);
                let mut all_nodes = Vec::new();
                for i in 0..iterations {
                    tracing::trace!(iteration = i, "for iteration");
                    let css_vec = extract_css(&body, child_ctx.clone());
                    for stmt in css_vec {
                        all_nodes.push(AstNode::Css(stmt));
                    }
                }
                let stream: AstStream = Local::from_iter(all_nodes).box_it_clone();
                Box::new(move |_| stream.clone())
            }

            // @each: flat_map over list — stub pass-through (list resolution pending)
            AstNode::Each { .. } => {
                tracing::debug!("each: pass-through (list resolution pending)");
                Box::new(|stream| stream)
            }

            // @while: flat_map with iteration limit to prevent infinite loops
            AstNode::While { cond, body } => {
                const MAX_WHILE_ITERATIONS: i64 = 10_000;
                let span = debug_span!("while", scope_id = ctx.scope_id);
                let _guard = span.enter();
                let child_ctx = Rc::new(ctx.child_scope(1));
                let mut all_nodes = Vec::new();
                for i in 0..MAX_WHILE_ITERATIONS {
                    if !eval_cond(&cond) {
                        tracing::debug!(iteration = i, "while condition false, exiting");
                        break;
                    }
                    let css_vec = extract_css(&body, child_ctx.clone());
                    for stmt in css_vec {
                        all_nodes.push(AstNode::Css(stmt));
                    }
                }
                let stream: AstStream = Local::from_iter(all_nodes).box_it_clone();
                Box::new(move |_| stream.clone())
            }

            // Diagnostics: @warn / @debug tap the stream and emit tracing event
            AstNode::Warn { message } => {
                tracing::warn!(%message, scope_id = ctx.scope_id, "@warn directive");
                Box::new(|stream| stream)
            }
            AstNode::Debug { expr: _, message } => {
                tracing::debug!(%message, scope_id = ctx.scope_id, "@debug directive");
                Box::new(|stream| stream)
            }

            // 7.1 @mixin: register definition, no CSS output
            AstNode::Mixin { name, params, body } => {
                let span = debug_span!("mixin_def", %name, scope_id = ctx.scope_id);
                let _guard = span.enter();
                ctx.bus.register_mixin(MixinDef { name, params, body });
                Box::new(|stream| stream)
            }

            // 7.2 @include: expand registered mixin body
            AstNode::MixinCall { name, args } => {
                let span = debug_span!("mixin_call", %name, ?args, scope_id = ctx.scope_id);
                let _guard = span.enter();
                let child_ctx = Rc::new(ctx.child_scope(1));
                let expanded = match ctx.bus.lookup_mixin(&name) {
                    Some(def) => {
                        tracing::debug!(params = ?def.params, body_len = def.body.len(), "mixin found");
                        // For now, expand body directly. A full impl would
                        // bind @args to @params via ctx.ver().
                        let css_vec = extract_css(&def.body, child_ctx);
                        css_vec.into_iter().map(AstNode::Css).collect::<Vec<_>>()
                    }
                    None => {
                        tracing::warn!("mixin not found");
                        Vec::new()
                    }
                };
                let stream: AstStream = Local::from_iter(expanded).box_it_clone();
                Box::new(move |_| stream.clone())
            }

            // 7.4 @use: load module via module_events
            AstNode::UseRule { path } => {
                let span = debug_span!("use_rule", %path, scope_id = ctx.scope_id);
                let _guard = span.enter();
                let mut module_subject = ctx.bus.module_events();
                module_subject.next(crate::reactive::ModuleEvent::Load { name: path });
                Box::new(|stream| stream)
            }

            // 7.3 @function: register callable function
            AstNode::FunctionDecl { name, params, body } => {
                let span = debug_span!("function_def", %name, scope_id = ctx.scope_id);
                let _guard = span.enter();
                ctx.bus.register_fn(FnDef { name, params, body });
                Box::new(|stream| stream)
            }

            // Fallback: pass-through
            other => {
                tracing::trace!(node = ?other, "pass-through operator");
                Box::new(|stream| stream)
            }
        }
    }
}

/// Simplified condition evaluation.
///
/// Treats `AstNode::Placeholder` as false (default/nodes without
/// explicit boolean context). Any other node is a truthy expression.
fn eval_cond(cond: &AstNode) -> bool {
    !matches!(cond, AstNode::Placeholder)
}

/// Evaluate a batch of AST nodes to CSS statements synchronously.
fn extract_css(nodes: &[AstNode], ctx: Rc<EvalContext>) -> Vec<CssStmt> {
    let result = Rc::new(RefCell::new(Vec::new()));
    let r = result.clone();

    let stream: AstStream = Local::from_iter(nodes.to_vec()).box_it_clone();
    let css_stream = crate::reactive::evaluate_to_css(stream, ctx);
    css_stream.subscribe(move |stmt| r.borrow_mut().push(stmt));

    match Rc::try_unwrap(result) {
        Ok(cell) => cell.into_inner(),
        Err(_) => Vec::new(),
    }
}

/// Helper: extract a `CssStmt` from an `AstNode` if it's a `Css` variant.
pub fn apply_css_node(node: AstNode) -> Option<CssStmt> {
    match node {
        AstNode::Css(stmt) => Some(stmt),
        _ => None,
    }
}
