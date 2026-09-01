//! 语法分析器——递归下降 + Pratt 表达式解析。

pub mod ast;
pub mod at_rule_kinds;
mod ast_impl;

use crate::error::{Result, SassError};
use crate::lex::token::Token;
use ast::*;
use crate::__tracing::warn;

/// 语法分析器。
pub struct Parser<'tok> {
    tokens: &'tok [Token],
    pos: usize,
    /// 是否在规则体（mixin/function/style_rule）内——用于 @forward 上下文验证。
    in_body: bool,
    /// 是否已解析过非模块规则（非 @forward/@use/@import）。
    saw_other_rule: bool,
}

impl<'tok> Parser<'tok> {
    /// 创建新的 Parser。
    pub fn new(tokens: &'tok [Token]) -> Self {
        Self { tokens, pos: 0, in_body: false, saw_other_rule: false }
    }

    /// 解析入口。
    pub fn parse(tokens: &'tok [Token]) -> Result<Ast> {
        let mut p = Self::new(tokens);
        let mut nodes = Vec::new();
        while !p.at_end() {
            p.skip_ws();
            if p.at_end() {
                break;
            }
            let node = p.parse_node()?;
            // 跟踪非模块规则——@forward 必须在这些规则之前
            match &node {
                Node::Forward { .. } | Node::Use { .. } | Node::Import { .. } => {}
                _ => p.saw_other_rule = true,
            }
            nodes.push(node);
        }
        Ok(Ast { nodes })
    }

    // —— 基础操作 ——
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }
    #[allow(dead_code)]
    fn peek_n(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.pos + n)
    }
    fn advance(&mut self) -> Option<&Token> {
        let t = self.tokens.get(self.pos);
        if !matches!(t, Some(Token::Eof) | None) {
            self.pos += 1;
        }
        t
    }
    fn at_end(&self) -> bool {
        let mut i = self.pos;
        while matches!(self.tokens.get(i), Some(Token::Whitespace)) {
            i += 1;
        }
        matches!(self.tokens.get(i), None | Some(Token::Eof))
    }
fn skip_ws(&mut self) {
while let Some(tok) = self.peek() {
match tok {
Token::Whitespace => self.pos += 1,
// 静默注释 (//) 在 skip_ws 中跳过；块注释 (/* */) 保留由 parse_node 处理
Token::Comment(_, true) => self.pos += 1,
_ => break,
}
}
}

/// 跳过所有空白和注释（含块注释）。
fn skip_ws_and_comments(&mut self) {
while let Some(tok) = self.peek() {
match tok {
Token::Whitespace | Token::Comment(_, _) => self.pos += 1,
_ => break,
}
}
}

    fn expect(&mut self, tok: &Token) -> Result<()> {
        self.skip_ws();
        if self.peek() == Some(tok) {
            self.advance();
            Ok(())
        } else {
            let found = self.peek().map(|t| t.to_string()).unwrap_or("EOF".into());
            let ctx_start = self.pos.saturating_sub(5);
            let ctx_end = (self.pos + 5).min(self.tokens.len());
            let context: Vec<String> = self.tokens[ctx_start..ctx_end]
                .iter()
                .map(|t| t.to_string())
                .collect();
            warn!(expected = %tok, found = %found, pos = self.pos, context = ?context, "expect failed");
            Err(SassError::Parse {
                expected: tok.to_string(),
                found,
            })
        }
    }
}

mod at_rules;
mod at_rules_modules;
mod expr;
mod nodes;
mod params;
