//! Parser — parses token stream into AST.
//!
//! Organized into:
//! - `mod.rs` — main entry + top-level statements
//! - `expr.rs` — expression parsing with operator precedence
//! - `atrule.rs` — at-rule parsing
//! - `selector.rs` — selector and declaration parsing
//! - `helpers.rs` — text reading utilities

use crate::ast::*;
use crate::error::{SassError, SourcePos};
use crate::token::{Token, TokenSpan};
use tracing::instrument;

pub(crate) mod expr;
mod atrule;
mod helpers;
mod selector;

use expr::ExprParser;

/// Parse a token stream into a list of statements (the AST).
#[instrument(name = "parse", skip_all, fields(stage = "parser"))]
pub fn parse(tokens: Vec<TokenSpan>) -> Result<Vec<Stmt>, SassError> {
    let span = tracing::info_span!("parse", stage = "parser", token_count = tokens.len());
    let _enter = span.enter();

    let mut parser = Parser::new(tokens);
    let mut stmts = Vec::new();

    while !parser.is_at_end() {
        // Skip line comments at top level
        if matches!(parser.peek(), Token::LineComment(_)) {
            parser.advance();
            continue;
        }
        let stmt = parser.parse_stmt()?;
        stmts.push(stmt);
    }

    tracing::debug!(stage = "parser", stmt_count = stmts.len(), "parse complete");
    Ok(stmts)
}

/// The parser state.
pub struct Parser {
    pub(crate) tokens: Vec<TokenSpan>,
    pub(crate) pos: usize,
}

impl Parser {
    pub(crate) fn new(tokens: Vec<TokenSpan>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub(crate) fn peek(&self) -> &Token {
        &self.tokens[self.pos].token
    }

    pub(crate) fn peek_span(&self) -> &TokenSpan {
        &self.tokens[self.pos]
    }

    pub(crate) fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.pos += 1;
        }
        &self.tokens[self.pos.saturating_sub(1)].token
    }

    pub(crate) fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    pub(crate) fn current_pos(&self) -> SourcePos {
        self.peek_span().pos.clone()
    }

    pub(crate) fn expect(&mut self, expected: Token) -> Result<(), SassError> {
        if self.peek() == &expected {
            self.advance();
            Ok(())
        } else {
            Err(SassError::parse(
                format!("expected {:?} but got {:?}", expected, self.peek()),
                self.current_pos(),
            ))
        }
    }

    /// Check if the current position starts a CSS custom property (`--name: value`).
    /// This is the case when we see `--` (two consecutive Minus tokens).
    pub(crate) fn is_css_custom_property(&self) -> bool {
        if matches!(self.peek(), Token::Minus) {
            let next_idx = self.pos + 1;
            if let Some(ts) = self.tokens.get(next_idx) {
                return matches!(ts.token, Token::Minus | Token::InterpolationStart);
            }
        }
        false
    }

    /// Parse a single top-level statement.
    pub fn parse_stmt(&mut self) -> Result<Stmt, SassError> {
        match self.peek() {
            Token::Variable(_) => self.parse_variable_decl(),
            Token::Dot | Token::Ident(_) | Token::Ampersand | Token::Hash => self.parse_style_rule(),
            Token::Number(..) | Token::String(..) | Token::LParen => self.parse_style_rule(),
            Token::LBracket | Token::Colon | Token::Star | Token::Gt | Token::Percent | Token::Plus => self.parse_style_rule(),
            Token::AtRule(_) => self.parse_at_rule(),
            Token::BlockComment(_) => {
                let pos = self.current_pos();
                if let Token::BlockComment(text) = self.advance() {
                    Ok(Stmt::Comment(text.clone()))
                } else {
                    Err(SassError::parse("expected block comment", pos))
                }
            }
            Token::LineComment(_) => {
                self.advance();
                if self.is_at_end() {
                    return Ok(Stmt::Comment(String::new()));
                }
                self.parse_stmt()
            }
            Token::InterpolationStart => self.parse_style_rule(),
            Token::Eof => Err(SassError::parse("unexpected EOF", self.current_pos())),
            _ => Err(SassError::parse(
                format!("unexpected token {:?} in statement", self.peek()),
                self.current_pos(),
            )),
        }
    }

    /// Parse a variable declaration: `$name: value;`
    fn parse_variable_decl(&mut self) -> Result<Stmt, SassError> {
        let pos = self.current_pos();
        let name = match self.advance() {
            Token::Variable(n) => n.clone(),
            _ => return Err(SassError::parse("expected variable", pos)),
        };

        self.expect(Token::Colon)?;

        let mut default = false;
        let mut global = false;

        let value = {
            let mut expr_parser = ExprParser::new(self);
            expr_parser.parse_expr()?
        };

        loop {
            match self.peek() {
                Token::Ident(s) if s == "!default" => {
                    default = true;
                    self.advance();
                }
                Token::Ident(s) if s == "!global" => {
                    global = true;
                    self.advance();
                }
                _ => break,
            }
        }

        if matches!(self.peek(), Token::Semicolon) {
            self.advance();
        }

        Ok(Stmt::VariableDecl { name, value, default, global })
    }

    /// Parse a style rule: `selector { body }`
    fn parse_style_rule(&mut self) -> Result<Stmt, SassError> {
        let selector = self.parse_selector()?;
        self.expect(Token::LBrace)?;
        let body = self.parse_block_body()?;
        self.expect(Token::RBrace)?;
        Ok(Stmt::StyleRule { selector, body })
    }

    /// Parse a block body (list of statements until `}`)
    pub fn parse_block_body(&mut self) -> Result<Vec<Stmt>, SassError> {
        let span = tracing::trace_span!("parse_block_body", stage = "parser");
        let _enter = span.enter();
        let mut stmts = Vec::new();

        while !self.is_at_end() && !matches!(self.peek(), Token::RBrace) {
            let stmt = match self.peek() {
                Token::Ident(_) | Token::Dot | Token::InterpolationStart => {
                    if self.is_declaration_start() {
                        self.parse_declaration()
                    } else {
                        self.parse_style_rule()
                    }
                }
                Token::Variable(_) => self.parse_variable_decl(),
                Token::AtRule(_) => self.parse_at_rule(),
                Token::Colon | Token::LBracket | Token::Ampersand | Token::Number(..) | Token::String(..) | Token::Star | Token::Gt | Token::Percent | Token::Plus => self.parse_style_rule(),
                Token::Minus => {
                    // Check if this is a CSS custom property (--#{$var}: value)
                    if self.is_css_custom_property() {
                        self.parse_declaration()
                    } else {
                        self.parse_style_rule()
                    }
                }
                Token::BlockComment(_) => {
                    let pos = self.current_pos();
                    if let Token::BlockComment(text) = self.advance() {
                        Ok(Stmt::Comment(text.clone()))
                    } else {
                        Err(SassError::parse("expected block comment", pos))
                    }
                }
                Token::LineComment(_) => {
                    self.advance();
                    continue;
                }
                _ => self.parse_stmt(),
            };
            stmts.push(stmt?);
        }

        Ok(stmts)
    }

    /// Parse an at-rule (delegates to at-rule parser)
    pub fn parse_at_rule(&mut self) -> Result<Stmt, SassError> {
        let pos = self.current_pos();
        let name = match self.advance() {
            Token::AtRule(n) => n.clone(),
            _ => return Err(SassError::parse("expected @-rule", pos)),
        };

        match name.as_str() {
            "if" => self.parse_if(),
            "for" => self.parse_for(),
            "each" => self.parse_each(),
            "while" => self.parse_while(),
            "mixin" => self.parse_mixin_def(),
            "include" => self.parse_include(),
            "function" => self.parse_function_def(),
            "return" => self.parse_return(),
            "error" => self.parse_error_stmt(),
            "warn" => self.parse_warn_stmt(),
            "debug" => self.parse_debug_stmt(),
            "at-root" => self.parse_at_root(),
            "media" => self.parse_media(),
            "supports" => self.parse_supports(),
            "use" => self.parse_use(),
            "forward" => self.parse_forward(),
            "import" => self.parse_import(),
            "extend" => self.parse_extend(),
            "content" => self.parse_content(),
            _ => self.parse_css_at_rule(&name),
        }
    }
}
