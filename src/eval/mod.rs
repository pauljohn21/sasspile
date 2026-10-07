use rxrust::prelude::*;
use std::convert::Infallible;
use std::sync::Arc;
use crate::bus::{CompilerBus, FnDef, MixinDef};
use crate::runtime::EvalContext;
use crate::types::*;

pub mod builtin;
mod expr;

// Re-export for internal use + tests
pub use expr::eval_expr;
use expr::{combine_selectors, resolve_selector, truthy, value_to_number, value_to_string};

// ── RxRust 求值器: 响应式流算子替代递归 ─────────────────────────────────
//
// 核心设计:
//   AST 节点流 → expand(reactive recursion) → 事件流 → scan(累积折叠) → CSS 树流
//
// 不是"函数调用自己"，而是"emit 子节点回同一个 Observable"
// 不是"手动栈管理"，而是"scan 算子在流上累积状态"

/// 求值事件: 在 reactive recursion 中间节点不是递归调用，
/// 而是 emit 事件回流 — 子节点自动进入流的下游
#[derive(Clone)]
enum EvalEvent {
    /// 进入一棵嵌套 Rule（selector 已 combine 完毕）
    EnterRule(String),
    /// 离开当前 Rule scope
    LeaveRule,
    /// 进入 @media scope
    EnterMedia(String),
    /// 离开 @media scope
    LeaveMedia,
    /// 进入 @supports scope
    EnterSupports(String),
    /// 离开 @supports scope
    LeaveSupports,
    /// 终端 CssStmt（Decl、Comment 等）
    Terminal(CssStmt),
}

/// 累积 frames 栈 — scan 算子的内部状态
/// 每层 frame 累积该 scope 的 CssStmt 列表
#[derive(Clone)]
struct Frame {
    #[allow(dead_code)]
    kind: FrameKind,
    selector: Option<String>,
    query: Option<String>,
    stmts: Vec<CssStmt>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FrameKind { Root, Rule, Media, Supports }

impl Default for Frame {
    fn default() -> Self {
        Self { kind: FrameKind::Root, selector: None, query: None, stmts: Vec::new() }
    }
}

/// 入口: 收集 AST 流 → 响应式求值管线
pub fn eval_stream(ast_stream: AstStream, ctx: Arc<EvalContext>) -> CssStream {
    let bus = ctx.bus().clone();

    // 收集 AST 节点（parser 产物是流，先收集为 Vec）
    let nodes_arc = Arc::new(std::sync::Mutex::new(Vec::<AstNode>::new()));
    let nref = nodes_arc.clone();
    ast_stream.subscribe(move |node| { nref.lock().unwrap().push(node); });
    let nodes = nodes_arc.lock().unwrap().clone();

    // 响应式求值管线
    eval_nodes_pipeline(&nodes, ctx, bus)
}

/// 响应式求值管线核心: 事件流 → scan 累积 → CSS 树
///
/// 模拟 rxrust scan 算子: 在事件流上累积 frame 栈。
/// Enter* 事件 = push frame；Leave* 事件 = pop frame 并组装 CssStmt。
fn eval_nodes_pipeline(
    nodes: &[AstNode],
    ctx: Arc<EvalContext>,
    bus: CompilerBus,
) -> CssStream {
    let output: SharedSubject<'static, CssStmt, Infallible> = Shared::subject();
    let mut out_ref = output.clone();

    // Phase 1: 响应式展开 — AST 节点 → 事件流
    let events = expand_nodes_to_events(nodes, &ctx, &bus);

    // Phase 2: scan 累积 — 事件流在 frame 栈上折叠为 CSS 树
    let mut frames = vec![Frame::default()];
    for event in &events {
        apply_event(event, &mut frames, &out_ref);
    }
    let stmts = frames.pop().map(|f| f.stmts).unwrap_or_default();

    for stmt in &stmts { out_ref.next(stmt.clone()); }
    output.box_it()
}

/// 响应式展开: 模拟 expand 算子 — 节点 → 事件序列
///
/// 核心思想: "emit children back into the same stream"
/// 不是递归函数调用自己，而是把子节点重新注入 work-queue。
/// 这在 rxrust 中对应 expand 算子: 每个元素产生新 Observable 合并回流。
fn expand_nodes_to_events(
    nodes: &[AstNode],
    ctx: &EvalContext,
    bus: &CompilerBus,
) -> Vec<EvalEvent> {
    let mut events = Vec::<EvalEvent>::new();

    // Work item — 携带正确的 EvalContext（作用域在产生时就确定）
    // 这是对递归 ctx 参数传递的模拟: 每个 mixin/rule body 需要自己的 ctx
    enum Work {
        Node(AstNode, Option<String>, Arc<EvalContext>),
        LeaveRule,
        LeaveMedia,
        LeaveSupports,
    }
    let root_ctx = Arc::new(ctx.clone());
    let mut queue: Vec<Work> = nodes.iter().map(|n| Work::Node(n.clone(), None, root_ctx.clone())).collect();
    queue.reverse();

    let mut count: usize = 0;
    const MAX: usize = 100_000;

    while let Some(work) = queue.pop() {
        count += 1;
        if count > MAX { break; }

        match work {
            Work::LeaveRule => events.push(EvalEvent::LeaveRule),
            Work::LeaveMedia => events.push(EvalEvent::LeaveMedia),
            Work::LeaveSupports => events.push(EvalEvent::LeaveSupports),

            Work::Node(node, parent_sel, work_ctx) => {
                // 在当前 work item 的上下文中处理（mixin/rule body 有自己的 scope）
                let ctx: &EvalContext = &work_ctx;
                match node {
                AstNode::StyleDecl { property, value } => {
                    let val = eval_expr(&value, ctx, bus);
                    events.push(EvalEvent::Terminal(CssStmt::Decl {
                        property,
                        value: value_to_string(&val),
                    }));
                }
                AstNode::Rule { selector, inner } => {
                    let child_ctx = ctx.child_scope(1);
                    let resolved = resolve_selector(&selector, ctx);
                    let combined = match &parent_sel {
                        Some(p) => combine_selectors(p, &resolved),
                        None => resolved,
                    };
                    events.push(EvalEvent::EnterRule(combined.clone()));
                    queue.push(Work::LeaveRule);
                    for n in inner.iter().rev() {
                        queue.push(Work::Node(n.clone(), Some(combined.clone()), Arc::new(child_ctx.clone())));
                    }
                }
                AstNode::Media { query, inner } => {
                    let child_ctx = ctx.child_scope(2);
                    events.push(EvalEvent::EnterMedia(query.clone()));
                    queue.push(Work::LeaveMedia);
                    for n in inner.iter().rev() {
                        queue.push(Work::Node(n.clone(), parent_sel.clone(), Arc::new(child_ctx.clone())));
                    }
                }
                AstNode::Supports { query, inner } => {
                    let child_ctx = ctx.child_scope(3);
                    events.push(EvalEvent::EnterSupports(query.clone()));
                    queue.push(Work::LeaveSupports);
                    for n in inner.iter().rev() {
                        queue.push(Work::Node(n.clone(), parent_sel.clone(), Arc::new(child_ctx.clone())));
                    }
                }
                AstNode::Import(inner_nodes) => {
                    for n in inner_nodes.iter().rev() {
                        queue.push(Work::Node(n.clone(), parent_sel.clone(), Arc::new(ctx.clone())));
                    }
                }
                AstNode::VariableDecl { name, value, .. } => {
                    let val = eval_expr(&value, ctx, bus);
                    ctx.bind_var(&name, val);
                }
                AstNode::MixinDecl { name, params, body } => {
                    bus.register_mixin(MixinDef { name, params, body });
                }
                AstNode::FunctionDecl { name, params, body } => {
                    bus.register_fn(FnDef { name, params, body });
                }
                AstNode::MixinCall { name, args } => {
                    if let Some(mixin) = bus.lookup_mixin(&name) {
                        let child_ctx = ctx.child_scope(8);
                        for (p, a) in mixin.params.iter().zip(args.iter()) {
                            let val = eval_expr(a, ctx, bus);
                            child_ctx.bind_var(&p.name, val);
                        }
                        for n in mixin.body.iter().rev() {
                            queue.push(Work::Node(n.clone(), parent_sel.clone(), Arc::new(child_ctx.clone())));
                        }
                    }
                }
                AstNode::If { cond, then_branch, else_branch } => {
                    let cond_val = eval_expr(&cond, ctx, bus);
                    let branch = if truthy(&cond_val) {
                        then_branch
                    } else if let Some(eb) = else_branch { eb } else { vec![] };
                    for n in branch.iter().rev() {
                        queue.push(Work::Node(n.clone(), parent_sel.clone(), Arc::new(ctx.clone())));
                    }
                }
                AstNode::For { var, from, to, inclusive, body } => {
                    let from_n = value_to_number(&eval_expr(&from, ctx, bus)) as i64;
                    let to_n = value_to_number(&eval_expr(&to, ctx, bus)) as i64;
                    let child_ctx = ctx.child_scope(5);
                    let mut i = from_n;
                    while if inclusive { i <= to_n } else { i < to_n } {
                        child_ctx.bind_var(&var, Value::Number(i as f64, None));
                        for n in body.iter().rev() {
                            queue.push(Work::Node(n.clone(), parent_sel.clone(), Arc::new(child_ctx.clone())));
                        }
                        i += 1;
                    }
                }
                AstNode::Each { vars, list, body } => {
                    let list_val = eval_expr(&list, ctx, bus);
                    let items = match list_val {
                        Value::List(items) => items,
                        v => vec![v],
                    };
                    let child_ctx = ctx.child_scope(6);
                    for item_val in items {
                        if vars.len() == 1 {
                            child_ctx.bind_var(&vars[0], item_val);
                        }
                        for n in body.iter().rev() {
                            queue.push(Work::Node(n.clone(), parent_sel.clone(), Arc::new(child_ctx.clone())));
                        }
                    }
                }
                AstNode::While { cond, body } => {
                    let child_ctx = ctx.child_scope(7);
                    let mut iterations = 0;
                    loop {
                        let cond_val = eval_expr(&cond, ctx, bus);
                        if !truthy(&cond_val) { break; }
                        for n in body.iter().rev() {
                            queue.push(Work::Node(n.clone(), parent_sel.clone(), Arc::new(child_ctx.clone())));
                        }
                        iterations += 1;
                        if iterations > 10000 { break; }
                    }
                }
                AstNode::Css(stmt) => {
                    events.push(EvalEvent::Terminal(stmt));
                }
                AstNode::Warn(val) => { let _ = eval_expr(&val, ctx, bus); }
                AstNode::Debug(val) => { let _ = eval_expr(&val, ctx, bus); }
                AstNode::Return(_) => {}
                _ => {}
            }
            } // end Work::Node
        }
    }

    events
}

/// scan 累积: 消费事件，维护 frame 栈，当 scope 关闭时 emit CssStmt
fn apply_event(
    event: &EvalEvent,
    frames: &mut Vec<Frame>,
    _out: &SharedSubject<'static, CssStmt, Infallible>,
) {
    match event {
        EvalEvent::EnterRule(sel) => {
            frames.push(Frame { kind: FrameKind::Rule, selector: Some(sel.clone()), query: None, stmts: Vec::new() });
        }
        EvalEvent::LeaveRule => {
            let frame = frames.pop().expect("unbalanced LeaveRule");
            let rule = CssStmt::Rule { selector: frame.selector.unwrap(), inner: frame.stmts };
            frames.last_mut().unwrap().stmts.push(rule);
        }
        EvalEvent::EnterMedia(query) => {
            frames.push(Frame { kind: FrameKind::Media, selector: None, query: Some(query.clone()), stmts: Vec::new() });
        }
        EvalEvent::LeaveMedia => {
            let frame = frames.pop().expect("unbalanced LeaveMedia");
            let media = CssStmt::Media { query: frame.query.unwrap(), inner: frame.stmts };
            frames.last_mut().unwrap().stmts.push(media);
        }
        EvalEvent::EnterSupports(query) => {
            frames.push(Frame { kind: FrameKind::Supports, selector: None, query: Some(query.clone()), stmts: Vec::new() });
        }
        EvalEvent::LeaveSupports => {
            let frame = frames.pop().expect("unbalanced LeaveSupports");
            let supports = CssStmt::Supports { query: frame.query.unwrap(), inner: frame.stmts };
            frames.last_mut().unwrap().stmts.push(supports);
        }
        EvalEvent::Terminal(stmt) => {
            frames.last_mut().unwrap().stmts.push(stmt.clone());
        }
    }
}

pub fn eval_nodes_sync(
    nodes: &[AstNode],
    ctx: &EvalContext,
    bus: &CompilerBus,
) -> Vec<CssStmt> {
    let events = expand_nodes_to_events(nodes, ctx, bus);
    let mut frames = vec![Frame::default()];
    let out = Shared::subject();
    for event in &events { apply_event(event, &mut frames, &out); }
    frames.pop().map(|f| f.stmts).unwrap_or_default()
}

pub fn eval_ast_stream_sync(ast_stream: AstStream, ctx: Arc<EvalContext>) -> Vec<CssStmt> {
    let bus = ctx.bus().clone();
    let v_nodes = Arc::new(std::sync::Mutex::new(Vec::<AstNode>::new()));

    let v_ref = v_nodes.clone();
    ast_stream.subscribe(move |node| { v_ref.lock().unwrap().push(node); });

    let nodes = v_nodes.lock().unwrap().clone();
    eval_nodes_sync(&nodes, &ctx, &bus)
}

// Expression evaluation lives in expr.rs (leaf-level, bounded depth)


