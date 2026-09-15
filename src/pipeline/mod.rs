//! 反应式流管线 —— 编译器 = 流的变换链
//!
//! ```text
//! Source chars ──scan──▶ LexerScanner ──flat_map──▶ Tokens ──flat_map──▶ CSS chars
//!               tokenize                     extract             serialize
//!
//! scan: acc = LexerScanner { state, emitted }, 每步产出 emitted tokens (0..N)
//! flat_map: 将 Vec<Token> 展开为单个 Token 流
//! flat_map: Token → rendered chars
//! ```

mod otel_op;

pub use otel_op::otel_span;

use rxrust::prelude::*;
use std::convert::Infallible;
use crate::lex::stream::{feed, LexerState};
use crate::lex::token::Token;

#[derive(Clone)]
struct LexerScanner {
    state: LexerState,
    emitted: Vec<Token>,
}

impl LexerScanner {
    fn new() -> Self {
        Self { state: LexerState::default(), emitted: Vec::new() }
    }

    fn feed(mut self, ch: char) -> Self {
        let (new_state, tokens) = feed(self.state, ch);
        Self { state: new_state, emitted: tokens }
    }
}

/// 编译管线：源码字符串 → CSS 字符流
pub fn compile(input: &str) -> LocalBoxedObservable<'static, char, Infallible> {
    // 追加空白字符以 flush 最后累积的多字符 token（ident/number/string）
    let mut chars: Vec<char> = input.chars().collect();
    chars.push(' ');

    Local::from_iter(chars)
        .scan(LexerScanner::new(), |scanner, ch| scanner.feed(ch))
        .flat_map(|scanner| Local::from_iter(scanner.emitted))
        .flat_map(|token| Local::from_iter(token.to_string().chars().collect::<Vec<_>>()))
        .box_it()
}

/// pipe 辅助：将 Observable 通过一个变换函数
pub fn pipe<T, F, R>(obs: T, f: F) -> R
where
    F: FnOnce(T) -> R,
{
    f(obs)
}
