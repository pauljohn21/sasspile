//! 语法分析器——递归下降 + Result 组合。
//!
//! 将 Token 序列转换为抽象语法树（AST）。

use crate::error::{Result, SassError};
use crate::lex::token::Token;
use crate::parse::ast::{Ast, Node};

/// 语法分析器——将 Token 序列转为 AST。
pub struct Parser<'tok> {
    tokens: &'tok [Token],
    pub(super) pos: usize,
}

impl<'tok> Parser<'tok> {
    /// 创建新的 Parser。
    pub fn new(tokens: &'tok [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    /// 解析入口——Token 流 → AST。
    pub fn parse(tokens: &'tok [Token]) -> Result<Ast> {
        let mut parser = Self::new(tokens);
        let mut nodes = Vec::new();
        while !parser.at_end() {
            nodes.push(parser.parse_node()?);
        }
        Ok(Ast { nodes })
    }

    /// 查看当前 token（不消费）。
    pub(super) fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    /// 消费当前 token 并前进。
    pub(super) fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.pos);
        if token.is_some() && !matches!(token, Some(Token::Eof)) {
            self.pos += 1;
        }
        token
    }

    /// 判断是否到达末尾（跳过尾部空白）。
    pub(super) fn at_end(&self) -> bool {
        let mut pos = self.pos;
        while matches!(self.tokens.get(pos), Some(Token::Whitespace)) {
            pos += 1;
        }
        matches!(self.tokens.get(pos), None | Some(Token::Eof))
    }

    /// 跳过空白 token。
    pub(super) fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(Token::Whitespace)) {
            self.pos += 1;
        }
    }

    /// 解析单个节点。
    pub(super) fn parse_node(&mut self) -> Result<Node> {
        self.skip_whitespace();
        match self.peek() {
            Some(Token::AtRule(name)) => self.parse_at_rule(name.clone()),
            Some(Token::Dollar(_)) => self.parse_variable_decl(),
            Some(Token::Ident(_)) | Some(Token::Hash(_)) | Some(Token::Dot) | Some(Token::Star) => {
                self.parse_rule_or_decl()
            }
            Some(Token::Comment(text)) => {
                let text = text.clone();
                self.advance();
                Ok(Node::Comment(text))
            }
            Some(Token::Whitespace) => {
                self.advance();
                self.parse_node()
            }
            Some(other) => Err(SassError::ParseError {
                expected: "节点开始".to_string(),
                found: other.to_string(),
            }),
            None => Err(SassError::ParseError {
                expected: "节点".to_string(),
                found: "EOF".to_string(),
            }),
        }
    }

    /// 解析规则或声明（根据后续 token 判断）。
    fn parse_rule_or_decl(&mut self) -> Result<Node> {
        // 用 lookahead 判断是规则还是声明：
        // 向后扫描，如果先遇到 { 则是规则；如果先遇到 : 则是声明
        let is_rule = self.lookahead_is_rule();

        if is_rule {
            let selector = self.parse_selector()?;
            self.skip_whitespace();
            if self.peek() == Some(&Token::LBrace) {
                self.advance();
            }
            let body = self.parse_body()?;
            self.skip_whitespace();
            if self.peek() == Some(&Token::RBrace) {
                self.advance();
            }
            Ok(Node::Rule { selector, body })
        } else {
            // 声明
            let property = self.parse_property_name()?;
            self.skip_whitespace();
            if self.peek() == Some(&Token::Colon) {
                self.advance();
            }
            self.parse_decl_after_property(property)
        }
    }

    /// 解析属性名（到 : 为止）。
    fn parse_property_name(&mut self) -> Result<String> {
        let mut parts = Vec::new();
        while let Some(token) = self.peek() {
            match token {
                Token::Ident(s) => {
                    parts.push(s.clone());
                    self.advance();
                }
                Token::Minus => {
                    parts.push("-".to_string());
                    self.advance();
                }
                Token::Whitespace => {
                    self.advance();
                }
                _ => break,
            }
        }
        Ok(parts.join(""))
    }

    /// Lookahead 判断：向后扫描，{ 先出现是规则，; 先出现是声明。
    fn lookahead_is_rule(&self) -> bool {
        for i in self.pos..self.tokens.len() {
            match self.tokens.get(i) {
                Some(Token::LBrace) => return true,
                Some(Token::Semicolon) => return false,
                Some(Token::Whitespace) => continue,
                _ => continue,
            }
        }
        true // 默认规则
    }

    /// 解析选择器——拼接 token 为字符串。
    fn parse_selector(&mut self) -> Result<String> {
        let mut parts = Vec::new();
        while let Some(token) = self.peek() {
            match token {
                Token::Ident(_)
                | Token::Hash(_)
                | Token::Dot
                | Token::Star
                | Token::LBracket
                | Token::Comma
                | Token::Plus
                | Token::Greater
                | Token::Dollar(_)
                | Token::Number(_) => {
                    parts.push(token.to_string());
                    self.advance();
                }
                Token::Colon => {
                    parts.push(token.to_string());
                    self.advance();
                }
                Token::Whitespace => {
                    parts.push(" ".to_string());
                    self.advance();
                    if !matches!(
                        self.peek(),
                        Some(Token::Ident(_) | Token::Hash(_) | Token::Dot | Token::Colon | Token::Star | Token::Comma | Token::Plus | Token::Greater)
                    ) {
                        break;
                    }
                }
                _ => break,
            }
        }
        Ok(parts.join("").trim().to_string())
    }

    /// 解析声明（: 已被消费）。
    fn parse_decl_after_property(&mut self, property: String) -> Result<Node> {
        self.skip_whitespace();
        let value = self.parse_value()?;
        self.skip_whitespace();
        // 检查 !important
        let important = if self.peek() == Some(&Token::Bang) {
            self.advance();
            self.skip_whitespace();
            if matches!(self.peek(), Some(Token::Ident(s)) if s == "important") {
                self.advance();
            }
            true
        } else {
            false
        };
        self.skip_whitespace();
        if self.peek() == Some(&Token::Semicolon) {
            self.advance();
        }
        Ok(Node::Decl {
            property,
            value,
            important,
        })
    }

    /// 解析规则体。
    fn parse_body(&mut self) -> Result<Vec<Node>> {
        let mut nodes = Vec::new();
        loop {
            self.skip_whitespace();
            match self.peek() {
                Some(Token::RBrace) | None | Some(Token::Eof) => break,
                _ => nodes.push(self.parse_node()?),
            }
        }
        Ok(nodes)
    }

    /// 解析变量声明。
    fn parse_variable_decl(&mut self) -> Result<Node> {
        let name = match self.peek() {
            Some(Token::Dollar(name)) => {
                let n = name.clone();
                self.advance();
                n
            }
            other => {
                return Err(SassError::ParseError {
                    expected: "$变量名".to_string(),
                    found: format!("{other:?}"),
                })
            }
        };
        self.skip_whitespace();
        if self.peek() != Some(&Token::Colon) {
            return Err(SassError::ParseError {
                expected: ":".to_string(),
                found: format!("{:?}", self.peek()),
            });
        }
        self.advance();
        self.skip_whitespace();
        let value = self.parse_value()?;
        self.skip_whitespace();
        if self.peek() == Some(&Token::Semicolon) {
            self.advance();
        }
        Ok(Node::Variable { name, value })
    }

    /// 解析 @规则。
    fn parse_at_rule(&mut self, name: String) -> Result<Node> {
        self.advance(); // 消费 @
        self.skip_whitespace();
        // 解析参数（到 { 或 ; 为止），支持括号
        let params = if !matches!(self.peek(), Some(Token::LBrace | Token::Semicolon)) {
            Some(self.parse_at_rule_params()?)
        } else {
            None
        };
        self.skip_whitespace();
        let body = if self.peek() == Some(&Token::LBrace) {
            self.advance();
            let body = self.parse_body()?;
            if self.peek() == Some(&Token::RBrace) {
                self.advance();
            }
            Some(body)
        } else {
            None
        };
        self.skip_whitespace();
        if self.peek() == Some(&Token::Semicolon) {
            self.advance();
        }
        Ok(Node::AtRule { name, params, body })
    }

    /// 解析 @规则参数——支持括号和复杂表达式。
    fn parse_at_rule_params(&mut self) -> Result<String> {
        let mut parts = Vec::new();
        while let Some(token) = self.peek() {
            match token {
                Token::LBrace | Token::Semicolon | Token::Eof => break,
                Token::Whitespace => {
                    parts.push(" ".to_string());
                    self.advance();
                }
                _ => {
                    parts.push(token.to_string());
                    self.advance();
                }
            }
        }
        Ok(parts.join("").trim().to_string())
    }
}
