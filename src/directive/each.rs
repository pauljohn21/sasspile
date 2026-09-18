use std::convert::Infallible;

use rxrust::prelude::*;
use tracing::{info_span, Span};

#[derive(Clone)]
pub struct EachOp<S> {
    pub source: S,
}

/// @each 状态机
///
/// 流转:
///   Idle ──@each sig──> Collecting{body, depth=0} ──{──> depth++ ... ──} depth=0──> expand
///   Idle ──SASS @each sig+body──> 立即展开 (同 token)
#[derive(Clone, Debug, Default)]
enum EachState {
    #[default]
    Idle,
    Collecting {
        var_name: String,
        items: Vec<String>,
        body: Vec<String>,
        depth: usize,
    },
}

#[derive(Clone, Debug)]
struct State {
    inner: EachState,
}

impl Default for State {
    fn default() -> Self {
        Self { inner: EachState::Idle }
    }
}

/// 解析 @each 签名: `@each $var in item1, item2, item3` 或 `@each $var in (a, b, c)`
fn parse_each_sig(s: &str) -> Option<(String, Vec<String>)> {
    let s = s.trim();
    let s = s.strip_prefix("@each")?.trim();

    // 提取 $var
    let (var_name, rest) = s.split_once(' ')?;
    let var_name = var_name.trim().to_string();
    let rest = rest.trim();

    // 期望 `in ...`
    let rest = rest.strip_prefix("in")?.trim();

    // 去除括号
    let rest = rest.trim().trim_start_matches('(').trim_end_matches(')').trim();

    // 逗号分隔列表
    let items: Vec<String> = rest
        .split(',')
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect();

    if items.is_empty() {
        return None;
    }

    Some((var_name, items))
}

/// 展开 body tokens: 每个 item 产生一份 body, 替换 $var
fn expand_body(body: &[String], var_name: &str, value: &str) -> Vec<String> {
    body.iter().map(|tok| tok.replace(var_name, value)).collect()
}

/// Idle 状态处理
fn handle_idle(token: String) -> (State, Vec<String>) {
    let trimmed = token.trim();

    // SASS 格式判定: token 含 @each 且签名行之后还有 body 内容
    // SCSS 格式的 @each token 仅含签名 (后面紧跟独立的 `{` token)
    if let Some(idx) = token.find("@each") {
        let (prefix, rest) = token.split_at(idx);
        if let Some(sig_line_end) = rest.find('\n') {
            let (sig_part, body_part) = rest.split_at(sig_line_end);
            // 签名行之后必须跟有 body (非空内容) 才视为 SASS 格式
            if !body_part.trim().is_empty() {
                if let Some((var_name, items)) = parse_each_sig(sig_part) {
                    let body = vec![body_part.trim().to_string()];
                    let expanded: Vec<String> = items
                        .iter()
                        .flat_map(|v| expand_body(&body, &var_name, v))
                        .collect();
                    let mut outputs = Vec::new();
                    if !prefix.trim().is_empty() {
                        outputs.push(prefix.trim_end().to_string());
                    }
                    outputs.extend(expanded);
                    return (State::default(), outputs);
                }
            }
        }
    }

    // SCSS 格式: token 纯签名
    if let Some((var_name, items)) = parse_each_sig(trimmed) {
        return (
            State {
                inner: EachState::Collecting { var_name, items, body: vec![], depth: 0 },
            },
            vec![],
        );
    }

    // 不匹配, 透传
    (State::default(), vec![token])
}

/// 收集 body 状态处理
fn handle_collecting(
    var_name: String,
    items: Vec<String>,
    mut body: Vec<String>,
    depth: usize,
    token: String,
) -> (State, Vec<String>) {
    match token.trim() {
        "{" => {
            if depth == 0 {
                (
                    State {
                        inner: EachState::Collecting { var_name, items, body, depth: 1 },
                    },
                    vec![],
                )
            } else {
                body.push(token);
                (
                    State {
                        inner: EachState::Collecting { var_name, items, body, depth: depth + 1 },
                    },
                    vec![],
                )
            }
        }
        "}" => {
            if depth <= 1 {
                let expanded: Vec<String> =
                    items.iter().flat_map(|v| expand_body(&body, &var_name, v)).collect();
                (State::default(), expanded)
            } else {
                body.push(token);
                (
                    State {
                        inner: EachState::Collecting { var_name, items, body, depth: depth - 1 },
                    },
                    vec![],
                )
            }
        }
        _ => {
            body.push(token);
            (
                State {
                    inner: EachState::Collecting { var_name, items, body, depth },
                },
                vec![],
            )
        }
    }
}

impl<S> ObservableType for EachOp<S>
where
    S: ObservableType,
{
    type Item<'a>
        = String
    where
        Self: 'a;
    type Err = S::Err;
}

impl<S, C> CoreObservable<C> for EachOp<S>
where
    C: Context,
    S: CoreObservable<C::With<EachObserver<C::Inner>>>,
    C::Inner: Send,
{
    type Unsub = S::Unsub;

    fn subscribe(self, context: C) -> Self::Unsub {
        let wrapped = context.transform(|observer| EachObserver {
            observer,
            state: State::default(),
        });
        self.source.subscribe(wrapped)
    }
}

#[derive(Clone)]
pub struct EachObserver<O> {
    observer: O,
    state: State,
}

impl<O> Observer<String, Infallible> for EachObserver<O>
where
    O: Observer<String, Infallible> + Send,
{
    fn next(&mut self, value: String) {
        let span = info_span!("each_op", token = %value);
        let _enter = span.enter();

        let state = self.state.clone();
        let (new_state, outputs) = match state.inner {
            EachState::Idle => handle_idle(value),
            EachState::Collecting { var_name, items, body, depth } => {
                handle_collecting(var_name, items, body, depth, value)
            }
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
