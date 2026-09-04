//! 节点解析 + 参数解析。
//!
//! 包含 `parse_node/parse_rule/parse_decl/parse_variable` 等节点级解析，
//! 以及 `parse_params/parse_args/parse_config` 等参数解析和辅助方法。

use super::Parser;
use super::ast::*;
use crate::__tracing::{trace, warn};
use crate::error::{Result, SassError};
use crate::lex::token::Token;

impl Parser<'_> {
    // —— 节点解析 ——
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self), fields(pos = self.pos)))]
    pub(crate) fn parse_node(&mut self) -> Result<Node> {
        self.skip_ws();
        let peek_str = self
            .peek()
            .map_or_else(|| "EOF".into(), std::string::ToString::to_string);
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
                match self.peek() {
                    None | Some(Token::Eof | Token::RBrace) => {
                        // 文件末尾或 body 末尾的孤立 ; — 返回空注释节点
                        return Ok(Node::Comment(String::new(), true));
                    }
                    _ => self.parse_node(),
                }
            }
            Some(Token::Whitespace) => {
                self.advance();
                self.parse_node()
            }
            _ => {
                // 检测命名空间变量赋值：Ident . Dollar → namespace.$var: value
                match self.is_namespace_var() {
                    true => return self.parse_namespace_var(),
                    false => {}
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
        match is_r {
            true => self.parse_rule(),
            false => self.parse_decl(),
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
                    match bracket_depth > 0 {
                        true => {
                        // 向前跳过所有连续空白，找到下一个非空白 token
                        let mut look = 1;
                        while matches!(self.peek_n(look), Some(Token::Whitespace)) {
                            look += 1;
                        }
                        let next_non_ws = self.peek_n(look);
                        let s_ends_bracket = s.ends_with('[');
                        let s_ends_eq = s.ends_with('=');
                        let s_has_eq = s.contains('=');
                        match (s_ends_bracket || s_ends_eq, next_non_ws, s_has_eq) {
                            (true, _, _) => {
                                // [ 或 = 后的空白跳过
                                self.advance();
                            }
                            (false, Some(Token::RBracket | Token::Assign | Token::Tilde | Token::Pipe | Token::Caret | Token::Star), _) => {
                                // ] 前或属性操作符（= ~= |= ^= *=）前的空白跳过
                                self.advance();
                            }
                            (false, _, true) => {
                                // 属性值后的空白 — 检查是否是合法 modifier（任意单字符标识符，前向兼容）
                                match next_non_ws {
                                    Some(Token::Ident(id))
                                        if id.len() == 1
                                            && id
                                                .chars()
                                                .next()
                                                .is_some_and(|c| c.is_ascii_alphabetic()) =>
                                    {
                                        let mod_id = id.clone();
                                        // 检查 modifier 后面是否是 ]
                                        let after_mod = self.peek_n(look + 1);
                                        match matches!(after_mod, Some(Token::RBracket)) {
                                            true => {
                                                // 跳过空白
                                                for _ in 0..look {
                                                    self.advance();
                                                }
                                                // 消费 modifier 字符
                                                match !s.ends_with(' ') {
                                                    true => { s.push(' '); }
                                                    false => {}
                                                }
                                                s.push_str(&mod_id);
                                                self.advance();
                                            }
                                            false => {
                                                return Err(SassError::Parse {
                                                    expected: "]".into(),
                                                    found: "modifier".into(),
                                                });
                                            }
                                        }
                                    }
                                    _ => {
                                        return Err(SassError::Parse {
                                            expected: "]".into(),
                                            found: "modifier".into(),
                                        });
                                    }
                                }
                            }
                            (false, _, false) => {
                                // 属性名后空白后不是操作符 → 非法
                                return Err(SassError::Parse {
                                    expected: "]".into(),
                                    found: "modifier".into(),
                                });
                            }
                        }
                    }
                        false => {
                            s.push(' ');
                            self.advance();
                        }
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
        self.skip_ws_and_comments();
        self.expect(&Token::Colon)?;
        self.skip_ws_and_comments();
        // 声明值中使用斜杠分隔符语义——`1/2` 保留为 `1/2` 而非计算除法
        let value = self.parse_decl_value()?;
        let important = self.check_important()?;
        self.skip_ws_and_comments();
        match self.peek() {
            Some(Token::Semicolon) => { self.advance(); }
            _ => {}
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
                Token::Comment(_, _) => {
                    self.advance();
                } // 跳过注释
                _ => {
                    s.push_str(&t.to_string());
                    self.advance();
                }
            }
        }
        Ok(s)
    }

    pub(crate) fn check_important(&mut self) -> Result<bool> {
        self.skip_ws_and_comments();
        match self.peek() {
            Some(Token::Bang) => {
                self.advance();
                self.skip_ws_and_comments();
                match self.peek() {
                    Some(Token::Ident(s)) if s == "important" => {
                        self.advance();
                        return Ok(true);
                    }
                    _ => {}
                }
            }
            _ => {}
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
        match self.peek() {
            Some(Token::Semicolon) => { self.advance(); }
            _ => {}
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
        match i >= self.tokens.len() || !matches!(self.tokens[i], Token::Ident(_)) {
            true => return false,
            false => {}
        }
        i += 1;
        while i < self.tokens.len() && matches!(self.tokens[i], Token::Whitespace) {
            i += 1;
        }
        // .
        match i >= self.tokens.len() || !matches!(self.tokens[i], Token::Dot) {
            true => return false,
            false => {}
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
        // 命名空间变量赋值不允许 !global
        match flags.global {
            true => {
                return Err(SassError::Eval(
                    "!global isn't allowed for variables in other modules.".into(),
                ));
            }
            false => {}
        }
        self.skip_ws();
        match self.peek() {
            Some(Token::Semicolon) => { self.advance(); }
            _ => {}
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
                Some(Token::RBrace | Token::Eof) | None => break,
                _ => nodes.push(self.parse_node()?),
            }
        }
        self.skip_ws();
        match self.peek() {
            Some(Token::RBrace) => { self.advance(); }
            _ => {}
        }
        self.in_body = prev;
        Ok(nodes)
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
        match self.peek_keyword(kw) {
            true => {
                self.advance();
                Ok(())
            }
            false => Err(SassError::Parse {
                expected: kw.into(),
                found: "other".into(),
            }),
        }
    }
}
