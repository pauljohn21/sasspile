use std::convert::Infallible;

use rxrust::prelude::*;
use tracing::info_span;

#[derive(Clone)]
pub struct ForOp<S> {
    pub source: S,
}

#[derive(Clone, Debug, Default)]
enum ForState {
    #[default]
    Idle,
    Collecting {
        var_name: String,
        values: Vec<String>,
        body: Vec<String>,
        depth: usize,
    },
}

#[derive(Clone, Debug, Default)]
struct State {
    inner: ForState,
}

/// 解析 @for 签名: `@for $var from N through M` 或 `@for $var from N to M`
fn parse_for_sig(s: &str) -> Option<(String, Vec<String>)> {
    let s = s.trim();
    let s = s.strip_prefix("@for")?.trim();
    let (var_name, rest) = s.split_once(' ')?;
    let var_name = var_name.trim().to_string();
    let rest = rest.trim();
    let rest = rest.strip_prefix("from")?.trim();

    if let Some((from_str, to_str)) = rest.split_once("through") {
        let from: i64 = from_str.trim().parse().ok()?;
        let to: i64 = to_str.trim().parse().ok()?;
        Some((var_name, generate_range(from, to, true)))
    } else if let Some((from_str, to_str)) = rest.split_once(" to ") {
        let from: i64 = from_str.trim().parse().ok()?;
        let to: i64 = to_str.trim().parse().ok()?;
        Some((var_name, generate_range(from, to, false)))
    } else {
        None
    }
}

fn generate_range(from: i64, to: i64, inclusive: bool) -> Vec<String> {
    let mut values = Vec::new();
    if from <= to {
        let end = if inclusive { to + 1 } else { to };
        for i in from..end {
            values.push(i.to_string());
        }
    } else {
        let end = if inclusive { to - 1 } else { to };
        let mut i = from;
        while i > end {
            values.push(i.to_string());
            i -= 1;
        }
    }
    values
}

fn expand_body(body: &[String], var_name: &str, value: &str) -> Vec<String> {
    body.iter().map(|tok| tok.replace(var_name, value)).collect()
}

fn handle_idle(token: String) -> (State, Vec<String>) {
    let trimmed = token.trim();

    // SASS 格式判定: token 含 @for 且签名行之后还有 body 内容
    // SCSS 格式的 @for token 仅含签名 (后面紧跟独立的 `{` token)
    if let Some(idx) = token.find("@for") {
        let (prefix, rest) = token.split_at(idx);
        // 找签名行的结尾 (\n)
        if let Some(sig_line_end) = rest.find('\n') {
            let (sig_part, body_part) = rest.split_at(sig_line_end);
            // 签名行之后必须跟有 body (非空内容) 才视为 SASS 格式
            if !body_part.trim().is_empty() {
                if let Some((var_name, values)) = parse_for_sig(sig_part) {
                    let body = vec![body_part.trim().to_string()];
                    let expanded: Vec<String> = values
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

    // SCSS 格式: 纯签名 token
    if let Some((var_name, values)) = parse_for_sig(trimmed) {
        return (
            State {
                inner: ForState::Collecting {
                    var_name,
                    values,
                    body: vec![],
                    depth: 0,
                },
            },
            vec![],
        );
    }

    (State::default(), vec![token])
}

fn handle_collecting(
    var_name: String,
    values: Vec<String>,
    mut body: Vec<String>,
    depth: usize,
    token: String,
) -> (State, Vec<String>) {
    match token.trim() {
        "{" => {
            let new_depth = if depth == 0 { 1 } else { depth + 1 };
            (
                State {
                    inner: ForState::Collecting {
                        var_name,
                        values,
                        body,
                        depth: new_depth,
                    },
                },
                vec![],
            )
        }
        "}" => {
            if depth <= 1 {
                let expanded: Vec<String> = values
                    .iter()
                    .flat_map(|v| expand_body(&body, &var_name, v))
                    .collect();
                (State::default(), expanded)
            } else {
                body.push(token);
                (
                    State {
                        inner: ForState::Collecting {
                            var_name,
                            values,
                            body,
                            depth: depth - 1,
                        },
                    },
                    vec![],
                )
            }
        }
        _ => {
            if depth > 0 {
                body.push(token);
            }
            (
                State {
                    inner: ForState::Collecting {
                        var_name,
                        values,
                        body,
                        depth,
                    },
                },
                vec![],
            )
        }
    }
}

impl<S> ObservableType for ForOp<S>
where
    S: ObservableType,
{
    type Item<'a>
        = String
    where
        Self: 'a;
    type Err = S::Err;
}

impl<S, C> CoreObservable<C> for ForOp<S>
where
    C: Context,
    S: CoreObservable<C::With<ForObserver<C::Inner>>>,
    C::Inner: Send,
{
    type Unsub = S::Unsub;

    fn subscribe(self, context: C) -> Self::Unsub {
        let wrapped = context.transform(|observer| ForObserver {
            observer,
            state: State::default(),
        });
        self.source.subscribe(wrapped)
    }
}

#[derive(Clone)]
pub struct ForObserver<O> {
    observer: O,
    state: State,
}

impl<O> Observer<String, Infallible> for ForObserver<O>
where
    O: Observer<String, Infallible> + Send,
{
    fn next(&mut self, value: String) {
        let span = info_span!("for_op", token = %value);
        let _enter = span.enter();

        let state = self.state.clone();
        let (new_state, outputs) = match state.inner {
            ForState::Idle => handle_idle(value),
            ForState::Collecting { var_name, values, body, depth } => {
                handle_collecting(var_name, values, body, depth, value)
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
