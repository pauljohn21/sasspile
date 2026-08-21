//! 节点解析 + 参数解析。
//!
//! 包含 parse_node/parse_rule/parse_decl/parse_variable 等节点级解析，
//! 以及 parse_params/parse_args/parse_config 等参数解析和辅助方法。

use super::Parser;
use super::ast::*;
use crate::error::{Result, SassError};
use crate::lex::token::Token;
use crate::__tracing::{trace, warn};

impl<'tok> Parser<'tok> {
    // —— 节点解析 ——
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self), fields(pos = self.pos)))]
    pub(crate) fn parse_node(&mut self) -> Result<Node> {
        self.skip_ws();
        let peek_str = self
            .peek()
            .map(|t| t.to_string())
            .unwrap_or_else(|| "EOF".into());
        trace!(peek = %peek_str, "parse_node");
        match self.peek() {
            Some(Token::AtRule(name)) => self.parse_at_rule(name.clone()),
            Some(Token::Dollar(_)) => self.parse_variable(),
            Some(Token::Comment(t, silent)) => {
                let node = Node::Comment(t.clone(), *silent);
                self.advance();
                Ok(node)
            }
            Some(Token::Semicolon) => {
                // 顶层孤立的 ; — 跳过（如 `downstream {...};` 中的尾部分号）
                self.advance();
                self.skip_ws();
                if self.peek().is_none() || matches!(self.peek(), Some(Token::Eof | Token::RBrace)) {
                    // 文件末尾或 body 末尾的孤立 ; — 返回空注释节点
                    return Ok(Node::Comment(String::new(), true));
                }
                self.parse_node()
            }
            Some(Token::Whitespace) => {
                self.advance();
                self.parse_node()
            }
            _ => {
                // 检测命名空间变量赋值：Ident . Dollar → namespace.$var: value
                if self.is_namespace_var() {
                    return self.parse_namespace_var();
                }
                self.parse_rule_or_decl()
            }
        }
    }

    /// Lookahead: { 先出现 → 规则, ; 或 } 先出现 → 声明。
    pub(crate) fn is_rule(&self) -> bool {
        let mut i = self.pos;
        while i < self.tokens.len() {
            match &self.tokens[i] {
                Token::LBrace => return true,
                Token::Semicolon | Token::RBrace => return false,
                Token::LParen => {
                    // 跳过括号内容
                    let mut depth = 1;
                    i += 1;
                    while i < self.tokens.len() && depth > 0 {
                        match &self.tokens[i] {
                            Token::LParen => depth += 1,
                            Token::RParen => depth -= 1,
                            _ => {}
                        }
                        i += 1;
                    }
                    continue;
                }
                Token::Whitespace => {
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        }
        true
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self), fields(pos = self.pos)))]
    pub(crate) fn parse_rule_or_decl(&mut self) -> Result<Node> {
        let is_r = self.is_rule();
        trace!(is_rule = is_r, "parse_rule_or_decl");
        if is_r {
            self.parse_rule()
        } else {
            self.parse_decl()
        }
    }

    pub(crate) fn parse_rule(&mut self) -> Result<Node> {
        let selector = self.parse_selector()?;
        self.skip_ws();
        self.expect(&Token::LBrace)?;
        let body = self.parse_body()?;
        Ok(Node::Rule { selector, body })
    }

    pub(crate) fn parse_selector(&mut self) -> Result<String> {
        let mut s = String::new();
        let mut bracket_depth = 0i32;
        while let Some(t) = self.peek() {
            crate::__tracing::trace!(token = ?t, accumulated = %s, "parse_selector token");
            match t {
                Token::LBrace => break,
                Token::LBracket => {
                    bracket_depth += 1;
                    s.push('[');
                    self.advance();
                }
                Token::RBracket => {
                    bracket_depth -= 1;
                    s.push(']');
                    self.advance();
                }
                Token::Whitespace => {
                    if bracket_depth > 0 {
                        // 向前跳过所有连续空白，找到下一个非空白 token
                        let mut look = 1;
                        while matches!(self.peek_n(look), Some(Token::Whitespace)) {
                            look += 1;
                        }
                        let next_non_ws = self.peek_n(look);
                        if s.ends_with('[') || s.ends_with('=') {
                            // [ 或 = 后的空白跳过
                            self.advance();
                        } else if matches!(next_non_ws, Some(Token::RBracket | Token::Assign | Token::Tilde | Token::Pipe | Token::Caret | Token::Star)) {
                            // ] 前或属性操作符（= ~= |= ^= *=）前的空白跳过
                            self.advance();
                        } else {
                            // 避免连续空格
                            if !s.ends_with(' ') {
                                s.push(' ');
                            }
                            self.advance();
                        }
                    } else {
                        s.push(' ');
                        self.advance();
                    }
                }
                Token::Comment(_, _) => {
                    self.advance();
                } // 跳过注释
                _ => {
                    s.push_str(&t.to_string());
                    self.advance();
                }
            }
        }
        Ok(s.trim().to_string())
    }

    pub(crate) fn parse_decl(&mut self) -> Result<Node> {
        let property = self.parse_property()?;
        self.skip_ws();
        self.expect(&Token::Colon)?;
        self.skip_ws();
        // 声明值中使用斜杠分隔符语义——`1/2` 保留为 `1/2` 而非计算除法
        let value = self.parse_decl_value()?;
        let important = self.check_important()?;
        self.skip_ws();
        if self.peek() == Some(&Token::Semicolon) {
            self.advance();
        }
        Ok(Node::Decl {
            property,
            value,
            important,
        })
    }

    pub(crate) fn parse_property(&mut self) -> Result<String> {
        let mut s = String::new();
        while let Some(t) = self.peek() {
            match t {
                Token::Colon | Token::Whitespace | Token::RBrace | Token::Semicolon => break,
                _ => {
                    s.push_str(&t.to_string());
                    self.advance();
                }
            }
        }
        Ok(s)
    }

    pub(crate) fn check_important(&mut self) -> Result<bool> {
        self.skip_ws();
        if self.peek() == Some(&Token::Bang) {
            self.advance();
            self.skip_ws();
            if let Some(Token::Ident(s)) = self.peek()
                && s == "important" {
                    self.advance();
                    return Ok(true);
                }
        }
        Ok(false)
    }

    pub(crate) fn parse_variable(&mut self) -> Result<Node> {
        let name = match self.peek() {
            Some(Token::Dollar(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => {
                return Err(SassError::Parse {
                    expected: "$var".into(),
                    found: "other".into(),
                });
            }
        };
        self.skip_ws();
        self.expect(&Token::Colon)?;
        self.skip_ws();
        let value = self.parse_value()?;
        let flags = self.parse_var_flags()?;
        self.skip_ws();
        if self.peek() == Some(&Token::Semicolon) {
            self.advance();
        }
        Ok(Node::Variable { name, value, flags })
    }

    /// 检测是否为命名空间变量赋值（Ident . Dollar 模式）。
    fn is_namespace_var(&self) -> bool {
        let mut i = self.pos;
        // 跳过 Whitespace
        while i < self.tokens.len() && matches!(self.tokens[i], Token::Whitespace) {
            i += 1;
        }
        // Ident
        if i >= self.tokens.len() || !matches!(self.tokens[i], Token::Ident(_)) {
            return false;
        }
        i += 1;
        while i < self.tokens.len() && matches!(self.tokens[i], Token::Whitespace) {
            i += 1;
        }
        // .
        if i >= self.tokens.len() || !matches!(self.tokens[i], Token::Dot) {
            return false;
        }
        i += 1;
        while i < self.tokens.len() && matches!(self.tokens[i], Token::Whitespace) {
            i += 1;
        }
        // Dollar
        i < self.tokens.len() && matches!(self.tokens[i], Token::Dollar(_))
    }

    /// 解析命名空间变量赋值——`namespace.$var: value;`。
    fn parse_namespace_var(&mut self) -> Result<Node> {
        let ns = match self.peek() {
            Some(Token::Ident(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => unreachable!(),
        };
        // 消费 .
        self.skip_ws();
        self.expect(&Token::Dot)?;
        self.skip_ws();
        // 消费 $var
        let var_name = match self.peek() {
            Some(Token::Dollar(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => unreachable!(),
        };
        let name = format!("{ns}.{var_name}");
        self.skip_ws();
        self.expect(&Token::Colon)?;
        self.skip_ws();
        let value = self.parse_value()?;
        let flags = self.parse_var_flags()?;
        self.skip_ws();
        if self.peek() == Some(&Token::Semicolon) {
            self.advance();
        }
        Ok(Node::Variable { name, value, flags })
    }

    pub(crate) fn parse_var_flags(&mut self) -> Result<VarFlags> {
        let mut flags = VarFlags::default();
        self.skip_ws();
        while self.peek() == Some(&Token::Bang) {
            self.advance();
            self.skip_ws();
            if let Some(Token::Ident(s)) = self.peek() {
                match s.as_str() {
                    "default" => flags.default = true,
                    "global" => flags.global = true,
                    _ => {}
                }
                self.advance();
            }
            self.skip_ws();
        }
        Ok(flags)
    }

    pub(crate) fn parse_body(&mut self) -> Result<Vec<Node>> {
        let mut nodes = Vec::new();
        let prev = self.in_body;
        self.in_body = true;
        loop {
            self.skip_ws();
            match self.peek() {
                Some(Token::RBrace) | None | Some(Token::Eof) => break,
                _ => nodes.push(self.parse_node()?),
            }
        }
        self.skip_ws();
        if self.peek() == Some(&Token::RBrace) {
            self.advance();
        }
        self.in_body = prev;
        Ok(nodes)
    }

    // —— 参数解析 ——
    pub(crate) fn parse_params(&mut self) -> Result<Vec<Param>> {
        self.expect(&Token::LParen)?;
        let mut params = Vec::new();
        loop {
            self.skip_ws();
            if self.peek() == Some(&Token::RParen) {
                break;
            }
            let name = match self.peek() {
                Some(Token::Dollar(n)) => {
                    let n = n.clone();
                    self.advance();
                    n
                }
                _ => {
                    return Err(SassError::Parse {
                        expected: "$param".into(),
                        found: "other".into(),
                    });
                }
            };
            self.skip_ws();
            let default = if self.peek() == Some(&Token::Colon) {
                self.advance();
                self.skip_ws();
                Some(self.parse_expr(0)?)
            } else {
                None
            };
            let rest = if self.peek() == Some(&Token::DotDotDot) {
                self.advance();
                true
            } else {
                false
            };
            params.push(Param {
                name,
                default,
                rest,
            });
            self.skip_ws();
            if self.peek() == Some(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.skip_ws();
        if self.peek() == Some(&Token::RParen) {
            self.advance();
        }
        Ok(params)
    }

    pub(crate) fn parse_args(&mut self) -> Result<Vec<Arg>> {
        self.expect(&Token::LParen)?;
        let mut args = Vec::new();
        loop {
            self.skip_ws();
            if self.peek() == Some(&Token::RParen) {
                break;
            }
            // 检查关键字参数 $name: value 或 name: value（如 if(else: c)）
            // 也支持 CSS 函数语法 name=value（如 alpha(opacity=0)）
            let is_kwarg = match self.peek() {
                Some(Token::Dollar(_)) => {
                    let next = self.tokens.get(self.pos + 1);
                    let after_ws = if matches!(next, Some(Token::Whitespace)) {
                        self.tokens.get(self.pos + 2)
                    } else {
                        next
                    };
                    matches!(after_ws, Some(Token::Colon))
                }
                Some(Token::Ident(s))
                    if !matches!(s.as_str(), "true" | "false" | "null" | "and" | "or" | "not") =>
                {
                    let next = self.tokens.get(self.pos + 1);
                    let after_ws = if matches!(next, Some(Token::Whitespace)) {
                        self.tokens.get(self.pos + 2)
                    } else {
                        next
                    };
                    matches!(after_ws, Some(Token::Colon) | Some(Token::Assign))
                }
                _ => false,
            };
            let (name, value) = if is_kwarg {
                let n = match self.peek() {
                    Some(Token::Dollar(n)) => {
                        let n = n.clone();
                        self.advance();
                        n
                    }
                    Some(Token::Ident(n)) => {
                        let n = n.clone();
                        self.advance();
                        n
                    }
                    _ => unreachable!(),
                };
                self.skip_ws();
                // 消费 : 或 = （CSS 函数语法如 alpha(opacity=0)）
                if matches!(self.peek(), Some(Token::Colon) | Some(Token::Assign)) {
                    self.advance();
                }
                self.skip_ws();
                (Some(n), self.parse_expr(0)?)
            } else {
                // 解析位置参数表达式
                let expr = self.parse_expr(0)?;
                // 检查 if() 冒号语法：expression : value
                // 例如 if(css(): c) 或 if(sass(true): c; else: d)
                self.skip_ws();
                if self.peek() == Some(&Token::Colon) {
                    // 消费 :
                    self.advance();
                    self.skip_ws();
                    let val = self.parse_expr(0)?;
                    let spread = if self.peek() == Some(&Token::DotDotDot) {
                        self.advance();
                        true
                    } else {
                        false
                    };
                    args.push(Arg {
                        name: None,
                        value: val,
                        spread,
                        condition: Some(expr),
                    });
                    // 检查 ; condition: value 或 ; else: other 语法（支持多条件）
                    self.skip_ws();
                    while self.peek() == Some(&Token::Semicolon) {
                        self.advance();
                        self.skip_ws();
                        // 期望 else : value 或 condition : value
                        if let Some(Token::Ident(s)) = self.peek()
                            && s == "else" {
                                self.advance();
                                self.skip_ws();
                                if self.peek() == Some(&Token::Colon) {
                                    self.advance();
                                    self.skip_ws();
                                }
                                let else_val = self.parse_expr(0)?;
                                args.push(Arg {
                                    name: Some("else".to_string()),
                                    value: else_val,
                                    spread: false,
                                    condition: None,
                                });
                                break; // else 是最后一个
                            }
                        // 解析 condition : value
                        let cond2 = self.parse_expr(0)?;
                        self.skip_ws();
                        if self.peek() == Some(&Token::Colon) {
                            self.advance();
                            self.skip_ws();
                            let val2 = self.parse_expr(0)?;
                            let spread2 = if self.peek() == Some(&Token::DotDotDot) {
                                self.advance();
                                true
                            } else {
                                false
                            };
                            args.push(Arg {
                                name: None,
                                value: val2,
                                spread: spread2,
                                condition: Some(cond2),
                            });
                        }
                    }
                    // 跳过后续处理
                    self.skip_ws();
                    if self.peek() == Some(&Token::Comma) {
                        self.advance();
                        continue;
                    } else {
                        break;
                    }
                }
                (None, expr)
            };
            let spread = if self.peek() == Some(&Token::DotDotDot) {
                self.advance();
                true
            } else {
                false
            };
            args.push(Arg {
                name,
                value,
                spread,
                condition: None,
            });
            self.skip_ws();
            if self.peek() == Some(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.skip_ws();
        if self.peek() == Some(&Token::RParen) {
            self.advance();
        }
        Ok(args)
    }

    pub(crate) fn parse_config(&mut self) -> Result<Vec<ConfigVar>> {
        let mut config = Vec::new();
        self.skip_ws();
        if self.peek() == Some(&Token::RParen) {
            return Err(SassError::Eval("expected \"$\".".into()));
        }
        loop {
            self.skip_ws();
            if self.peek() == Some(&Token::RParen) {
                break;
            }
            let name = match self.peek() {
                Some(Token::Dollar(n)) => {
                    let n = n.clone();
                    if n.is_empty() {
                        return Err(SassError::Eval("Expected identifier.".into()));
                    }
                    self.advance();
                    n
                }
                _ => {
                    return Err(SassError::Eval("expected \"$\".".into()));
                }
            };
            self.skip_ws();
            self.expect(&Token::Colon)?;
            self.skip_ws();
            let value = self.parse_expr(0)?;
            self.skip_ws();
            let mut is_default = false;
            while self.peek() == Some(&Token::Bang) {
                self.advance();
                self.skip_ws();
                if let Some(Token::Ident(s)) = self.peek() {
                    if s == "default" {
                        is_default = true;
                    }
                    self.advance();
                    self.skip_ws();
                }
            }
            config.push(ConfigVar { name, value, is_default });
            if self.peek() == Some(&Token::Comma) {
                self.advance();
                self.skip_ws();
                let is_ok = matches!(self.peek(), Some(Token::Dollar(_)));
                if !is_ok && !matches!(self.peek(), Some(Token::RParen)) {
                    return Err(SassError::Eval("expected \")\".".into()));
                }
            } else {
                break;
            }
        }
        if self.peek() == Some(&Token::RParen) {
            self.advance();
        }
        Ok(config)
    }

    pub(crate) fn parse_member_list(&mut self) -> Result<Vec<String>> {
        let mut members = Vec::new();
        self.skip_ws();
        loop {
            match self.peek() {
                Some(Token::Semicolon) | Some(Token::LBrace) | None => break,
                Some(Token::Dollar(n)) => {
                    if n.is_empty() {
                        return Err(SassError::Eval(
                            "Expected variable, mixin, or function name".into(),
                        ));
                    }
                    members.push(format!("${n}"));
                    self.advance();
                }
                Some(Token::Ident(n)) => {
                    members.push(n.clone());
                    self.advance();
                }
                _ => {
                    return Err(SassError::Eval(
                        "Expected variable, mixin, or function name".into(),
                    ));
                }
            }
            self.skip_ws();
            if self.peek() == Some(&Token::Comma) {
                self.advance();
                self.skip_ws();
                let is_ok = matches!(self.peek(), Some(Token::Dollar(_)));
                if !is_ok && !matches!(self.peek(), Some(Token::Ident(_))) {
                    return Err(SassError::Eval(
                        "Expected variable, mixin, or function name".into(),
                    ));
                }
            } else {
                break;
            }
        }
        Ok(members)
    }

    // —— 辅助方法 ——
    pub(crate) fn parse_ident_name(&mut self) -> Result<String> {
        self.skip_ws();
        match self.peek() {
            Some(Token::Ident(s)) => {
                let s = s.clone();
                self.advance();
                Ok(s)
            }
            _ => Err(SassError::Parse {
                expected: "identifier".into(),
                found: "other".into(),
            }),
        }
    }

    pub(crate) fn parse_string_value(&mut self) -> Result<String> {
        self.skip_ws();
        match self.peek() {
            Some(Token::String(s, _)) => {
                let s = s.clone();
                self.advance();
                Ok(s)
            }
            Some(Token::Ident(s)) => {
                let s = s.clone();
                self.advance();
                Ok(s)
            }
            _ => Err(SassError::Parse {
                expected: "string".into(),
                found: "other".into(),
            }),
        }
    }

    pub(crate) fn peek_keyword(&self, kw: &str) -> bool {
        matches!(self.peek(), Some(Token::Ident(s)) if s == kw)
    }

    pub(crate) fn expect_keyword(&mut self, kw: &str) -> Result<()> {
        if self.peek_keyword(kw) {
            self.advance();
            Ok(())
        } else {
            Err(SassError::Parse {
                expected: kw.into(),
                found: "other".into(),
            })
        }
    }
}
