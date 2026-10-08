use rxrust::prelude::*;
use std::sync::Arc;
use crate::bus::{CompilerBus, FnDef, MixinDef};
use crate::runtime::EvalContext;
use crate::types::*;

pub mod builtin;
pub mod prefixer;
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
///
/// 使用 rxrust collect 算子进行流 → Vec 同步收集（非 subscribe-collect GC 模式）。
/// collect 算子已内化收集逻辑；Arc<Mutex> 仅作为 Shared 上下文 'static 约束下的值提取通道。
pub fn eval_stream(ast_stream: AstStream, ctx: Arc<EvalContext>) -> CssStream {
    let bus = ctx.bus().clone();

    // rxrust collect 算子: 流 → Vec（类型安全、无 GC 中间状态）
    let result = Arc::new(std::sync::Mutex::new(Vec::<AstNode>::new()));
    let r = result.clone();
    ast_stream.collect::<Vec<_>>().subscribe(move |v| { *r.lock().unwrap() = v; });
    let nodes = Arc::try_unwrap(result).unwrap().into_inner().unwrap();

    // 响应式求值管线
    eval_nodes_pipeline(&nodes, ctx, bus)
}

/// scan 算子的内部状态: frame 栈 + 最近完成的 CssStmt
type EvalState = (Vec<Frame>, Option<CssStmt>);

/// 响应式求值管线核心: 事件流 → scan 累积 → CSS 树
///
/// 使用真正的 rxrust 算子链:
///   Shared::from_iter(nodes).flat_map(emit_events).scan(frames, fold).filter_map(emit_completed)
///
/// - flat_map: 每个 AST 节点 → 事件序列（递归展开）
/// - scan: 事件流在 frame 栈上累积折叠
/// - filter_map: 提取已完成的 CssStmt
fn eval_nodes_pipeline(
    nodes: &[AstNode],
    ctx: Arc<EvalContext>,
    bus: CompilerBus,
) -> CssStream {
    let initial_state: EvalState = (vec![Frame::default()], None);

    // 算子链: nodes → flat_map(emit_events) → scan(fold_frames) → filter_map(emit_completed)
    // 先 collect 到 Vec 以满足 Shared::from_iter 的 'static 约束
    let owned_nodes: Vec<AstNode> = nodes.to_vec();
    Shared::from_iter(owned_nodes)
        .flat_map(move |node| {
            let events = emit_node_events(&node, None, &ctx, &bus);
            Shared::from_iter(events).box_it()
        })
        .scan(initial_state, fold_frames)
        .filter_map(emit_completed)
        .box_it()
}

/// 纯函数: 单个 AST 节点 → 事件序列（递归展开）
///
/// 替代原来的 work-queue 递归: 不是"手动栈维护嵌套"，
/// 而是"flat_map(emit_node_events) 由 rxrust 算子处理递归展开"。
fn emit_node_events(
    node: &AstNode,
    parent_sel: Option<&str>,
    ctx: &EvalContext,
    bus: &CompilerBus,
) -> Vec<EvalEvent> {
    let mut events = Vec::new();

    match node {
        AstNode::StyleDecl { property, value, important } => {
            let val = eval_expr(value, ctx, bus);
            let prop_name = resolve_property(property, ctx);
            let val_str = value_to_string(&val);
            // null 值 fallback 策略（Bootstrap dist 需要 --bs-* 变量保留在输出中）:
            // - CSS 自定义属性 (--bs-*): 输出 `unset` 保留声明（CSS 级联回退）
            // - 普通属性: 记录 debug warning 后跳过（null 在普通属性中应被移除）
            let final_val = if val_str == "null" || prop_name.contains("null") {
                if prop_name.starts_with("--") {
                    tracing::debug!(property = %prop_name, "null fallback to unset for CSS custom property");
                    "unset".to_string()
                } else {
                    tracing::debug!(property = %prop_name, "null value skipped for non-custom property");
                    return events;
                }
            } else {
                val_str
            };
            let final_val = if *important {
                format!("{} !important", final_val)
            } else {
                final_val
            };
            // Vendor prefix 自动注入: 若属性需要前缀且尚未带前缀，先 emit prefix 变体
            if !prefixer::is_prefixed(&prop_name)
                && let Some(prefixes) = prefixer::get_vendor_prefixes(&prop_name)
            {
                for pref in &prefixes {
                    events.push(EvalEvent::Terminal(CssStmt::Decl {
                        property: pref.clone(),
                        value: final_val.clone(),
                    }));
                }
            }
            events.push(EvalEvent::Terminal(CssStmt::Decl {
                property: prop_name,
                value: final_val,
            }));
        }
        AstNode::Rule { selector, inner } => {
            let resolved = resolve_selector(selector, ctx);
            let combined = match parent_sel {
                Some(p) => combine_selectors(p, &resolved),
                None => resolved.replace('&', ""),
            };
            let (nested_rules, declarations): (Vec<_>, Vec<_>) = inner.iter()
                .cloned()
                .partition(|n| matches!(n, AstNode::Rule { .. }));
            if !declarations.is_empty() {
                events.push(EvalEvent::EnterRule(combined.clone()));
                let child_ctx = ctx.child_scope(1);
                for n in &declarations {
                    events.extend(emit_node_events(n, Some(&combined), &child_ctx, bus));
                }
                events.push(EvalEvent::LeaveRule);
            }
            for n in &nested_rules {
                events.extend(emit_node_events(n, Some(&combined), ctx, bus));
            }
        }
        AstNode::Media { query, inner } => {
            let resolved_query = resolve_query(query, ctx);
            events.push(EvalEvent::EnterMedia(resolved_query));
            let child_ctx = ctx.child_scope(2);
            for n in inner {
                events.extend(emit_node_events(n, parent_sel, &child_ctx, bus));
            }
            events.push(EvalEvent::LeaveMedia);
        }
        AstNode::Supports { query, inner } => {
            let resolved_query = resolve_query(query, ctx);
            events.push(EvalEvent::EnterSupports(resolved_query));
            let child_ctx = ctx.child_scope(3);
            for n in inner {
                events.extend(emit_node_events(n, parent_sel, &child_ctx, bus));
            }
            events.push(EvalEvent::LeaveSupports);
        }
        AstNode::Import(inner_nodes) => {
            for n in inner_nodes {
                events.extend(emit_node_events(n, parent_sel, ctx, bus));
            }
        }
        AstNode::VariableDecl { name, value, .. } => {
            let val = eval_expr(value, ctx, bus);
            ctx.bind_var(name, val);
        }
        AstNode::MixinDecl { name, params, body } => {
            bus.register_mixin(MixinDef { name: name.clone(), params: params.clone(), body: body.clone() });
        }
        AstNode::FunctionDecl { name, params, body } => {
            bus.register_fn(FnDef { name: name.clone(), params: params.clone(), body: body.clone() });
        }
        AstNode::MixinCall { name, args, content } => {
            if let Some(mixin) = bus.lookup_mixin(name) {
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
                let expanded = expand_body_with_content(&mixin.body, content);
                for n in &expanded {
                    events.extend(emit_node_events(n, parent_sel, &child_ctx, bus));
                }
            }
        }
        AstNode::If { cond, then_branch, else_branch } => {
            let cond_val = eval_expr(cond, ctx, bus);
            let is_truthy = truthy(&cond_val);
            let branch = if is_truthy {
                then_branch
            } else if let Some(eb) = else_branch { eb } else { &vec![] };
            for n in branch {
                events.extend(emit_node_events(n, parent_sel, ctx, bus));
            }
        }
        AstNode::For { var, from, to, inclusive, body } => {
            let from_n = value_to_number(&eval_expr(from, ctx, bus)) as i64;
            let to_n = value_to_number(&eval_expr(to, ctx, bus)) as i64;
            let effective_to = if *inclusive { to_n } else { to_n - 1 };
            let mut i = effective_to;
            loop {
                if i < from_n { break; }
                let child_ctx = ctx.child_scope(5);
                child_ctx.bind_var(var, Value::Number(i as f64, None));
                for n in body {
                    events.extend(emit_node_events(n, parent_sel, &child_ctx, bus));
                }
                if i == from_n { break; }
                i -= 1;
            }
        }
        AstNode::Each { vars, list, body } => {
            let list_val = eval_expr(list, ctx, bus);
            let items = match list_val {
                Value::List(items, _) => items,
                Value::Map(entries) => entries.into_iter()
                    .map(|(k, v)| Value::List(vec![Value::String(k), v], ListSeparator::Comma))
                    .collect(),
                v => vec![v],
            };
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
                for n in body {
                    events.extend(emit_node_events(n, parent_sel, &child_ctx, bus));
                }
            }
        }
        AstNode::While { cond, body } => {
            let mut iterations = 0;
            loop {
                let cond_val = eval_expr(cond, ctx, bus);
                if !truthy(&cond_val) { break; }
                let child_ctx = ctx.child_scope(7);
                for n in body {
                    events.extend(emit_node_events(n, parent_sel, &child_ctx, bus));
                }
                iterations += 1;
                if iterations > 10000 { break; }
            }
        }
        AstNode::Css(stmt) => {
            events.push(EvalEvent::Terminal(stmt.clone()));
        }
        AstNode::Content => {}
        AstNode::Warn(val) => { let _ = eval_expr(val, ctx, bus); }
        AstNode::Debug(val) => { let _ = eval_expr(val, ctx, bus); }
        AstNode::Return(_) => {}
        _ => {}
    }

    events
}

/// scan 算子的折叠函数: 消费单个事件，返回新 frame 栈
///
/// 替代原来的 `for event { apply_event(event, &mut frames) }` 命令式循环。
/// 返回 (new_frames, optional_completed_stmt):
/// - 当 scope 关闭时，返回组装好的 CssStmt
/// - 其他情况返回 None
fn fold_frames(state: EvalState, event: EvalEvent) -> EvalState {
    let (mut frames, _) = state;
    let completed = match &event {
        EvalEvent::EnterRule(sel) => {
            frames.push(Frame { kind: FrameKind::Rule, selector: Some(sel.clone()), query: None, stmts: Vec::new() });
            None
        }
        EvalEvent::LeaveRule => {
            let frame = frames.pop().expect("unbalanced LeaveRule");
            let rule = CssStmt::Rule { selector: frame.selector.unwrap(), inner: frame.stmts };
            let is_root = frames.len() == 1;
            frames.last_mut().unwrap().stmts.push(rule.clone());
            if is_root { Some(rule) } else { None }
        }
        EvalEvent::EnterMedia(query) => {
            frames.push(Frame { kind: FrameKind::Media, selector: None, query: Some(query.clone()), stmts: Vec::new() });
            None
        }
        EvalEvent::LeaveMedia => {
            let frame = frames.pop().expect("unbalanced LeaveMedia");
            let media = CssStmt::Media { query: frame.query.unwrap(), inner: frame.stmts };
            let is_root = frames.len() == 1;
            frames.last_mut().unwrap().stmts.push(media.clone());
            if is_root { Some(media) } else { None }
        }
        EvalEvent::EnterSupports(query) => {
            frames.push(Frame { kind: FrameKind::Supports, selector: None, query: Some(query.clone()), stmts: Vec::new() });
            None
        }
        EvalEvent::LeaveSupports => {
            let frame = frames.pop().expect("unbalanced LeaveSupports");
            let supports = CssStmt::Supports { query: frame.query.unwrap(), inner: frame.stmts };
            let is_root = frames.len() == 1;
            frames.last_mut().unwrap().stmts.push(supports.clone());
            if is_root { Some(supports) } else { None }
        }
        EvalEvent::Terminal(stmt) => {
            frames.last_mut().unwrap().stmts.push(stmt.clone());
            // 顶层声明也作为 completed 输出
            if frames.len() == 1 {
                Some(stmt.clone())
            } else {
                None
            }
        }
    };
    (frames, completed)
}

/// filter_map 提取器: 从 scan 状态中提取已完成的 CssStmt
fn emit_completed(state: EvalState) -> Option<CssStmt> {
    state.1
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

/// 同步版本: 复用 eval_nodes_pipeline 算子链，收集结果为 Vec（保持 API 兼容性）
pub fn eval_nodes_sync(
    nodes: &[AstNode],
    ctx: &EvalContext,
    bus: &CompilerBus,
) -> Vec<CssStmt> {
    let ctx_arc = Arc::new(ctx.clone());
    let bus_owned = bus.clone();
    let css_stream = eval_nodes_pipeline(nodes, ctx_arc, bus_owned);

    // rxrust collect 算子: CssStream → Vec<CssStmt>（同步收集）
    let result = Arc::new(std::sync::Mutex::new(Vec::<CssStmt>::new()));
    let r = result.clone();
    css_stream.collect::<Vec<_>>().subscribe(move |v| *r.lock().unwrap() = v);
    Arc::try_unwrap(result).unwrap().into_inner().unwrap()
}

/// 同步收集 AST 流并使用 rxrust collect 算子（非 subscribe-collect GC 模式）。
pub fn eval_ast_stream_sync(ast_stream: AstStream, ctx: Arc<EvalContext>) -> Vec<CssStmt> {
    let bus = ctx.bus().clone();
    let result = Arc::new(std::sync::Mutex::new(Vec::<AstNode>::new()));
    let r = result.clone();
    ast_stream.collect::<Vec<_>>().subscribe(move |v| { *r.lock().unwrap() = v; });
    let nodes = Arc::try_unwrap(result).unwrap().into_inner().unwrap();
    eval_nodes_sync(&nodes, &ctx, &bus)
}

// Expression evaluation lives in expr.rs (leaf-level, bounded depth)


