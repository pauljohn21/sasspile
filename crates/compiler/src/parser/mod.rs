//! Parser — 将 Token 流转为 SassAstNode（解析树）流
//!
//! 使用 `scan(ParserState)` 实现流式语法分析。

mod ast;
mod state;
mod syntax;

pub use ast::SassAstNode;
pub use syntax::InputSyntax;
pub use state::ParserState;

use rxrust::prelude::*;
use std::convert::Infallible;
use crate::lexer::{Token, TokenKind};

/// SassAstNode stream type
pub type SassAstStream = LocalBoxedObservableClone<'static, SassAstNode, Infallible>;

/// 语法分析入口：将 Token 流转为 SassAstNode 流
///
/// 使用 `scan(ParserState)` 累积 Token 缓冲区 + 解析状态，
/// `scan` 每步输出刚完成解析的节点列表（0 个或多个），
/// `flat_map` 将其展平为单个节点流。
pub fn parse(input: LocalBoxedObservableClone<'static, Token, Infallible>) -> SassAstStream {
    let init: (ParserState, Vec<SassAstNode>) = (ParserState::new(), Vec::new());

    input
        .scan(init, |acc, tok| {
            let (mut state, _buf) = acc;
            state.push_token(tok.clone());

            // Try to parse as many statements as possible from current state
            let mut nodes = Vec::new();
            while let Some(node) = try_parse_statement(&mut state) {
                nodes.push(node);
            }
            // Each step emits only the NEW nodes parsed this step;
            // `flat_map` flattens Vec<SassAstNode> into individual nodes.
            (state.clone(), nodes)
        })
        .flat_map(|(_, nodes)| Local::from_iter(nodes))
        .box_it_clone()
}

/// Try to parse one statement from the current state.
/// Returns Some(node) if a statement was parsed (and consumed tokens).
/// Returns None if no complete statement is available yet.
fn try_parse_statement(state: &mut ParserState) -> Option<SassAstNode> {
    // Scan for terminator: ; or }
    let mut terminator = None;
    let len = state.tokens_len();
    for i in 0..len {
        if let Some(tok) = state.peek_n(i) {
            if tok.char_kind() == Some(';') || tok.char_kind() == Some('}') {
                terminator = Some(i);
                break;
            }
        }
    }

    let end_pos = terminator?;

    // Collect tokens for this statement [0, end_pos] (inclusive of terminator)
    let mut stmt_tokens = Vec::new();
    for _ in 0..=end_pos {
        if let Some(tok) = state.peek() {
            stmt_tokens.push(tok.clone());
            state.advance();
        }
    }

    if stmt_tokens.is_empty() {
        return None;
    }

    // ── Variable declaration: $name: value; ──
    if stmt_tokens.len() >= 3 && stmt_tokens[0].kind == TokenKind::Variable {
        if stmt_tokens[1].char_kind() == Some(':') {
            let name = stmt_tokens[0].text.clone();
            let value_part: Vec<Token> = stmt_tokens[2..]
                .iter()
                .filter(|t| t.char_kind() != Some(';'))
                .cloned()
                .collect();
            let value_text: String = value_part.iter().map(|t| t.text.as_str()).collect();

            // Check for !default
            let has_default = stmt_tokens.iter().any(|t| {
                t.kind == TokenKind::Ident && t.text == "default"
            }) || stmt_tokens.iter().any(|t| t.text.contains("default"));

            return Some(SassAstNode::VariableDecl {
                name,
                value: Box::new(SassAstNode::Raw(value_text.trim().to_string())),
                has_default,
            });
        }
    }

    // ── Rule: selector { ... } ──
    if let Some(brace_pos) = stmt_tokens.iter().position(|t| t.char_kind() == Some('{')) {
        if brace_pos > 0 {
            let selector: String = stmt_tokens[..brace_pos]
                .iter()
                .map(|t| t.text.as_str())
                .collect();

            let inner_part: Vec<Token> = stmt_tokens[brace_pos + 1..]
                .iter()
                .filter(|t| t.char_kind() != Some('}') && t.char_kind() != Some(';'))
                .cloned()
                .collect();
            let inner_text: String = inner_part.iter().map(|t| t.text.as_str()).collect();

            let inner = if inner_text.trim().is_empty() {
                vec![]
            } else {
                vec![SassAstNode::Raw(inner_text)]
            };

            return Some(SassAstNode::Rule {
                selector: selector.trim().to_string(),
                inner,
            });
        }
    }

    // ── Style declaration: prop: value; ──
    if stmt_tokens.len() >= 3 {
        if let Some(colon_pos) = stmt_tokens.iter().position(|t| t.char_kind() == Some(':')) {
            if colon_pos > 0 {
                let prop: String = stmt_tokens[..colon_pos]
                    .iter()
                    .map(|t| t.text.as_str())
                    .collect();
                let value: String = stmt_tokens[colon_pos + 1..]
                    .iter()
                    .filter(|t| t.char_kind() != Some(';'))
                    .map(|t| t.text.as_str())
                    .collect();

                if !prop.trim().is_empty() {
                    return Some(SassAstNode::StyleDecl {
                        prop: prop.trim().to_string(),
                        value: value.trim().to_string(),
                    });
                }
            }
        }
    }

    // ── Fallback: raw text ──
    let text: String = stmt_tokens.iter().map(|t| t.text.as_str()).collect();
    Some(SassAstNode::Raw(text))
}

/// Created by script — helper re-export for integration tests
pub use crate::lexer::normalize_newlines;
