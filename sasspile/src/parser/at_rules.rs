//! @-rule parsing for the SCSS parser.
//!
//! Contains parsing logic for all CSS/Sass at-rules.

use tracing::instrument;

use super::ast::*;
use super::core::Parser;

impl<'tok> Parser<'tok> {
    /// Parse an at-rule.
    #[instrument(skip(self))]
    pub(super) fn parse_at_rule(&mut self) -> Option<AtRule> {
        let keyword = match self.peek_kind() {
            Some(TokenKind::AtKeyword(kw)) => kw.clone(),
            _ => return None,
        };
        self.advance();
        // At-rule 关键字后可以跟任意空白（含换行），如 `@if\n  true {}` / `@for \n  $i from 1 through 10 {}`
        self.skip_whitespace();

        match keyword.as_str() {
            "use" => self.parse_use_rule(),
            "import" => self.parse_import_rule(),
            "mixin" => self.parse_mixin_def(),
            "include" => self.parse_include_rule(),
            "function" => self.parse_function_def(),
            "return" => self.parse_return_rule(),
            "if" => self.parse_if_rule(),
            "for" => self.parse_for_rule(),
            "each" => self.parse_each_rule(),
            "while" => self.parse_while_rule(),
            "extend" => self.parse_extend_rule(),
            "media" => self.parse_media_rule(),
            "supports" => self.parse_supports_rule(),
            "content" => Some(AtRule::Content),
            "debug" => self.parse_debug_rule(),
            "warn" => self.parse_warn_rule(),
            "error" => self.parse_error_rule(),
            "forward" => self.parse_forward_rule(),
            "at-root" => self.parse_at_root_rule(),
            "keyframes" => self.parse_keyframes_rule(),
            // Unknown at-rules: just consume until semicolon or block
            _ => self.parse_unknown_at_rule(),
        }
    }

    fn parse_use_rule(&mut self) -> Option<AtRule> {
        let url = self.expect_string()?;
        // `as` clause
        let namespace = if self.check_ident("as") {
            self.advance();
            if self.check(&crate::lexer::TokenKind::Star) {
                self.advance();
                Some("*".to_string())
            } else {
                self.skip_whitespace();
                let name = self.peek_ident().unwrap_or_default();
                if !name.is_empty() { self.advance(); }
                Some(name)
            }
        } else {
            None
        };
        // `with (...)` config map
        let config = if self.check_ident("with") {
            self.advance();
            self.parse_config_map()?
        } else {
            Vec::new()
        };
        // `show` / `hide` member visibility lists
        if self.check_ident("show") || self.check_ident("hide") {
            self.advance();
            loop {
                if matches!(self.peek_kind(), Some(TokenKind::Variable(_))) {
                    self.advance();
                } else {
                    break;
                }
                self.skip_whitespace();
                if self.check(&TokenKind::Comma) {
                    self.advance();
                    self.skip_whitespace();
                } else {
                    break;
                }
            }
        }
        self.consume_semicolon();
        let _ = config;
        Some(AtRule::Use(UseRule { url, namespace, config: Vec::new() }))
    }

    fn parse_import_rule(&mut self) -> Option<AtRule> {
        let urls = vec![self.expect_string()?];
        // Consume optional import conditions/media queries:
        // @import "url" supports(...), "url2" media, ... ;
        // Strategy: consume tokens until ; (with paren depth tracking).
        // Stop at LBrace (rule body) to avoid consuming into outer blocks.
        let mut depth = 0i32;
        while !self.at_eof() {
            match self.peek_kind() {
                Some(crate::lexer::TokenKind::Semicolon) if depth == 0 => break,
                Some(crate::lexer::TokenKind::LBrace) | Some(crate::lexer::TokenKind::RBrace) if depth == 0 => break,
                Some(crate::lexer::TokenKind::LParen) => { depth += 1; self.advance(); }
                Some(crate::lexer::TokenKind::RParen) => { depth -= 1; self.advance(); }
                _ => { self.advance(); }
            }
        }
        self.consume_semicolon();
        Some(AtRule::Import(ImportRule { urls }))
    }

    fn parse_forward_rule(&mut self) -> Option<AtRule> {
        let url = self.expect_string()?;
        // @forward supports: as prefix-*, with (...), show/hide $member
        // `as` clause
        if self.check_ident("as") {
            self.advance();
            // `as prefix-*` — consume prefix ident, optional `-`, and `*`
            if matches!(self.peek_kind(), Some(TokenKind::Ident(_))) {
                self.advance();
            }
            if self.check(&TokenKind::Minus) {
                self.advance();
            }
            if self.check(&TokenKind::Star) {
                self.advance();
            }
        }
        // `with (...)` config map
        let config = if self.check_ident("with") {
            self.advance();
            self.parse_config_map()?
        } else {
            Vec::new()
        };
        // `show` / `hide` member visibility lists
        if self.check_ident("show") || self.check_ident("hide") {
            self.advance();
            // Consume comma-separated variable list: $a, $b, ...
            loop {
                if matches!(self.peek_kind(), Some(TokenKind::Variable(_))) {
                    self.advance();
                } else {
                    break;
                }
                // Skip optional whitespace before comma
                self.skip_whitespace();
                if self.check(&TokenKind::Comma) {
                    self.advance();
                    self.skip_whitespace();
                } else {
                    break;
                }
            }
        }
        self.consume_semicolon();
        let _ = config;
        Some(AtRule::Forward(ForwardRule { url }))
    }

    #[instrument(skip(self), fields(mixin_name = tracing::field::Empty, params_count = tracing::field::Empty, pos = tracing::field::Empty))]
    fn parse_mixin_def(&mut self) -> Option<AtRule> {
        let span = tracing::Span::current();
        span.record("pos", self.current_pos());
        let name = self.expect_vendor_ident_name()?;
        span.record("mixin_name", &name);
        let params = self.parse_param_list()?;
        span.record("params_count", params.len());
        if !self.consume(&TokenKind::LBrace) {
            self.error("expected '{' to start mixin body");
            return None;
        }
        let body = self.parse_body()?;
        Some(AtRule::Mixin(MixinDef { name, params, body }))
    }

    #[instrument(skip(self), fields(include_name = tracing::field::Empty, pos = tracing::field::Empty, has_body = tracing::field::Empty))]
    fn parse_include_rule(&mut self) -> Option<AtRule> {
        let span = tracing::Span::current();
        span.record("pos", self.current_pos());
        let name = self.expect_vendor_ident_name()?;
        span.record("include_name", &name);
        let args = self.parse_arg_list()?;
        // Handle `using (args)` for @content passing
        if self.check_ident("using") {
            self.advance();
            self.parse_arg_list()?;
        }
        // @include can be followed by ; or { body }
        if self.check(&crate::lexer::TokenKind::LBrace) {
            span.record("has_body", true);
            let body = self.parse_body()?;
            Some(AtRule::Include(IncludeRule { name, args, body }))
        } else {
            span.record("has_body", false);
            self.consume_semicolon();
            Some(AtRule::Include(IncludeRule { name, args, body: Vec::new() }))
        }
    }

    fn parse_function_def(&mut self) -> Option<AtRule> {
        let name = self.expect_vendor_ident_name()?;
        let params = self.parse_param_list()?;
        if !self.consume(&TokenKind::LBrace) {
            self.error("expected '{'");
            return None;
        }
        let body = self.parse_body()?;
        Some(AtRule::Function(FunctionDef { name, params, body }))
    }

    fn parse_return_rule(&mut self) -> Option<AtRule> {
        let expr = self.parse_expr()?;
        self.consume_semicolon();
        Some(AtRule::Return(expr))
    }

    fn parse_if_rule(&mut self) -> Option<AtRule> {
        let condition = self.parse_expr()?;
        if !self.consume(&TokenKind::LBrace) {
            self.error("expected '{'");
            return None;
        }
        let body = self.parse_body()?;

        // Handle optional @else if ... { } or @else { }
        // @else is lexed as AtKeyword("else")
        let else_body = if matches!(self.peek_kind(), Some(TokenKind::AtKeyword(ref s)) if s == "else") {
            self.advance(); // consume @else
            // Check for @else if
            if self.check_ident("if") {
                self.advance(); // consume "if"
                // Recursive: parse chained @if/@else
                let nested_if = self.parse_if_rule()?;
                Some(vec![super::ast::Node::AtRule(nested_if)])
            } else {
                // @else { ... }
                if !self.consume(&TokenKind::LBrace) {
                    self.error("expected '{'");
                    return None;
                }
                let else_body = self.parse_body()?;
                Some(else_body)
            }
        } else {
            None
        };

        Some(AtRule::If(IfStmt { condition, body, else_body }))
    }

    fn parse_for_rule(&mut self) -> Option<AtRule> {
        let var = match self.peek_var() {
            Some(v) => v,
            None => {
                self.error("expected variable after @for");
                return None;
            }
        };
        self.advance();
        if !self.check_ident("from") {
            self.error("expected 'from'");
            return None;
        }
        self.advance();
        // Parse start expr, stopping at 'to'/'through' keywords.
        let start = self.parse_for_range_value()?;
        let inclusive = if self.check_ident("through") {
            self.advance();
            true
        } else {
            if !self.check_ident("to") {
                self.error("expected 'to' or 'through'");
                return None;
            }
            self.advance();
            false
        };
        let end = self.parse_for_range_value()?;
        if !self.consume(&TokenKind::LBrace) {
            self.error("expected '{'");
            return None;
        }
        let body = self.parse_body()?;
        Some(AtRule::For(ForStmt { var, start, end, inclusive, body }))
    }

    /// Parse a single value in @for range (stops at 'to'/'through' keywords).
    fn parse_for_range_value(&mut self) -> Option<Expr> {
        use crate::lexer::TokenKind;
        let mut items = Vec::new();
        while !self.at_eof() {
            match self.peek_kind() {
                // Stop at keywords or block-openers
                Some(TokenKind::Ident(ref s)) if s == "to" || s == "through" => break,
                Some(TokenKind::LBrace) | Some(TokenKind::Semicolon) | Some(TokenKind::Eof) => break,
                Some(TokenKind::Whitespace) => { self.advance(); }
                _ => {
                    if let Some(item) = self.parse_additive() {
                        items.push(item);
                    } else {
                        break;
                    }
                }
            }
        }
        if items.len() == 1 {
            Some(items.into_iter().next().unwrap())
        } else if items.is_empty() {
            Some(Expr::Null)
        } else {
            Some(Expr::SpaceList(items))
        }
    }

    fn parse_each_rule(&mut self) -> Option<AtRule> {
        // @each $var1, $var2 in $list { ... }
        // Parse one or more comma-separated variables
        let mut vars = Vec::new();
        vars.push(self.peek_var()?);
        self.advance();
        // Support multiple variables: @each $k, $v in $map
        self.skip_whitespace();
        while self.check(&TokenKind::Comma) {
            self.advance(); // consume ,
            self.skip_whitespace();
            if let Some(v) = self.peek_var() {
                self.advance();
                vars.push(v);
                self.skip_whitespace();
            } else {
                break;
            }
        }
        if !self.check_ident("in") {
            self.error("expected 'in'");
            return None;
        }
        self.advance();
        let list = self.parse_expr()?;
        if !self.consume(&TokenKind::LBrace) {
            self.error("expected '{'");
            return None;
        }
        let body = self.parse_body()?;
        Some(AtRule::Each(EachStmt { vars, list, body }))
    }

    fn parse_while_rule(&mut self) -> Option<AtRule> {
        let condition = self.parse_expr()?;
        if !self.consume(&TokenKind::LBrace) {
            self.error("expected '{'");
            return None;
        }
        let body = self.parse_body()?;
        Some(AtRule::While(WhileStmt { condition, body }))
    }

    fn parse_extend_rule(&mut self) -> Option<AtRule> {
        let selector = self.parse_selector()?;
        // Handle !optional flag after selector
        if self.check(&crate::lexer::TokenKind::Not) {
            self.advance();
            // Skip the optional keyword
            if self.check_ident("optional") {
                self.advance();
            }
        }
        self.consume_semicolon();
        Some(AtRule::Extend(selector))
    }

    fn parse_media_rule(&mut self) -> Option<AtRule> {
        let query = self.parse_value_text()?;
        if !self.consume(&TokenKind::LBrace) {
            self.error("expected '{'");
            return None;
        }
        let body = self.parse_body()?;
        Some(AtRule::Media(MediaRule { query, body }))
    }

    fn parse_supports_rule(&mut self) -> Option<AtRule> {
        let condition = self.parse_value_text()?;
        if !self.consume(&TokenKind::LBrace) {
            self.error("expected '{'");
            return None;
        }
        let body = self.parse_body()?;
        Some(AtRule::Supports(SupportsRule { condition, body }))
    }

    fn parse_debug_rule(&mut self) -> Option<AtRule> {
        let expr = self.parse_expr()?;
        self.consume_semicolon();
        Some(AtRule::Debug(expr))
    }

    fn parse_warn_rule(&mut self) -> Option<AtRule> {
        let expr = self.parse_expr()?;
        self.consume_semicolon();
        Some(AtRule::Warn(expr))
    }

    fn parse_error_rule(&mut self) -> Option<AtRule> {
        let expr = self.parse_expr()?;
        self.consume_semicolon();
        Some(AtRule::Error(expr))
    }

    /// Parse @at-root rule.
    fn parse_at_root_rule(&mut self) -> Option<AtRule> {
        // Consume optional selector or just the block
        if !self.consume(&TokenKind::LBrace) {
            // Consume value text then expect {
            let _ = self.parse_value_text();
            if !self.consume(&TokenKind::LBrace) {
                self.error("expected '{'");
                return None;
            }
        }
        let body = self.parse_body()?;
        Some(AtRule::AtRoot(body))
    }

    /// Parse @keyframes rule.
    fn parse_keyframes_rule(&mut self) -> Option<AtRule> {
        let name = self.parse_value_text().unwrap_or_default();
        if !self.consume(&TokenKind::LBrace) {
            self.error("expected '{'");
            return None;
        }
        let body = self.parse_body()?;
        Some(AtRule::Media(MediaRule {
            query: format!("@keyframes {name}"),
            body,
        }))
    }

    /// Parse unknown at-rule by consuming until semicolon or balanced block.
    /// Stops (without consuming) at RBrace when depth=0, letting the outer body handle it.
    fn parse_unknown_at_rule(&mut self) -> Option<AtRule> {
        let mut depth = 0u32;
        while !self.at_eof() {
            match self.peek_kind() {
                Some(TokenKind::Semicolon) if depth == 0 => {
                    self.consume_semicolon();
                    break;
                }
                Some(TokenKind::RBrace) if depth == 0 => {
                    // Don't consume — outer body needs this }
                    break;
                }
                Some(TokenKind::LBrace) => {
                    depth += 1;
                    self.advance();
                }
                Some(TokenKind::RBrace) if depth > 0 => {
                    depth -= 1;
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }
        // Return a generic representation
        Some(AtRule::Content)
    }
}

// Import TokenKind for the impl block
use crate::lexer::TokenKind;
