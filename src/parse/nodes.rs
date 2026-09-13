//! 节点解析——`ParseStream` 的 `&mut self` 方法。
//!
//! 所有解析方法消费 token 自然推进 pos。
//! 与 `tokio_stream::Stream::poll_next(Pin<&mut Self>, _)` 同态。

use crate::error::{Result, SassError};
use crate::lex::token::Token;

use super::ast::*;
use super::ParseStream;

impl<'tok> ParseStream<'tok> {
    // ═══════════════════════════════════════════════════════════════════════
    // 节点解析
    // ═══════════════════════════════════════════════════════════════════════

    /// 根据首个 token 分派到具体解析器。
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self), fields(pos = self.pos)))]
    pub(crate) fn parse_node(&mut self) -> Result<Node> {
        self.skip_ws();
        let peek_str = self
            .peek()
            .map_or_else(|| "EOF".into(), std::string::ToString::to_string);
        #[cfg(feature = "tracing")]
        tracing::trace!(peek = %peek_str, "parse_node");
        match self.peek() {
            Some(Token::AtRule(name)) => self.parse_at_rule(name.clone()),
            Some(Token::Dollar(_)) => self.parse_variable(),
            Some(Token::Comment(t, silent)) => {
                let node = Node::Comment(t.clone(), *silent);
                self.advance();
                Ok(node)
            }
            Some(Token::Semicolon) => {
                self.advance();
                self.skip_ws();
                match self.peek() {
                    None | Some(Token::Eof | Token::RBrace) => {
                        Ok(Node::Comment(String::new(), true))
                    }
                    _ => self.parse_node(),
                }
            }
            Some(Token::Whitespace) => {
                self.advance();
                self.parse_node()
            }
            _ => match self.is_namespace_var() {
                true => self.parse_namespace_var(),
                false => self.parse_rule_or_decl(),
            },
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

    /// 根据 is_rule 决定走 parse_rule 还是 parse_decl。
    pub(crate) fn parse_rule_or_decl(&mut self) -> Result<Node> {
        match self.is_rule() {
            true => self.parse_rule(),
            false => self.parse_decl(),
        }
    }

    /// 解析 CSS 规则——`selector { body }`。
    pub(crate) fn parse_rule(&mut self) -> Result<Node> {
        let selector = self.parse_selector()?;
        self.skip_ws();
        self.expect(&Token::LBrace)?;
        let body = self.parse_body()?;
        Ok(Node::Rule { selector, body })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // 选择器解析
    // ═══════════════════════════════════════════════════════════════════════

    /// 解析选择器到 `{`。
    pub(crate) fn parse_selector(&mut self) -> Result<String> {
        let mut s = String::new();
        let mut bracket_depth = 0i32;
        while let Some(t) = self.peek() {
            #[cfg(feature = "tracing")]
            tracing::trace!(token = ?t, accumulated = %s, "parse_selector token");
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
                                self.advance();
                            }
                            (false, Some(Token::RBracket | Token::Assign | Token::Tilde | Token::Pipe | Token::Caret | Token::Star), _) => {
                                self.advance();
                            }
                            (false, _, true) => {
                                match next_non_ws {
                                    Some(Token::Ident(id))
                                        if id.len() == 1
                                            && id.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) =>
                                    {
                                        let mod_id = id.clone();
                                        let after_mod = self.peek_n(look + 1);
                                        match matches!(after_mod, Some(Token::RBracket)) {
                                            true => {
                                                for _ in 0..look {
                                                    self.advance();
                                                }
                                                if !s.ends_with(' ') {
                                                    s.push(' ');
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
                                return Err(SassError::Parse {
                                    expected: "]".into(),
                                    found: "modifier".into(),
                                });
                            }
                        }
                    } else {
                        s.push(' ');
                        self.advance();
                    }
                }
                Token::Comment(_, _) => {
                    self.advance();
                }
                _ => {
                    s.push_str(&t.to_string());
                    self.advance();
                }
            }
        }
        Ok(s.trim().to_string())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // 声明解析
    // ═══════════════════════════════════════════════════════════════════════

    /// 解析声明——`property: value;`。
    pub(crate) fn parse_decl(&mut self) -> Result<Node> {
        let property = self.parse_property()?;
        self.skip_ws_and_comments();
        self.expect(&Token::Colon)?;
        self.skip_ws_and_comments();
        let value = self.parse_decl_value()?;
        let important = self.check_important()?;
        self.skip_ws_and_comments();
        if matches!(self.peek(), Some(Token::Semicolon)) {
            self.advance();
        }
        Ok(Node::Decl {
            property,
            value,
            important,
        })
    }

    /// 解析属性名。
    fn parse_property(&mut self) -> Result<String> {
        let mut s = String::new();
        while let Some(t) = self.peek() {
            match t {
                Token::Colon | Token::Whitespace | Token::RBrace | Token::Semicolon => break,
                Token::Comment(_, _) => {
                    self.advance();
                }
                _ => {
                    s.push_str(&t.to_string());
                    self.advance();
                }
            }
        }
        Ok(s)
    }

    /// 检查并消费 `!important`。
    pub(crate) fn check_important(&mut self) -> Result<bool> {
        self.skip_ws_and_comments();
        if matches!(self.peek(), Some(Token::Bang)) {
            self.advance();
            self.skip_ws_and_comments();
            if matches!(self.peek(), Some(Token::Ident(s)) if s == "important") {
                self.advance();
                return Ok(true);
            }
        }
        Ok(false)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // 变量解析
    // ═══════════════════════════════════════════════════════════════════════

    /// 解析变量赋值——`$var: value;`。
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
        self.skip_ws_and_comments();
        self.expect(&Token::Colon)?;
        self.skip_ws_and_comments();
        let value = self.parse_value()?;
        let flags = self.parse_var_flags()?;
        self.skip_ws_and_comments();
        if matches!(self.peek(), Some(Token::Semicolon)) {
            self.advance();
        }
        self.skip_ws_and_comments();
        Ok(Node::Variable { name, value, flags })
    }

    /// 检测是否为命名空间变量赋值。
    fn is_namespace_var(&self) -> bool {
        let mut i = self.pos;
        while i < self.tokens.len() && matches!(self.tokens[i], Token::Whitespace) {
            i += 1;
        }
        if i >= self.tokens.len() || !matches!(self.tokens[i], Token::Ident(_)) {
            return false;
        }
        i += 1;
        while i < self.tokens.len() && matches!(self.tokens[i], Token::Whitespace) {
            i += 1;
        }
        if i >= self.tokens.len() || !matches!(self.tokens[i], Token::Dot) {
            return false;
        }
        i += 1;
        while i < self.tokens.len() && matches!(self.tokens[i], Token::Whitespace) {
            i += 1;
        }
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
        self.skip_ws();
        self.expect(&Token::Dot)?;
        self.skip_ws();
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
        if flags.global {
            return Err(SassError::Eval(
                "!global isn't allowed for variables in other modules.".into(),
            ));
        }
        self.skip_ws();
        if matches!(self.peek(), Some(Token::Semicolon)) {
            self.advance();
        }
        Ok(Node::Variable { name, value, flags })
    }

    /// 解析变量标志 `!default` `!global`。
    pub(crate) fn parse_var_flags(&mut self) -> Result<VarFlags> {
        let mut flags = VarFlags::default();
        self.skip_ws();
        while matches!(self.peek(), Some(Token::Bang)) {
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

    // ═══════════════════════════════════════════════════════════════════════
    // 规则体解析
    // ═══════════════════════════════════════════════════════════════════════

    pub(crate) fn parse_body(&mut self) -> Result<Vec<Node>> {
        let mut nodes = Vec::new();
        let prev_in_body = self.in_body;
        self.in_body = true;
        loop {
            self.skip_ws();
            match self.peek() {
                Some(Token::RBrace | Token::Eof) | None => break,
                _ => nodes.push(self.parse_node()?),
            }
        }
        self.skip_ws();
        if matches!(self.peek(), Some(Token::RBrace)) {
            self.advance();
        }
        self.in_body = prev_in_body;
        Ok(nodes)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // 辅助方法
    // ═══════════════════════════════════════════════════════════════════════

    /// 解析标识符名称。
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

    /// 解析字符串值。
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

    /// 检查当前 token 是否匹配关键字。
    pub(crate) fn peek_keyword(&self, kw: &str) -> bool {
        matches!(self.peek(), Some(Token::Ident(s)) if s == kw)
    }

    /// 消费期望的关键字。
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
