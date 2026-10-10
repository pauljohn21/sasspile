use rxrust::prelude::*;
use std::sync::Arc;
use crate::runtime::EvalContext;
use crate::types::*;

pub mod builtin;
pub mod prefixer;
mod expr;
mod emit;

// Re-export for internal use + tests
pub use expr::eval_expr;
pub(crate) use emit::emit_events;

// ── RxRust 求值器: 纯响应式算子链 (scan_map &mut 零 clone) ──────────────────
//
// 核心设计:
//   AST 节点流 → flat_map(emit_events) → scan_map(&mut state, fold_frame) → filter_map(extract)
//
// - emit_events (emit.rs): 返回 lazy Observable<EvalEvent>，递归 flat_map 让 rxrust 处理展平
// - fold_frame: Fn(&mut EvalState, EvalEvent) -> Option<CssStmt>，scan_map 直接修改状态，零 clone
// - filter_map: 从 scan_map 的 Output (Option<CssStmt>) 透传已完成的 CssStmt
//
// 关键 (基于 rxrust 源码分析):
//   scan_map 用 &mut Acc，不像 scan 每帧 Clone 整个状态 (scan.rs L68-70)
//   collect 是终止算子，替代 Arc<Mutex<Vec>> + subscribe 反模式
//   box_it() 只在管线最终返回边界调用一次

/// 求值事件: 在响应式递归中，子节点通过 flat_map 返回 Observable 自动注入下游流
#[derive(Clone, Debug)]
pub(crate) enum EvalEvent {
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
    /// @extend <target_selector>: 将当前 selector 追加到 target_selector 的 selector list
    AddSelector { target: String, source: String },
}

/// scan_map 算子的内部状态: frame 栈 + 待处理的 @extend
///
/// 使用 &mut self 直接修改，无需 Clone。scan_map (scan_map.rs L65-67) 直接调用
/// `FnMut(&mut Acc, Item) -> Output` 不产生 clone 开销。
pub(super) struct EvalState {
    frames: Vec<Frame>,
    pending_extends: Vec<(String, String)>,
}

impl EvalState {
    /// 创建 root 状态的求值器
    pub(super) fn root() -> Self {
        Self {
            frames: vec![Frame::default()],
            pending_extends: Vec::new(),
        }
    }
}

/// 累积 frames 栈 — scan_map 算子的内部状态
#[derive(Clone)]
pub(super) struct Frame {
    kind: FrameKind,
    selector: Option<String>,
    query: Option<String>,
    stmts: Vec<CssStmt>,
    /// 被 @extend 引用的 selectors: Vec<source_selector>
    extended_by: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum FrameKind { Root, Rule, Media, Supports }

impl Default for Frame {
    fn default() -> Self {
        Self { kind: FrameKind::Root, selector: None, query: None, stmts: Vec::new(), extended_by: Vec::new() }
    }
}

impl Frame {
    pub(super) fn new_rule(selector: String) -> Self {
        Self { kind: FrameKind::Rule, selector: Some(selector), query: None, stmts: Vec::new(), extended_by: Vec::new() }
    }
    pub(super) fn new_media(query: String) -> Self {
        Self { kind: FrameKind::Media, selector: None, query: Some(query), stmts: Vec::new(), extended_by: Vec::new() }
    }
    pub(super) fn new_supports(query: String) -> Self {
        Self { kind: FrameKind::Supports, selector: None, query: Some(query), stmts: Vec::new(), extended_by: Vec::new() }
    }
    /// 检查此 frame 是否匹配给定的 extend target（精确匹配每个逗号分隔项）
    pub(super) fn matches_target(&self, target: &str) -> bool {
        match &self.selector {
            Some(sel) => {
                let entries: Vec<&str> = sel.split(',').map(str::trim).collect();
                entries.iter().any(|e| *e == target || *e == format!("{} ", target).trim())
            }
            None => false,
        }
    }
}

// ── 入口函数 ─────────────────────────────────────────────────────────────────

/// 入口: 响应式求值管线 — 直接消费 AstStream
///
/// 纯 rxrust 算子链 (scan_map &mut 零 clone):
///   ast_stream
///       .flat_map(emit_events)            ← 展开 AST 节点为事件流
///       .scan_map(EvalState::root(), fold_frame)  ← &mut EvalState, 零 clone
///       .filter_map(|opt| opt)            ← Option<CssStmt> 透传
///       .box_it()                         ← 唯一的类型擦除边界
///
/// ⚠️ 消除 collect_boxed(ast_stream) 断裂点：直接消费上游 AstStream
pub fn eval_stream(ast_stream: AstStream, ctx: Arc<EvalContext>) -> CssStream {
    let bus = ctx.bus().clone();
    ast_stream
        .flat_map(move |node| emit_events(node, None, ctx.clone(), bus.clone()))
        .scan_map(EvalState::root(), fold_frame)
        .filter_map(|opt| opt)
        .box_it()
}

// ── fold_frame: scan_map 闭包 (&mut EvalState, EvalEvent) -> Option<CssStmt> ──

/// scan_map 算子的折叠函数: Fn(&mut EvalState, EvalEvent) -> Option<CssStmt>
///
/// 替代原来的 `for event { state = fold_frames(state, ev) }` 命令式循环。
/// scan_map 算子 (scan_map.rs L65-67) 通过 &mut self.acc 直接修改状态，零 clone。
/// 返回 Option<CssStmt>: Some 表示顶层 frame 完成，None 表示中间状态。
fn fold_frame(state: &mut EvalState, event: EvalEvent) -> Option<CssStmt> {
    if matches!(event, EvalEvent::EnterRule(_) | EvalEvent::LeaveRule) {
        tracing::trace!(?event, frames = state.frames.len(), "fold_frame");
    }
    match event {
        EvalEvent::EnterRule(sel) => {
            state.frames.push(Frame::new_rule(sel));
            None
        }
        EvalEvent::LeaveRule => {
            let frame = state.frames.pop().expect("unbalanced LeaveRule");
            let final_selector = apply_frame_extends(state, &frame);
            let rule = CssStmt::Rule { selector: final_selector, inner: frame.stmts };
            let is_root = state.frames.len() == 1;
            state.frames.last_mut().unwrap().stmts.push(rule.clone());
            if is_root { Some(rule) } else { None }
        }
        EvalEvent::EnterMedia(query) => {
            state.frames.push(Frame::new_media(query));
            None
        }
        EvalEvent::LeaveMedia => {
            let frame = state.frames.pop().expect("unbalanced LeaveMedia");
            let media = CssStmt::Media { query: frame.query.unwrap(), inner: frame.stmts };
            let is_root = state.frames.len() == 1;
            state.frames.last_mut().unwrap().stmts.push(media.clone());
            if is_root { Some(media) } else { None }
        }
        EvalEvent::EnterSupports(query) => {
            state.frames.push(Frame::new_supports(query));
            None
        }
        EvalEvent::LeaveSupports => {
            let frame = state.frames.pop().expect("unbalanced LeaveSupports");
            let supports = CssStmt::Supports { query: frame.query.unwrap(), inner: frame.stmts };
            let is_root = state.frames.len() == 1;
            state.frames.last_mut().unwrap().stmts.push(supports.clone());
            if is_root { Some(supports) } else { None }
        }
        EvalEvent::Terminal(stmt) => {
            state.frames.last_mut().unwrap().stmts.push(stmt.clone());
            // 顶层声明也作为 completed 输出
            if state.frames.len() == 1 { Some(stmt) } else { None }
        }
        EvalEvent::AddSelector { target, source } => {
            // 尝试应用到当前已打开的 frame
            let mut applied = false;
            for frame in state.frames.iter_mut() {
                if frame.matches_target(&target) {
                    frame.extended_by.push(source.clone());
                    applied = true;
                }
            }
            if !applied {
                // 递归搜索已关闭的规则
                if find_and_extend_selector(&mut state.frames, &target, &source) {
                    applied = true;
                }
            }
            if !applied {
                state.pending_extends.push((target, source));
            }
            None
        }
    }
}

/// Recursively find a rule matching `target` selector in frame tree and append `source` to its selector.
/// Returns true if found and modified.
fn find_and_extend_selector(frames: &mut [Frame], target: &str, source: &str) -> bool {
    tracing::trace!(target = ?target, source = %source, frames_len = frames.len(), "find_and_extend_selector");
    for (i, frame) in frames.iter_mut().enumerate() {
        tracing::trace!(i = i, kind = ?frame.kind, sel = ?frame.stmts.len(), "checking frame");
        if extend_selector_in_stmts(&mut frame.stmts, target, source) {
            return true;
        }
    }
    tracing::trace!("find_and_extend_selector: NOT FOUND");
    false
}

/// Recursively search stmts for a Rule matching target, append source to its selector.
/// Uses exact match per comma-separated entry. Deduplicates: does not append `source` if already present.
fn extend_selector_in_stmts(stmts: &mut [CssStmt], target: &str, source: &str) -> bool {
    for stmt in stmts.iter_mut() {
        match stmt {
            CssStmt::Rule { selector, inner } => {
                let entries: Vec<&str> = selector.split(',').map(str::trim).collect();
                if entries.contains(&target) {
                    if !entries.contains(&source) {
                        *selector = format!("{}, {}", source, selector);
                    }
                    return true;
                }
                if extend_selector_in_stmts(inner, target, source) {
                    return true;
                }
            }
            CssStmt::Media { inner, .. } | CssStmt::Supports { inner, .. } => {
                if extend_selector_in_stmts(inner, target, source) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// 应用 frame 的 extends，返回合并后的 selector
fn apply_frame_extends(state: &mut EvalState, frame: &Frame) -> String {
    let matching: Vec<String> = state.pending_extends.iter()
        .filter(|(target, _)| frame.matches_target(target))
        .map(|(_, source)| source.clone())
        .collect();
    state.pending_extends.retain(|(target, _)| !frame.matches_target(target));

    let all_extends: Vec<String> = frame.extended_by.iter()
        .chain(matching.iter())
        .cloned()
        .collect();

    if all_extends.is_empty() {
        frame.selector.clone().unwrap_or_default()
    } else {
        let mut combined = all_extends;
        combined.push(frame.selector.clone().unwrap_or_default());
        combined.join(", ")
    }
}
