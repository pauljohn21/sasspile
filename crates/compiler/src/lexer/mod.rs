//! Lexer — 将字符流转为 Token 流
//!
//! 使用 `scan(LexerState)` + `flat_map` 实现流式词法分析，
//! 内含换行符标准化与多字符 Token 识别（变量、标识符、数字等）。

mod token;
mod state;

pub use token::{Token, TokenKind};
pub use state::LexerState;

use rxrust::prelude::*;
use std::convert::Infallible;

/// Token stream type (local, boxed, cloneable)
pub type TokenStream = LocalBoxedObservableClone<'static, Token, Infallible>;

/// Char stream type (local, boxed, cloneable)
pub type CharStream = LocalBoxedObservableClone<'static, char, Infallible>;

/// Lexer accumulator
#[derive(Debug, Clone)]
struct LexerAcc {
    state: LexerState,
    pending: Option<String>,
    pending_is_var: bool,
}

impl LexerAcc {
    fn new() -> Self {
        Self {
            state: LexerState::new(),
            pending: None,
            pending_is_var: false,
        }
    }

    /// Flush pending identifier/variable into a Token
    fn flush_pending(&mut self) -> Option<Token> {
        if let Some(text) = self.pending.take() {
            let kind = if self.pending_is_var {
                TokenKind::Variable
            } else if text.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-') {
                TokenKind::Number
            } else {
                TokenKind::Ident
            };
            let tok = Token::with_pos(
                kind,
                self.state.pos - text.len() as u32,
                text,
                self.state.line,
                self.state.col,
            );
            self.pending_is_var = false;
            Some(tok)
        } else {
            None
        }
    }
}

/// 词法分析入口：将字符流转为 Token 流
pub fn lex(input: CharStream) -> TokenStream {
    type Acc = (LexerAcc, Vec<Token>);
    let init: Acc = (LexerAcc::new(), Vec::new());

    input
        .scan(init, |acc, c| {
            // Take ownership of acc's fields (scan passes by value)
            let mut lex_acc = acc.0.clone();
            // Only emit NEW tokens for this step (not accumulated from previous steps)
            let mut emit = Vec::new();

            // Newline normalization
            let ch = match c {
                '\r' | '\x0C' => '\n',
                _ => c,
            };
            lex_acc.state.advance(ch);

            let is_ident_ext = ch.is_alphanumeric() || ch == '-' || ch == '_';
            let is_var_start = ch == '$';

            if is_var_start && lex_acc.pending.is_none() {
                lex_acc.pending = Some("$".to_string());
                lex_acc.pending_is_var = true;
            } else if is_var_start && lex_acc.pending.is_some() {
                if let Some(tok) = lex_acc.flush_pending() {
                    emit.push(tok);
                }
                lex_acc.pending = Some("$".to_string());
                lex_acc.pending_is_var = true;
            } else if is_ident_ext {
                // Start or extend identifier
                if lex_acc.pending.is_none() {
                    lex_acc.pending = Some(ch.to_string());
                } else if let Some(ref mut s) = lex_acc.pending {
                    s.push(ch);
                }
            } else {
                if let Some(tok) = lex_acc.flush_pending() {
                    emit.push(tok);
                }
                let kind = match ch {
                    '&' => TokenKind::ParentSelector,
                    _ => TokenKind::Char(ch),
                };
                emit.push(Token::with_pos(
                    kind,
                    lex_acc.state.pos - 1,
                    ch.to_string(),
                    lex_acc.state.line,
                    lex_acc.state.col,
                ));
            }

            (lex_acc, emit)
        })
        .flat_map(|(_, tokens)| Local::from_iter(tokens))
        .box_it_clone()
}

/// Helper: normalize newlines in a string slice.
pub fn normalize_newlines(input: &str) -> String {
    input
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\x0C', "\n")
}
