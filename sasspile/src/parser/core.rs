//! Recursive descent parser for SCSS.
//!
//! Sub-modules split by concern:
//! - at_rules.rs  → @-rule parsing
//! - expr.rs     → expression parsing
//! - selector.rs → selector parsing

use tracing::instrument;

use crate::lexer::{Token, TokenKind};
use crate::diagnostics::{Diagnostic, Diagnostics};
use crate::source::SourceSpan;

/// SCSS parser.
pub struct Parser<'tok> {
    tokens: &'tok [Token],
    pub(crate) pos: usize,
    diagnostics: Diagnostics,
}

impl<'tok> Parser<'tok> {
    /// Returns a reference to the token slice for lookahead operations.
    pub(crate) fn tokens(&self) -> &'tok [Token] {
        self.tokens
    }
}

impl<'tok> Parser<'tok> {
    /// Create a new parser.
    pub fn new(tokens: &'tok [Token]) -> Self {
        Self {
            tokens,
            pos: 0,
            diagnostics: Diagnostics::new(),
        }
    }

    /// Parse the full stylesheet.
    #[instrument(skip(self), fields(nodes_parsed = 0, errors = 0))]
    pub fn parse(mut self) -> (super::ast::Stylesheet, Diagnostics) {
        let mut nodes = Vec::new();
        while !self.at_eof() {
            match self.parse_node() {
                Some(node) => {
                    nodes.push(node);
                    tracing::Span::current().record("nodes_parsed", nodes.len());
                }
                None => {
                    self.advance();
                    tracing::Span::current().record("errors", self.diagnostics.errors().len());
                }
            }
        }
        (super::ast::Stylesheet { nodes }, self.diagnostics)
    }

    // ──────────────────── Core parsing ────────────────────────────

    /// Parse a single top-level or nested node.
    #[instrument(skip(self), fields(decision = tracing::field::Empty, token = tracing::field::Empty, pos = tracing::field::Empty))]
    fn parse_node(&mut self) -> Option<super::ast::Node> {
        let span = tracing::Span::current();
        let pos = self.current_pos();
        span.record("pos", pos);
        if let Some(tok) = self.peek_kind() {
            span.record("token", format!("{tok:?}"));
        }
        match self.peek_kind() {
            Some(TokenKind::AtKeyword(kw)) => {
                span.record("decision", format!("at-rule:{kw}"));
                self.parse_at_rule().map(super::ast::Node::AtRule)
            }
            Some(TokenKind::Whitespace) => {
                self.advance();
                None
            }
            Some(_) => {
                // 顶层函数调用（map.merge(...), calc() 等）— 当作 noop 消费
                if super::lookahead::is_top_level_expr_call(self.tokens, self.pos) {
                    span.record("decision", "top-level-expr-call");
                    self.parse_expr();
                    return None;
                }
                let is_decl = super::lookahead::looks_like_declaration(self.tokens, self.pos);
                span.record("decision", if is_decl { "declaration" } else { "rule" });
                if is_decl {
                    self.parse_declaration().map(super::ast::Node::Declaration)
                } else {
                    self.parse_rule().map(super::ast::Node::Rule)
                }
            }
            None => None,
        }
    }

    /// Parse a style rule: selector { body }.
    /// Supports comma-separated selectors: a, b, c { ... }
    fn parse_rule(&mut self) -> Option<super::ast::Rule> {
        let mut selectors = Vec::new();
        selectors.push(self.parse_selector()?);

        // Support comma-separated selectors
        loop {
            // Skip whitespace
            self.skip_whitespace();
            if self.check(&TokenKind::Comma) {
                self.advance(); // consume ,
                // Parse next selector
                if let Some(sel) = self.parse_selector() {
                    selectors.push(sel);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let selector = if selectors.len() == 1 {
            selectors.into_iter().next().unwrap()
        } else {
            super::ast::Selector::Compound(selectors)
        };

        if !self.consume(&TokenKind::LBrace) {
            self.error("expected '{'");
            return None;
        }

        let nodes = self.parse_body()?;

        Some(super::ast::Rule { selector, nodes })
    }

    /// Parse a declaration: name: value; (supports $var: value with !global/!default).
    /// Also supports CSS custom properties: --color: red; --#{$name}: red;
    #[instrument(skip(self))]
    fn parse_declaration(&mut self) -> Option<super::ast::Declaration> {
        // Track whether this declaration came from a `$var` token (variable declaration).
        let mut is_variable = false;
        let name = match self.peek_kind() {
            Some(TokenKind::Ident(s)) => {
                let mut n = s.clone();
                self.advance();
                // Consume additional parts: -ident, -#{...}, etc. (e.g., border-#{$p}-color)
                loop {
                    if self.check(&TokenKind::Interpolation) {
                        self.advance();
                        let _ = self.parse_expr();
                        self.consume(&TokenKind::RBrace);
                        n.push_str("#{...}");
                    } else if self.check(&TokenKind::Minus) {
                        let saved = self.pos;
                        self.advance();
                        if matches!(self.peek_kind(), Some(TokenKind::Ident(_)) | Some(TokenKind::Interpolation)) {
                            n.push('-');
                            if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                                n.push_str(s.as_str());
                                self.advance();
                            }
                        } else {
                            self.pos = saved;
                            break;
                        }
                    } else if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                        // Adjacent ident (no separator, rare but possible)
                        n.push_str(s.as_str());
                        self.advance();
                    } else {
                        break;
                    }
                }
                n
            }
            Some(TokenKind::Variable(v)) => {
                is_variable = true;
                let n = v.clone();
                self.advance();
                n
            }
            // Interpolation as property name: #{$var}: value
            Some(TokenKind::Interpolation) => {
                let mut n = String::new();
                self.advance();
                let _ = self.parse_expr();
                self.consume(&TokenKind::RBrace);
                n.push_str("#{...}");
                // Continue consuming any trailing parts (e.g., #{...}-suffix)
                loop {
                    if self.check(&TokenKind::Interpolation) {
                        self.advance();
                        let _ = self.parse_expr();
                        self.consume(&TokenKind::RBrace);
                        n.push_str("#{...}");
                    } else if self.check(&TokenKind::Minus) {
                        let saved = self.pos;
                        self.advance();
                        if matches!(self.peek_kind(), Some(TokenKind::Ident(_)) | Some(TokenKind::Interpolation)) {
                            n.push('-');
                            if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                                n.push_str(s.as_str());
                                self.advance();
                            }
                        } else {
                            self.pos = saved;
                            break;
                        }
                    } else {
                        break;
                    }
                }
                n
            }
            // 自定义属性: --foo 或 --#{$name}
            Some(TokenKind::Minus) => {
                let mut n = String::new();
                while self.check(&TokenKind::Minus) {
                    self.advance();
                    n.push('-');
                }
                // 循环消费: (Interpolation | Ident | Minus+Ident/Interpolation)*
                // 必须持续迭代直到下一个 token 不属于属性名
                loop {
                    let start_len = n.len();
                    // Interpolation
                    while self.check(&TokenKind::Interpolation) {
                        self.advance();
                        let _ = self.parse_expr();
                        self.consume(&TokenKind::RBrace);
                        n.push_str("#{...}");
                    }
                    // Ident
                    if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                        n.push_str(s.as_str());
                        self.advance();
                    }
                    // 连字符 - 后跟 Ident/Interpolation
                    if self.check(&TokenKind::Minus) {
                        let saved = self.pos;
                        self.advance();
                        if matches!(self.peek_kind(), Some(TokenKind::Ident(_)) | Some(TokenKind::Interpolation)) {
                            n.push('-');
                        } else {
                            self.pos = saved;
                        }
                    }
                    // 如果没有新字符被消费，退出循环
                    if n.len() == start_len {
                        break;
                    }
                }
                n
            }
            _ => {
                self.error("expected property name");
                return None;
            }
        };

        if !self.consume(&TokenKind::Colon) {
            self.error("expected ':' after property name");
            return None;
        }

        let value = self.parse_expr()?;

        // Check for !important, !global, !default
        let mut important = false;
        while self.check(&TokenKind::Not) {
            self.advance();
            if self.check_ident("important") {
                self.advance();
                important = true;
            } else if self.check_ident("global") || self.check_ident("default") {
                self.advance();
                // Just consume these flags
            } else {
                break;
            }
        }

        if self.check(&TokenKind::Semicolon) {
            self.advance();
        }

        let span = SourceSpan::new(0, 0);
        Some(super::ast::Declaration {
            name,
            value,
            important,
            span,
            is_variable,
        })
    }

    /// Parse body until closing brace.
    #[instrument(skip(self), fields(kind = "body", depth = tracing::field::Empty, pos = tracing::field::Empty))]
    pub(super) fn parse_body(&mut self) -> Option<Vec<super::ast::Node>> {
        let span = tracing::Span::current();
        span.record("pos", self.current_pos());
        let mut nodes = Vec::new();
        let mut node_count = 0;
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            match self.parse_node() {
                Some(node) => {
                    node_count += 1;
                    nodes.push(node);
                }
                None => {
                    // Skip token and try to recover
                    if !self.at_eof() {
                        self.advance();
                    }
                }
            }
        }
        if !self.consume(&TokenKind::RBrace) {
            span.record("nodes_in_body", node_count);
            self.error("expected '}' at body close");
        }
        span.record("nodes_in_body", node_count);
        Some(nodes)
    }

    // ──────────────────── Common helpers ───────────────────────────

    /// Parse remaining tokens as text (for @media/@supports).
    pub(super) fn parse_value_text(&mut self) -> Option<String> {
        let mut text = String::new();
        while !self.check(&TokenKind::LBrace) && !self.at_eof() {
            match self.peek_kind() {
                Some(TokenKind::Whitespace) => text.push(' '),
                Some(TokenKind::Ident(ref s)) => text.push_str(s),
                Some(TokenKind::Variable(ref s)) => {
                    text.push('$');
                    text.push_str(s);
                }
                Some(TokenKind::String(ref s)) => {
                    text.push('"');
                    text.push_str(s);
                    text.push('"');
                }
                Some(TokenKind::Number(ref val, ref unit)) => {
                    if val.fract().abs() < f64::EPSILON {
                        text.push_str(&(*val as i64).to_string());
                    } else {
                        text.push_str(&val.to_string());
                    }
                    if let Some(u) = unit {
                        text.push_str(u);
                    }
                }
                Some(TokenKind::Colon) => text.push(':'),
                Some(TokenKind::LParen) => text.push('('),
                Some(TokenKind::RParen) => text.push(')'),
                Some(TokenKind::Dot) => text.push('.'),
                Some(TokenKind::Comma) => text.push(','),
                Some(TokenKind::Slash) => text.push('/'),
                Some(TokenKind::Hash) => text.push('#'),
                Some(TokenKind::Minus) => text.push('-'),
                _ => {}
            }
            self.advance();
        }
        Some(text.trim().to_string())
    }

    /// Expect a string literal.
    pub(super) fn expect_string(&mut self) -> Option<String> {
        match self.peek_kind() {
            Some(TokenKind::String(s)) => {
                let text = s.clone();
                self.advance();
                Some(text)
            }
            _ => {
                self.error("expected string literal");
                None
            }
        }
    }

    /// Expect an identifier.
    pub(super) fn expect_ident_name(&mut self) -> Option<String> {
        match self.peek_kind() {
            Some(TokenKind::Ident(s)) => {
                let text = s.clone();
                self.advance();
                Some(text)
            }
            _ => {
                self.error("expected identifier");
                None
            }
        }
    }

    /// Expect an identifier that may have a vendor prefix (e.g., `-moz-element`, `-a-url`).
    /// Also handles namespaced names like `foo.bar` (mixin include) or `string.length`.
    /// Lexer tokenizes `-name` as `Minus` + `Ident`, so we recombine them here.
    pub(super) fn expect_vendor_ident_name(&mut self) -> Option<String> {
        let mut name = String::new();
        // Consume optional leading Minus tokens (vendor prefix)
        while self.check(&TokenKind::Minus) {
            self.advance();
            name.push('-');
        }
        match self.peek_kind() {
            Some(TokenKind::Ident(s)) => {
                name.push_str(s.as_str());
                self.advance();
                // Handle dot-separated namespacing: foo.bar, string.length
                while self.check(&TokenKind::Dot) {
                    self.advance();
                    match self.peek_kind() {
                        Some(TokenKind::Ident(seg)) => {
                            name.push('.');
                            name.push_str(seg.as_str());
                            self.advance();
                        }
                        _ => break,
                    }
                }
                Some(name)
            }
            _ => {
                self.error("expected identifier");
                None
            }
        }
    }

    /// Consume semicolon if present.
    pub(super) fn consume_semicolon(&mut self) {
        if self.check(&TokenKind::Semicolon) {
            self.advance();
        }
    }

    // ──────────────────── Token navigation ─────────────────────────

    /// Skip whitespace in-place.
    pub(crate) fn skip_whitespace(&mut self) {
        while self.pos < self.tokens.len()
            && matches!(self.tokens[self.pos].kind, TokenKind::Whitespace)
        {
            self.pos += 1;
        }
    }

    /// Peek at current token kind (skips whitespace first).
    pub(crate) fn peek_kind(&mut self) -> Option<TokenKind> {
        self.skip_whitespace();
        self.tokens.get(self.pos).map(|t| t.kind.clone())
    }

    /// Peek at current identifier.
    pub(crate) fn peek_ident(&mut self) -> Option<String> {
        self.skip_whitespace();
        match self.tokens.get(self.pos) {
            Some(Token { kind: TokenKind::Ident(s), .. }) => Some(s.clone()),
            _ => None,
        }
    }

    /// Get current variable name.
    pub(crate) fn peek_var(&mut self) -> Option<String> {
        self.skip_whitespace();
        match self.tokens.get(self.pos) {
            Some(Token { kind: TokenKind::Variable(name), .. }) => Some(name.clone()),
            _ => None,
        }
    }

    /// Check if current token matches.
    pub(crate) fn check(&mut self, kind: &TokenKind) -> bool {
        match self.peek_kind() {
            Some(ref k) => std::mem::discriminant(k) == std::mem::discriminant(kind),
            None => false,
        }
    }

    /// Check if current ident matches.
    pub(crate) fn check_ident(&mut self, name: &str) -> bool {
        matches!(self.peek_kind(), Some(TokenKind::Ident(s)) if s == name)
    }

    /// Check if at end of tokens.
    pub(crate) fn at_eof(&mut self) -> bool {
        matches!(self.peek_kind(), None | Some(TokenKind::Eof))
    }

    /// Get current byte position (for tracing spans).
    pub(crate) fn current_pos(&self) -> u32 {
        self.tokens.get(self.pos).map(|t| t.span.start).unwrap_or(0)
    }

    /// Advance to next token.
    pub(crate) fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    /// Check and consume a token.
    pub(crate) fn consume(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Check additive operator.
    pub(crate) fn peek_add_op(&mut self) -> Option<super::ast::BinaryOp> {
        match self.peek_kind() {
            Some(TokenKind::Plus) => Some(super::ast::BinaryOp::Add),
            Some(TokenKind::Minus) => Some(super::ast::BinaryOp::Sub),
            _ => None,
        }
    }

    /// Check multiplicative operator.
    pub(crate) fn peek_mul_op(&mut self) -> Option<super::ast::BinaryOp> {
        match self.peek_kind() {
            Some(TokenKind::Star) => Some(super::ast::BinaryOp::Mul),
            Some(TokenKind::Slash) => Some(super::ast::BinaryOp::Div),
            Some(TokenKind::Percent) => Some(super::ast::BinaryOp::Mod),
            _ => None,
        }
    }

    /// Record an error with tracing + internal diagnostic.
    pub(crate) fn error(&mut self, message: &str) {
        let pos = self.current_pos();
        let tok_str = self.peek_kind().map(|t| format!("{t:?}"));
        tracing::warn!(
            parser_error = true,
            error_message = message,
            pos,
            token = tok_str.as_deref().unwrap_or("?"),
            "parse error"
        );
        let diag = Diagnostic::error("P001", message);
        self.diagnostics.push(diag);
    }

    /// Parse a map literal: (key: value, key2: value2).
    /// 注意: 使用 parse_logical 避免双重逗号处理 (parse_expr 现在处理逗号列表).
    pub(crate) fn parse_map_literal(&mut self) -> Option<super::ast::Expr> {
        use super::ast::Expr;
        let mut map = Vec::new();
        while !self.check(&TokenKind::RParen) && !self.at_eof() {
            let key = self.parse_logical()?;
            if !self.consume(&TokenKind::Colon) {
                // Not a map, just return as list element
                map.push((key, Expr::Null));
                break;
            }
            let value = self.parse_logical().unwrap_or(Expr::Null);
            map.push((key, value));
            if self.check(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
            // Skip whitespace
            while self.check(&TokenKind::Whitespace) {
                self.advance();
            }
        }
        self.consume(&TokenKind::RParen);
        Some(Expr::Map(map))
    }

    /// Parse a list literal: (expr1, expr2, ...).
    /// 注意: 使用 parse_logical 避免双重逗号处理 (parse_expr 现在处理逗号列表).
    pub(crate) fn parse_list_literal(&mut self) -> Option<super::ast::Expr> {
        use super::ast::Expr;
        let mut items = Vec::new();
        while !self.check(&TokenKind::RParen) && !self.at_eof() {
            let item = self.parse_logical()?;
            items.push(item);
            if self.check(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
            while self.check(&TokenKind::Whitespace) {
                self.advance();
            }
        }
        self.consume(&TokenKind::RParen);
        Some(Expr::List(items))
    }
}
