//! Selector parsing for SCSS.

use super::ast::*;
use super::core::Parser;

impl<'tok> Parser<'tok> {
    /// Parse a selector (handles pseudo-classes with parens like :is(), :not()).
    pub(super) fn parse_selector(&mut self) -> Option<Selector> {
        use crate::lexer::TokenKind;
        let mut parts = Vec::new();
        while !self.check(&TokenKind::LBrace) && !self.at_eof() {
            if let Some(part) = self.parse_selector_part() {
                parts.push(part);
            } else {
                break;
            }
        }
        if parts.is_empty() {
            None
        } else if parts.len() == 1 {
            parts.into_iter().next()
        } else {
            Some(Selector::Compound(parts))
        }
    }

    /// Parse a single selector part.
    fn parse_selector_part(&mut self) -> Option<Selector> {
        use crate::lexer::TokenKind;
        match self.peek_kind() {
            Some(TokenKind::Ident(s)) => {
                let name = s.clone();
                self.advance();
                Some(Selector::Type(name))
            }
            Some(TokenKind::Hash) => {
                self.advance();
                match self.parse_selector_part() {
                    Some(Selector::Type(name)) => Some(Selector::Id(name)),
                    _ => Some(Selector::Id(String::new())),
                }
            }
            Some(TokenKind::Dot) => {
                self.advance();
                // Handle class name: .ident, .#{...}, .ident-#{...}-suffix, etc.
                let mut name = String::new();
                while !self.check(&crate::lexer::TokenKind::LBrace)
                    && !self.check(&crate::lexer::TokenKind::RBrace)
                    && !self.check(&crate::lexer::TokenKind::Semicolon)
                    && !self.check(&crate::lexer::TokenKind::Whitespace)
                    && !self.check(&crate::lexer::TokenKind::Comma)
                    && !self.check(&crate::lexer::TokenKind::LParen)
                    && !self.check(&crate::lexer::TokenKind::RParen)
                    && !self.at_eof()
                {
                    match self.peek_kind() {
                        Some(crate::lexer::TokenKind::Interpolation) => {
                            self.advance();
                            let expr = self.parse_expr();
                            self.consume(&crate::lexer::TokenKind::RBrace);
                            // Preserve variable name for transformer lookup.
                            if let Some(Expr::Variable(var_name)) = &expr {
                                name.push_str("${");
                                name.push_str(var_name);
                                name.push('}');
                            } else {
                                name.push_str("#{...}");
                            }
                        }
                        Some(crate::lexer::TokenKind::Ident(ref s)) => {
                            name.push_str(s);
                            self.advance();
                        }
                        Some(crate::lexer::TokenKind::Minus) => {
                            name.push('-');
                            self.advance();
                        }
                        Some(crate::lexer::TokenKind::Hash) => {
                            name.push('#');
                            self.advance();
                        }
                        _ => break,
                    }
                }
                if name.is_empty() { None } else { Some(Selector::Class(name)) }
            }
            Some(TokenKind::Percent) => {
                // Placeholder selector: %name
                self.advance();
                let name = self.expect_ident_name()?;
                Some(Selector::Literal(format!("%{name}")))
            }
            Some(TokenKind::Ampersand) => {
                self.advance();
                // Handle &-xxx (BEM modifier like &-left)
                if self.check(&TokenKind::Minus) {
                    let mut suffix = String::new();
                    suffix.push('-');
                    self.advance();
                    // Collect ident after - (e.g., "-left", "-right")
                    if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                        suffix.push_str(&s);
                        self.advance();
                    }
                    // Also handle --var or rest
                    while let Some(TokenKind::Minus) = self.peek_kind() {
                        suffix.push('-');
                        self.advance();
                    }
                    if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                        suffix.push_str(&s);
                        self.advance();
                    }
                    return Some(Selector::ParentRef(Box::new(Selector::Literal(suffix))));
                }
                // Handle &xxx (direct concatenation like &header)
                if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                    let name = s.clone();
                    self.advance();
                    return Some(Selector::ParentRef(Box::new(Selector::Type(name))));
                }
                // Check for compound like &.foo, & > .foo, etc.
                if self.check(&TokenKind::Dot) || self.check(&TokenKind::Colon) || self.check(&TokenKind::LBracket) {
                    let inner = self.parse_selector_part().unwrap_or(Selector::Literal(String::new()));
                    return Some(Selector::ParentRef(Box::new(inner)));
                }
                if self.check(&TokenKind::Whitespace) {
                    self.advance();
                    // Could be "& *" or "& > .foo" - parse the next part
                    if self.check(&TokenKind::Star) {
                        self.advance();
                        return Some(Selector::ParentRef(Box::new(Selector::Universal)));
                    }
                    if self.check(&TokenKind::Greater) {
                        self.advance();
                        let right = self.parse_selector_part()?;
                        return Some(Selector::ParentRef(Box::new(right)));
                    }
                    // Default: & followed by descendant selectors
                    let inner = self.parse_selector_part().unwrap_or(Selector::Literal(String::new()));
                    return Some(Selector::ParentRef(Box::new(inner)));
                }
                Some(Selector::ParentRef(Box::new(Selector::Literal(String::new()))))
            }
            Some(TokenKind::Colon) => {
                self.advance();
                // Handle :: pseudo-element
                if self.check(&TokenKind::Colon) {
                    self.advance();
                }
                // 伪类名可能为 Interpolation / Ident / 关键字 Not / vendor-prefixed (-moz-... 等)
                let mut name = if matches!(self.peek_kind(), Some(TokenKind::Interpolation)) {
                    self.advance();
                    let _ = self.parse_expr();
                    self.consume(&crate::lexer::TokenKind::RBrace);
                    "#{...}".to_string()
                } else if let Some(n) = self.peek_ident() {
                    self.advance();
                    n
                } else if matches!(self.peek_kind(), Some(TokenKind::Not)) {
                    self.advance();
                    "not".to_string()
                } else {
                    String::new()
                };
                // Vendor-prefixed 伪类/伪元素: :-moz-focusring, :-webkit-slider-thumb 等
                while let Some(TokenKind::Minus) = self.peek_kind() {
                    let saved = self.pos;
                    self.advance();
                    if let Some(TokenKind::Ident(seg)) = self.peek_kind() {
                        name.push('-');
                        name.push_str(&seg);
                        self.advance();
                    } else {
                        self.pos = saved;
                        break;
                    }
                }
                // Handle pseudo-class with arguments: :is(...), :not(...), :nth-child(...), etc.
                if self.check(&TokenKind::LParen) {
                    self.advance(); // consume (
                    let mut depth = 1i32;
                    while !self.at_eof() && depth > 0 {
                        match self.peek_kind() {
                            Some(TokenKind::LParen) => { depth += 1; self.advance(); }
                            Some(TokenKind::RParen) => { depth -= 1; self.advance(); }
                            _ => { self.advance(); }
                        }
                    }
                }
                Some(Selector::Pseudo(name))
            }
            Some(TokenKind::Interpolation) => {
                // Interpolation in selector: #{...} or #{...}-foo
                let mut name = String::new();
                self.advance();
                let expr = self.parse_expr();
                self.consume(&crate::lexer::TokenKind::RBrace);
                // Preserve variable name for transformer lookup.
                if let Some(Expr::Variable(var_name)) = &expr {
                    name.push_str("${");
                    name.push_str(var_name);
                    name.push('}');
                } else {
                    name.push_str("#{...}");
                }
                // Consume trailing -ident parts (e.g., #{$x}-foo)
                while self.check(&crate::lexer::TokenKind::Minus) {
                    let saved = self.pos;
                    self.advance();
                    if matches!(self.peek_kind(), Some(crate::lexer::TokenKind::Ident(_))) {
                        name.push('-');
                        if let Some(TokenKind::Ident(ref s)) = self.peek_kind() {
                            name.push_str(s.as_str());
                            self.advance();
                        }
                    } else {
                        self.pos = saved;
                        break;
                    }
                }
                Some(Selector::Interpolation(name))
            }
            Some(TokenKind::LBracket) => {
                // Attribute selector: [attr], [attr=value], [attr^=value], etc.
                self.advance();
                self.parse_attribute_selector()
            }
            Some(TokenKind::Greater) => {
                // Child combinator: >
                self.advance();
                let right = self.parse_selector_part()?;
                Some(Selector::Child(Box::new(Selector::Literal(">".to_string())), Box::new(right)))
            }
            Some(TokenKind::Plus) => {
                // Adjacent sibling: +
                self.advance();
                let right = self.parse_selector_part()?;
                Some(Selector::Adjacent(Box::new(Selector::Literal("+".to_string())), Box::new(right)))
            }
            Some(TokenKind::Tilde) => {
                // General sibling: ~
                self.advance();
                let right = self.parse_selector_part()?;
                Some(Selector::Sibling(Box::new(Selector::Literal("~".to_string())), Box::new(right)))
            }
            Some(TokenKind::Star) => {
                // Universal selector: *
                self.advance();
                Some(Selector::Universal)
            }
            Some(TokenKind::Whitespace) => {
                self.advance();
                self.parse_selector_part()
            }
            _ => None,
        }
    }

    /// Parse attribute selector: [name], [name op value].
    fn parse_attribute_selector(&mut self) -> Option<Selector> {
        use crate::lexer::TokenKind;
        // Read attribute name (may contain multiple parts)
        let mut attr = String::new();
        while !self.check(&TokenKind::RBracket) && !self.check(&TokenKind::LBrace) && !self.at_eof() {
            match self.peek_kind() {
                Some(TokenKind::Ident(ref s)) => {
                    attr.push_str(s.as_str());
                    self.advance();
                }
                Some(TokenKind::Caret) => {
                    attr.push('^');
                    self.advance();
                }
                Some(TokenKind::Dollar) => {
                    attr.push('$');
                    self.advance();
                }
                Some(TokenKind::Tilde) => {
                    attr.push('~');
                    self.advance();
                }
                Some(TokenKind::Pipe) => {
                    attr.push('|');
                    self.advance();
                }
                Some(TokenKind::Star) => {
                    attr.push('*');
                    self.advance();
                }
                Some(TokenKind::Eq) => {
                    attr.push('=');
                    self.advance();
                }
                Some(TokenKind::String(ref s)) => {
                    attr.push_str(s.as_str());
                    self.advance();
                }
                Some(TokenKind::RBracket) => break,
                _ => {
                    // Skip unknown tokens inside attribute selector
                    self.advance();
                }
            }
        }
        self.consume(&TokenKind::RBracket);
        Some(Selector::Attribute(attr))
    }
}
