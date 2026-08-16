//! Expression parsing for SCSS.
//!
//! Implements operator-precedence parsing for Sass expressions.

use super::ast::*;
use super::core::Parser;

impl<'tok> Parser<'tok> {
    /// Parse an expression.
    /// Handles comma-separated value lists (e.g., `color: red, blue` → List([red, blue])).
    pub(super) fn parse_expr(&mut self) -> Option<Expr> {
        self.parse_list()
    }

    /// Parse comma-separated list expression.
    /// SCSS 中声明值常包含逗号分隔的列表: `box-shadow: 1px 2px red, 3px 4px blue`
    fn parse_list(&mut self) -> Option<Expr> {
        let mut items = Vec::new();
        items.push(self.parse_logical()?);
        while self.check(&crate::lexer::TokenKind::Comma) {
            self.advance(); // consume comma
            items.push(self.parse_logical()?);
        }
        if items.len() == 1 {
            Some(items.into_iter().next().unwrap())
        } else {
            Some(Expr::List(items))
        }
    }

    /// Parse logical expressions (and, or) - lowest precedence.
    pub(crate) fn parse_logical(&mut self) -> Option<Expr> {
        let mut left = self.parse_space_list()?;
        while let Some(op) = self.peek_logical_op() {
            self.advance();
            let right = self.parse_space_list()?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
        Some(left)
    }

    /// Parse space-separated value list (e.g., `1px sans-serif`, `1px 2px 3px`).
    /// In CSS/SCSS, values can be space-separated without explicit commas.
    fn parse_space_list(&mut self) -> Option<Expr> {
        let mut items = Vec::new();
        items.push(self.parse_additive()?);
        loop {
            // Skip whitespace and check if next is a value token (not comma, semicolon, brace, eof)
            use crate::lexer::TokenKind;
            match self.peek_kind() {
                Some(TokenKind::Comma) | Some(TokenKind::Semicolon)
                | Some(TokenKind::RBrace) | Some(TokenKind::RParen)
                | Some(TokenKind::Eof) | None => break,
                Some(TokenKind::LBrace) | Some(TokenKind::LBracket) => break,
                // Don't consume logical ops or add/mul ops
                Some(TokenKind::And) | Some(TokenKind::Or) => break,
                Some(TokenKind::Plus) | Some(TokenKind::Minus) => break,
                Some(TokenKind::Star) | Some(TokenKind::Slash) => break,
                // Percent is a valid standalone value in SCSS (e.g., `width: 50%`,
                // or `a {b: c %}` where % is a literal value).
                Some(TokenKind::Colon) => break,
                // `!important`/`!global`/`!default` flags — Not 在值之后出现是标志前缀
                Some(TokenKind::Not) => break,
                _ => {
                    // Try to parse another value item
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
        } else {
            Some(Expr::SpaceList(items))
        }
    }

    /// Check for logical operator (and/or).
    fn peek_logical_op(&mut self) -> Option<BinaryOp> {
        use crate::lexer::TokenKind;
        match self.peek_kind() {
            Some(TokenKind::And) => Some(BinaryOp::And),
            Some(TokenKind::Or) => Some(BinaryOp::Or),
            _ => None,
        }
    }

    /// Parse additive expressions (+, -).
    pub(super) fn parse_additive(&mut self) -> Option<Expr> {
        let mut left = self.parse_comparison()?;
        while let Some(op) = self.peek_add_op() {
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
        Some(left)
    }

    /// Parse comparison expressions (==, !=, >, <, >=, <=).
    fn parse_comparison(&mut self) -> Option<Expr> {
        let mut left = self.parse_multiplicative()?;
        while let Some(op) = self.peek_cmp_op() {
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
        Some(left)
    }

    /// Check for comparison operator.
    fn peek_cmp_op(&mut self) -> Option<BinaryOp> {
        use crate::lexer::TokenKind;
        match self.peek_kind() {
            Some(TokenKind::Eq) => Some(BinaryOp::Eq),
            Some(TokenKind::NotEq) => Some(BinaryOp::NotEq),
            Some(TokenKind::Greater) => Some(BinaryOp::Greater),
            Some(TokenKind::Less) => Some(BinaryOp::Less),
            Some(TokenKind::GreaterEq) => Some(BinaryOp::GreaterEq),
            Some(TokenKind::LessEq) => Some(BinaryOp::LessEq),
            _ => None,
        }
    }

    /// Parse multiplicative expressions (*, /, %).
    /// If right side fails to parse after the operator, rollback and treat
    /// the operator as not part of this expression (e.g., `c %` where % is a literal).
    fn parse_multiplicative(&mut self) -> Option<Expr> {
        let mut left = self.parse_unary()?;
        while let Some(op) = self.peek_mul_op() {
            let saved = self.pos;
            self.advance();
            match self.parse_unary() {
                Some(right) => {
                    left = Expr::Binary(op, Box::new(left), Box::new(right));
                }
                None => {
                    // Right side not parseable — rollback and stop
                    self.pos = saved;
                    break;
                }
            }
        }
        Some(left)
    }

    /// Parse unary expressions (-, not).
    /// 支持 vendor-prefixed 名称: -moz-foo, --custom-prop, --#{$prefix}-color, -#{$name}
    fn parse_unary(&mut self) -> Option<Expr> {
        use crate::lexer::TokenKind;
        if self.check(&TokenKind::Minus) {
            // Lookahead: Minus 后紧跟 Ident 或 Interpolation → vendor-prefixed name
            // 这是 SCSS/CSS 的关键模式: --#{$prefix}-color
            let rollback = self.pos;
            let mut minus_count = 0;
            while self.check(&TokenKind::Minus) {
                self.advance();
                minus_count += 1;
            }
            let next_is_name = matches!(
                self.peek_kind(),
                Some(TokenKind::Ident(_)) | Some(TokenKind::Interpolation)
            );
            if minus_count >= 1 && next_is_name {
                // Vendor-prefixed identifier: 重建名称，可能包含 #{} 插值
                let mut name = String::new();
                for _ in 0..minus_count {
                    name.push('-');
                }
                // 消费 Interpolation (#{...})
                while self.check(&TokenKind::Interpolation) {
                    self.advance();
                    let _ = self.parse_expr();
                    self.consume(&TokenKind::RBrace);
                    name.push_str("#{...}");
                }
                // 消费 Ident
                if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                    name.push_str(s.as_str());
                    self.advance();
                }
                // 消费中间可能的 -, Interpolation, Ident 序列 (如 --#{$p}-color)
                loop {
                    if self.check(&TokenKind::Minus) {
                        // 检查 - 后跟 Ident 或 Interpolation
                        let saved = self.pos;
                        self.advance();
                        if matches!(
                            self.peek_kind(),
                            Some(TokenKind::Ident(_)) | Some(TokenKind::Interpolation)
                        ) {
                            name.push('-');
                            // Interpolation
                            while self.check(&TokenKind::Interpolation) {
                                self.advance();
                                let _ = self.parse_expr();
                                self.consume(&TokenKind::RBrace);
                                name.push_str("#{...}");
                            }
                            // Ident
                            if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                                name.push_str(s.as_str());
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
                // Handle namespacing: -foo.bar()
                while self.check(&TokenKind::Dot) {
                    self.advance();
                    if let Some(TokenKind::Ident(seg)) = self.peek_kind() {
                        name.push('.');
                        name.push_str(seg.as_str());
                        self.advance();
                    } else {
                        break;
                    }
                }
                // Optional function call
                if self.check(&TokenKind::LParen) {
                    self.advance();
                    let args = self.parse_arg_list_inner()?;
                    if !self.consume(&TokenKind::RParen) {
                        self.error("expected ')'");
                    }
                    return Some(Expr::Call(name, args));
                }
                return Some(Expr::Variable(name));
            }
            // 不是 vendor-prefix，回退到一元取反
            self.pos = rollback;
            if self.check(&TokenKind::Minus) {
                self.advance();
                let expr = self.parse_primary()?;
                return Some(Expr::Unary(UnaryOp::Neg, Box::new(expr)));
            }
        }
        if self.check(&TokenKind::Not) {
            self.advance();
            let expr = self.parse_primary()?;
            Some(Expr::Unary(UnaryOp::Not, Box::new(expr)))
        } else {
            self.parse_primary()
        }
    }

    /// Parse primary expressions (literals, variables, calls, parens).
    fn parse_primary(&mut self) -> Option<Expr> {
        use crate::lexer::TokenKind;
        match self.peek_kind() {
            Some(TokenKind::Variable(name)) => {
                let v = name.clone();
                self.advance();
                Some(Expr::Variable(v))
            }
            Some(TokenKind::Number(val, unit)) => {
                let n = (val, unit);
                self.advance();
                Some(Expr::Number(n.0, n.1))
            }
            Some(TokenKind::String(s)) => {
                let text = s;
                self.advance();
                Some(Expr::String(text))
            }
            Some(TokenKind::Color(hex)) => {
                let h = hex;
                self.advance();
                Some(Expr::Color(h))
            }
            Some(TokenKind::Url(url)) => {
                let u = url.clone();
                self.advance();
                Some(Expr::Url(u))
            }
            Some(TokenKind::Ampersand) => {
                // Parent selector reference & (often in interpolation #{&})
                self.advance();
                Some(Expr::Variable("&".to_string()))
            }
            Some(TokenKind::Interpolation) => {
                // Handle interpolation #{...} in expressions
                self.advance();
                let _expr = self.parse_expr();
                self.consume(&TokenKind::RBrace);
                Some(Expr::String("#{...}".to_string()))
            }
            Some(TokenKind::LParen) => {
                self.advance();
                // Check if this is a map: (key: value, ...)
                if super::lookahead::is_map_syntax(self.tokens(), self.pos) {
                    return self.parse_map_literal();
                }
                // Check if it's a list: (a, b, c)
                if super::lookahead::is_list_syntax(self.tokens(), self.pos) {
                    return self.parse_list_literal();
                }
                // 空括号 () → 空 map（SCSS 中常用于 $theme-colors: () !default 这类写法）
                if self.check(&TokenKind::RParen) {
                    self.advance();
                    return Some(Expr::Map(Vec::new()));
                }
                let expr = self.parse_expr();
                // 尝试消费到匹配的 RParen（容错：var(--#{$x}-y) 这类非标准表达式）
                if !self.consume(&TokenKind::RParen) {
                    let mut depth = 1;
                    while depth > 0 && !self.at_eof() {
                        match self.peek_kind() {
                            Some(TokenKind::LParen) => depth += 1,
                            Some(TokenKind::RParen) => depth -= 1,
                            _ => {}
                        }
                        if depth > 0 {
                            self.advance();
                        } else {
                            self.advance(); // consume the final RParen
                        }
                    }
                    return Some(Expr::String("(...)".to_string()));
                }
                match expr {
                    Some(e) => Some(Expr::Parens(Box::new(e))),
                    None => Some(Expr::String("(...)".to_string())),
                }
            }
            Some(TokenKind::Ident(name)) => {
                let mut n = name.clone();
                self.advance();
                // Handle namespaced function calls: namespace.function(...)
                while self.check(&TokenKind::Dot) {
                    self.advance(); // consume .
                    if let Some(TokenKind::Ident(seg)) = self.peek_kind() {
                        n.push('.');
                        n.push_str(seg.as_str());
                        self.advance();
                    } else {
                        break;
                    }
                }
                if self.check(&TokenKind::LParen) {
                    self.advance();
                    let args = self.parse_arg_list_inner()?;
                    if !self.consume(&TokenKind::RParen) {
                        self.error("expected ')'");
                    }
                    Some(Expr::Call(n, args))
                } else if n == "true" {
                    Some(Expr::Boolean(true))
                } else if n == "false" {
                    Some(Expr::Boolean(false))
                } else if n == "null" {
                    Some(Expr::Null)
                } else {
                    // Bare identifier — treat as Identifier (CSS value or string),
                    // NOT as a variable reference.
                    Some(Expr::Identifier(n))
                }
            }
            Some(TokenKind::Percent) => {
                // Standalone % as a value (e.g., `a {b: c %}` or `a {b: %}`)
                self.advance();
                Some(Expr::Identifier("%".to_string()))
            }
            _ => None,
        }
    }

    /// Helper: parse comma-separated argument list.
    /// Supports named arguments (`$arg: value`) and rest args (`...`).
    /// 注意: 使用 parse_logical 避免双重逗号处理 (parse_expr 现在处理逗号列表).
    pub(super) fn parse_arg_list_inner(&mut self) -> Option<Vec<Expr>> {
        let mut args = Vec::new();
        use crate::lexer::TokenKind;
        while !self.check(&TokenKind::RParen) && !self.at_eof() {
            // Check for rest args: ...
            if self.check(&TokenKind::DotDotDot) {
                self.advance();
                args.push(Expr::Variable("...".to_string()));
                break;
            }
            // Check for named argument: $var: expr
            if let Some(var_name) = self.peek_var() {
                let saved = self.pos;
                self.advance(); // consume variable
                if self.check(&TokenKind::Colon) {
                    self.advance(); // consume colon
                    let value = self.parse_logical()?;
                    args.push(Expr::NamedArg(var_name, Box::new(value)));
                } else {
                    // Not a named arg, rollback and parse as expression
                    self.pos = saved;
                    let expr = self.parse_logical()?;
                    // 支持 $args... 展开语法：变量后紧跟 ...
                    if self.check(&TokenKind::DotDotDot) {
                        self.advance();
                        args.push(Expr::Spread(Box::new(expr)));
                    } else {
                        args.push(expr);
                    }
                }
            // All other expressions (including parenthesized maps, lists, grouped exprs)
            } else {
                let expr = self.parse_logical()?;
                // 也支持表达式后 ...（如 func()... 展开）
                if self.check(&TokenKind::DotDotDot) {
                    self.advance();
                    args.push(Expr::Spread(Box::new(expr)));
                } else {
                    args.push(expr);
                }
            }
            if self.check(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        Some(args)
    }

    /// Helper: parse public arg list.
    pub(super) fn parse_arg_list(&mut self) -> Option<Vec<Expr>> {
        use crate::lexer::TokenKind;
        if !self.check(&TokenKind::LParen) {
            return Some(Vec::new());
        }
        self.advance();
        let args = self.parse_arg_list_inner()?;
        if !self.consume(&TokenKind::RParen) {
            self.error("expected ')'");
        }
        Some(args)
    }

    /// Helper: parse param list with optional defaults and rest args (`...`).
    pub(super) fn parse_param_list(&mut self) -> Option<Vec<Param>> {
        use crate::lexer::TokenKind;
        if !self.check(&TokenKind::LParen) {
            return Some(Vec::new());
        }
        self.advance();
        let mut params = Vec::new();
        while !self.check(&TokenKind::RParen) && !self.at_eof() {
            let name = match self.peek_var() {
                Some(v) => v,
                None => break,
            };
            self.advance();
            let default = if self.check(&TokenKind::Colon) {
                self.advance();
                self.parse_logical()
            } else {
                None
            };
            // Handle rest args: $args...
            let has_rest = if self.check(&TokenKind::DotDotDot) {
                self.advance();
                true
            } else {
                false
            };
            params.push(Param { name, default });
            if has_rest {
                break;  // Rest args must be last
            }
            if self.check(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        if !self.consume(&TokenKind::RParen) {
            self.error("expected ')'");
        }
        Some(params)
    }

    /// Helper: parse config map.
    pub(super) fn parse_config_map(&mut self) -> Option<Vec<(String, Expr)>> {
        use crate::lexer::TokenKind;
        if !self.check(&TokenKind::LParen) {
            self.error("expected '(' for config");
            return None;
        }
        self.advance();
        let mut map = Vec::new();
        while !self.check(&TokenKind::RParen) && !self.at_eof() {
            let name = self.peek_var()?;
            self.advance();
            if !self.consume(&TokenKind::Colon) {
                self.error("expected ':'");
                break;
            }
            let value = self.parse_logical()?;
            map.push((name, value));
            if self.check(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        if !self.consume(&TokenKind::RParen) {
            self.error("expected ')'");
        }
        Some(map)
    }
}
