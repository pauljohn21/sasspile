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
use expr::{combine_selectors, resolve_property, resolve_selector, truthy, value_to_number, value_to_string};

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
                AstNode::StyleDecl { property, value, important } => {
                    let val = eval_expr(&value, ctx, bus);
                    let prop_name = resolve_property(&property, ctx);
                    let val_str = value_to_string(&val);
                    // null 值显示为 "null" 的声明不输出到 CSS（Bootstrap 中 null 是占位符）
                    // 覆盖 Value::Null、List([Null])、String("null") 等衍生情况
                    if val_str == "null" || prop_name.contains("null") {
                        continue;
                    }
                    // Append ` !important` when the SCSS source had `!important` flag
                    let final_val = if important {
                        format!("{} !important", val_str)
                    } else {
                        val_str
                    };
                    events.push(EvalEvent::Terminal(CssStmt::Decl {
                        property: prop_name,
                        value: final_val,
                    }));
                }
                AstNode::Rule { selector, inner } => {
                    let child_ctx = ctx.child_scope(1);
                    let resolved = resolve_selector(&selector, ctx);
                    let combined = match &parent_sel {
                        Some(p) => combine_selectors(p, &resolved),
                        // 无父级时，剥掉 `&`（顶层 mixin body 中的 `&:hover` → `:hover`）
                        None => resolved.replace('&', ""),
                    };
                    // 分离 rule 节点和非 rule 节点
                    // CSS 展平：嵌套规则（扁平输出）vs 声明（保留在父规则内）
                    let (nested_rules, declarations): (Vec<_>, Vec<_>) = inner.iter()
                        .cloned()
                        .partition(|n| matches!(n, AstNode::Rule { .. }));
                    // 有声明输出 EnterRule + 声明 + LeaveRule
                    if !declarations.is_empty() {
                        events.push(EvalEvent::EnterRule(combined.clone()));
                        queue.push(Work::LeaveRule);
                        for n in declarations.iter().rev() {
                            queue.push(Work::Node(n.clone(), Some(combined.clone()), Arc::new(child_ctx.clone())));
                        }
                    }
                    // 嵌套规则作为兄弟节点展开（在 LeaveRule 之后处理）
                    for n in nested_rules {
                        queue.push(Work::Node(n, Some(combined.clone()), Arc::new(ctx.clone())));
                    }
                }
                AstNode::Media { query, inner } => {
                    let child_ctx = ctx.child_scope(2);
                    let resolved_query = resolve_query(&query, ctx);
                    events.push(EvalEvent::EnterMedia(resolved_query));
                    queue.push(Work::LeaveMedia);
                    for n in inner.iter().rev() {
                        queue.push(Work::Node(n.clone(), parent_sel.clone(), Arc::new(child_ctx.clone())));
                    }
                }
                AstNode::Supports { query, inner } => {
                    let child_ctx = ctx.child_scope(3);
                    let resolved_query = resolve_query(&query, ctx);
                    events.push(EvalEvent::EnterSupports(resolved_query));
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
                AstNode::MixinCall { name, args, content } => {
                    if let Some(mixin) = bus.lookup_mixin(&name) {
                        let child_ctx = ctx.child_scope(8);
                        for (i, p) in mixin.params.iter().enumerate() {
                            let val = if let Some(a) = args.get(i) {
                                eval_expr(a, ctx, bus)
                            } else if let Some(default) = &p.default_value {
                                eval_expr(default, ctx, bus)
                            } else {
                                Value::Null
                            };
                            child_ctx.bind_var(&p.name, val);
                        }
                        let expanded = expand_body_with_content(&mixin.body, &content);
                        for n in expanded.iter().rev() {
                            queue.push(Work::Node(n.clone(), parent_sel.clone(), Arc::new(child_ctx.clone())));
                        }
                    }
                }
                AstNode::If { cond, then_branch, else_branch } => {
                    let cond_val = eval_expr(&cond, ctx, bus);
                    let is_truthy = truthy(&cond_val);
                    let branch = if is_truthy {
                        then_branch
                    } else if let Some(eb) = else_branch { eb } else { vec![] };
                    for n in branch.iter().rev() {
                        queue.push(Work::Node(n.clone(), parent_sel.clone(), Arc::new(ctx.clone())));
                    }
                }
                AstNode::For { var, from, to, inclusive, body } => {
                    let from_n = value_to_number(&eval_expr(&from, ctx, bus)) as i64;
                    let to_n = value_to_number(&eval_expr(&to, ctx, bus)) as i64;
                    // `to` 是排他的（不包含），`through` 是包含的
                    let effective_to = if inclusive { to_n } else { to_n - 1 };
                    // 反向迭代：stack 是 LIFO，反向 push 才能正向 pop
                    let mut i = effective_to;
                    loop {
                        if i < from_n { break; }
                        let child_ctx = ctx.child_scope(5);
                        child_ctx.bind_var(&var, Value::Number(i as f64, None));
                        for n in body.iter().rev() {
                            queue.push(Work::Node(n.clone(), parent_sel.clone(), Arc::new(child_ctx.clone())));
                        }
                        if i == from_n { break; }
                        i -= 1;
                    }
                }
                AstNode::Each { vars, list, body } => {
                    let list_val = eval_expr(&list, ctx, bus);
                    let items = match list_val {
                        Value::List(items, _) => items,
                        Value::Map(entries) => entries.into_iter()
                            .map(|(k, v)| Value::List(vec![Value::String(k), v], ListSeparator::Comma))
                            .collect(),
                        v => vec![v],
                    };
                    // 反向迭代 items：stack 是 LIFO，反向 push 才能正向 pop
                    for item_val in items.into_iter().rev() {
                        let child_ctx = ctx.child_scope(6);
                        match vars.len() {
                            1 => child_ctx.bind_var(&vars[0], item_val),
                            2 => {
                                if let Value::List(pair, _) = &item_val {
                                    if let Some(key) = pair.first() {
                                        child_ctx.bind_var(&vars[0], key.clone());
                                    }
                                    if let Some(val) = pair.get(1) {
                                        child_ctx.bind_var(&vars[1], val.clone());
                                    }
                                } else {
                                    child_ctx.bind_var(&vars[0], item_val);
                                }
                            }
                            _ => child_ctx.bind_var(&vars[0], item_val),
                        }
                        for n in body.iter().rev() {
                            queue.push(Work::Node(n.clone(), parent_sel.clone(), Arc::new(child_ctx.clone())));
                        }
                    }
                }
                AstNode::While { cond, body } => {
                    // 收集所有迭代产生的 work items，然后反向 push
                    let mut work_items: Vec<Work> = Vec::new();
                    let mut iterations = 0;
                    loop {
                        let cond_val = eval_expr(&cond, ctx, bus);
                        if !truthy(&cond_val) { break; }
                        let child_ctx = ctx.child_scope(7);
                        for n in body.iter().rev() {
                            work_items.push(Work::Node(n.clone(), parent_sel.clone(), Arc::new(child_ctx.clone())));
                        }
                        iterations += 1;
                        if iterations > 10000 { break; }
                    }
                    // 反向 push 到 queue，确保正向 pop 顺序
                    for work in work_items.into_iter().rev() {
                        queue.push(work);
                    }
                }
                AstNode::Css(stmt) => {
                    events.push(EvalEvent::Terminal(stmt));
                }
                AstNode::Content => {
                    // @content without a surrounding @include — no-op at top level
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

/// Resolve variable references in a media/supports query string.
/// The parser produces queries like "(min-width: $w)" or "(min-width: $min)".
/// This substitutes $var with its current runtime value.
fn resolve_query(query: &str, ctx: &EvalContext) -> String {
    if !query.contains('$') {
        return query.to_string();
    }
    let mut result = String::new();
    let mut chars = query.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '$' {
            // Collect variable name (alphanumeric + hyphen + underscore)
            let mut var_name = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_alphanumeric() || c == '_' || c == '-' {
                    var_name.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            if !var_name.is_empty() {
                let val = ctx.var(&var_name).unwrap_or(Value::Null);
                result.push_str(&value_to_string(&val));
            } else {
                result.push('$');
            }
        } else {
            result.push(ch);
        }
    }
    result
}

/// Expand mixin body: replace any `AstNode::Content` markers with the caller's content block.
/// Handles nested structures (Content inside Rule, Media, etc.) recursively.
fn expand_body_with_content(nodes: &[AstNode], content: &[AstNode]) -> Vec<AstNode> {
    nodes.iter()
        .flat_map(|node| expand_node_with_content(node, content))
        .collect()
}

/// Expand a single AstNode: if it's Content, return the content block;
/// otherwise recurse into container nodes. Returns a Vec because Content
/// expands to 0..N nodes.
fn expand_node_with_content(node: &AstNode, content: &[AstNode]) -> Vec<AstNode> {
    if matches!(node, AstNode::Content) {
        return content.to_vec();
    }
    // Recurse into container nodes that have inner children
    match node {
        AstNode::Rule { selector, inner } => {
            let expanded = expand_body_with_content(inner, content);
            vec![AstNode::Rule { selector: selector.clone(), inner: expanded }]
        }
        AstNode::Media { query, inner } => {
            let expanded = expand_body_with_content(inner, content);
            vec![AstNode::Media { query: query.clone(), inner: expanded }]
        }
        AstNode::Supports { query, inner } => {
            let expanded = expand_body_with_content(inner, content);
            vec![AstNode::Supports { query: query.clone(), inner: expanded }]
        }
        AstNode::If { cond, then_branch, else_branch } => {
            let then_expanded = expand_body_with_content(then_branch, content);
            let else_expanded = else_branch.as_ref().map(|eb| expand_body_with_content(eb, content));
            vec![AstNode::If { cond: cond.clone(), then_branch: then_expanded, else_branch: else_expanded }]
        }
        _ => vec![node.clone()],
    }
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


