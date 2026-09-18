use std::convert::Infallible;

use rxrust::prelude::*;
use tracing::{info_span, Span};

#[derive(Clone)]
pub struct IfOp<S> {
    pub source: S,
}

/// @if 状态机
///
/// 流转:
///   Idle ──@if cond──> AwaitingBrace{predicate} ──{──> InBlock{branch_taken}
///   InBlock ──} depth=0──> AfterBlock ──@else──> AwaitingBrace{!prev}
///   InBlock ──} depth=0──> AfterBlock ──@else if cond──> AwaitingBrace{predicate}
///   AfterBlock ──其他──> Idle (透传)
#[derive(Clone, Debug, Default)]
enum IfState {
    #[default]
    Idle,
    AwaitingBrace { predicate: bool },
    InBlock { depth: usize, branch_taken: bool },
    AfterBlock { prev_taken: bool },
}

#[derive(Clone, Debug)]
struct State {
    inner: IfState,
}

impl Default for State {
    fn default() -> Self {
        Self { inner: IfState::Idle }
    }
}

/// 解析 @if 谓词
///
/// 支持简单字面量: "true" / "false"
/// 未来可扩展变量查表 / 比较运算
fn parse_predicate(cond: &str) -> bool {
    let cond = cond.trim();
    match cond {
        "true" => true,
        "false" => false,
        _ => {
            // 非明确 false 的值暂按 truthy 处理
            !cond.is_empty()
        }
    }
}

/// Idle 状态处理
fn handle_idle(token: String) -> (State, Vec<String>) {
    let trimmed = token.trim();
    if let Some(cond) = trimmed.strip_prefix("@if") {
        let cond = cond.trim();
        let predicate = parse_predicate(cond);
        return (State { inner: IfState::AwaitingBrace { predicate } }, vec![]);
    }
    // 非 @if 指令, 透传
    (State::default(), vec![token])
}

/// 等待 { 状态
fn handle_awaiting_brace(predicate: bool, token: String) -> (State, Vec<String>) {
    match token.trim() {
        "{" => (
            State { inner: IfState::InBlock { depth: 1, branch_taken: predicate } },
            vec![],
        ),
        // 非 { 字符(如注释), 保持等待状态但产出空
        _ => (State { inner: IfState::AwaitingBrace { predicate } }, vec![]),
    }
}

/// 在 {} 块内, 跟踪嵌套深度
fn handle_in_block(depth: usize, branch_taken: bool, token: String) -> (State, Vec<String>) {
    match token.trim() {
        "{" => (
            State { inner: IfState::InBlock { depth: depth + 1, branch_taken } },
            vec![],
        ),
        "}" => {
            if depth <= 1 {
                (State { inner: IfState::AfterBlock { prev_taken: branch_taken } }, vec![])
            } else {
                (State { inner: IfState::InBlock { depth: depth - 1, branch_taken } }, vec![])
            }
        }
        // 普通 token
        _ => {
            if branch_taken {
                (State { inner: IfState::InBlock { depth, branch_taken } }, vec![token])
            } else {
                (State { inner: IfState::InBlock { depth, branch_taken } }, vec![])
            }
        }
    }
}

/// 块结束后, 等待 @else / @else if
fn handle_after_block(prev_taken: bool, token: String) -> (State, Vec<String>) {
    let trimmed = token.trim();

    if let Some(rest) = trimmed.strip_prefix("@else") {
        let rest = rest.trim();

        // 纯 @else
        if rest.is_empty() {
            if prev_taken {
                // 前一支已取, else 块将跳过; 但需进入 InBlock 以消费 body
                // 用 AwaitingBrace{false} 等待 {
                return (State { inner: IfState::AwaitingBrace { predicate: false } }, vec![]);
            }
            // 前一支未取, else 块将被执行
            return (State { inner: IfState::AwaitingBrace { predicate: true } }, vec![]);
        }

        // @else if <cond>
        if let Some(cond) = rest.strip_prefix("if") {
            let cond = cond.trim();
            if prev_taken {
                // 前一支已取, else if 必然跳过
                return (State { inner: IfState::AwaitingBrace { predicate: false } }, vec![]);
            }
            return (
                State { inner: IfState::AwaitingBrace { predicate: parse_predicate(cond) } },
                vec![],
            );
        }
    }

    // 非 @else 指令, 回到 Idle 并透传该 token
    let (mut state, mut out) = handle_idle(token);
    // 但如果 token 产生新的 Idle (未命中 @if), 需合并透传结果
    if matches!(state.inner, IfState::Idle) {
        return (state, out);
    }
    // token 本身触发了新的 @if: 透传原来的 token 作为前置
    // 这种情况极少, 暂时将 token 交由下一轮处理
    (State::default(), out)
}

impl<S> ObservableType for IfOp<S>
where
    S: ObservableType,
{
    type Item<'a>
        = String
    where
        Self: 'a;
    type Err = S::Err;
}

impl<S, C> CoreObservable<C> for IfOp<S>
where
    C: Context,
    S: CoreObservable<C::With<IfObserver<C::Inner>>>,
    C::Inner: Send,
{
    type Unsub = S::Unsub;

    fn subscribe(self, context: C) -> Self::Unsub {
        let wrapped = context.transform(|observer| IfObserver {
            observer,
            state: State::default(),
        });
        self.source.subscribe(wrapped)
    }
}

#[derive(Clone)]
pub struct IfObserver<O> {
    observer: O,
    state: State,
}

impl<O> Observer<String, Infallible> for IfObserver<O>
where
    O: Observer<String, Infallible> + Send,
{
    fn next(&mut self, value: String) {
        let span = info_span!("if_op", token = %value);
        let _enter = span.enter();

        let state = self.state.clone();
        let (new_state, outputs) = match state.inner {
            IfState::Idle => handle_idle(value),
            IfState::AwaitingBrace { predicate } => handle_awaiting_brace(predicate, value),
            IfState::InBlock { depth, branch_taken } => handle_in_block(depth, branch_taken, value),
            IfState::AfterBlock { prev_taken } => handle_after_block(prev_taken, value),
        };
        self.state = new_state;
        outputs.into_iter().for_each(|tok| {
            if !self.observer.is_closed() {
                self.observer.next(tok);
            }
        });
    }

    fn error(self, err: Infallible) {
        match err {}
    }

    fn complete(self) {
        self.observer.complete();
    }

    fn is_closed(&self) -> bool {
        self.observer.is_closed()
    }
}
