//! @规则解析。
//!
//! 包含 parse_at_rule/parse_if/parse_for/parse_each/parse_while 等所有 @ 规则解析方法。

use super::Parser;
use super::ast::*;
use crate::error::{Result, SassError};
use crate::lex::token::Token;

impl<'tok> Parser<'tok> {
    // —— @规则解析 ——
    pub(crate) fn parse_at_rule(&mut self, name: String) -> Result<Node> {
        self.advance(); // 消费 @rule
        match name.as_str() {
            "if" => self.parse_if(),
            "for" => self.parse_for(),
            "each" => self.parse_each(),
            "while" => self.parse_while(),
            "mixin" => self.parse_mixin_def(),
            "include" => self.parse_include(),
            "content" => {
                self.skip_ws();
                if self.peek() == Some(&Token::Semicolon) {
                    self.advance();
                }
                Ok(Node::Content)
            }
            "function" => self.parse_function_def(),
            "return" => self.parse_return(),
            "use" => self.parse_use(),
            "forward" => self.parse_forward(),
            "import" => self.parse_import(),
            "extend" => self.parse_extend(),
            "at-root" => self.parse_at_root(),
            "warn" => self.parse_warn(),
            "debug" => self.parse_debug(),
            "error" => self.parse_error(),
            _ => self.parse_generic_at_rule(name),
        }
    }

    pub(crate) fn parse_if(&mut self) -> Result<Node> {
        self.skip_ws();
        let cond = self.parse_value()?;
        self.skip_ws();
        self.expect(&Token::LBrace)?;
        let body = self.parse_body()?;
        let mut branches = vec![(cond, body)];
        let mut else_body = None;
        // @else if / @else
        loop {
            self.skip_ws();
            if !matches!(self.peek(), Some(Token::AtRule(n)) if n == "else") {
                break;
            }
            self.advance(); // 消费 @else
            self.skip_ws();
            // @else if —— if 可能是 AtRule("if") 或 Ident("if")
            let is_else_if = match self.peek() {
                Some(Token::AtRule(n)) if n == "if" => true,
                Some(Token::Ident(n)) if n == "if" => true,
                _ => false,
            };
            if is_else_if {
                self.advance(); // 消费 if
                self.skip_ws();
                let cond2 = self.parse_value()?;
                self.skip_ws();
                self.expect(&Token::LBrace)?;
                let body2 = self.parse_body()?;
                branches.push((cond2, body2));
            } else {
                self.skip_ws();
                self.expect(&Token::LBrace)?;
                else_body = Some(self.parse_body()?);
                break;
            }
        }
        Ok(Node::If {
            branches,
            else_body,
        })
    }

    pub(crate) fn parse_for(&mut self) -> Result<Node> {
        self.skip_ws();
        let var = match self.peek() {
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
        self.expect_keyword("from")?;
        self.skip_ws();
        let from = self.parse_value()?;
        self.skip_ws();
        let inclusive = if self.peek_keyword("through") {
            self.advance();
            true
        } else {
            self.expect_keyword("to")?;
            false
        };
        self.skip_ws();
        let to = self.parse_value()?;
        self.skip_ws();
        self.expect(&Token::LBrace)?;
        let body = self.parse_body()?;
        Ok(Node::For {
            var,
            from,
            to,
            inclusive,
            body,
        })
    }

    pub(crate) fn parse_each(&mut self) -> Result<Node> {
        self.skip_ws();
        let mut vars = Vec::new();
        loop {
            self.skip_ws();
            match self.peek() {
                Some(Token::Dollar(n)) => {
                    vars.push(n.clone());
                    self.advance();
                }
                _ => break,
            }
            self.skip_ws();
            if self.peek() == Some(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.skip_ws();
        self.expect_keyword("in")?;
        self.skip_ws();
        let list = self.parse_value()?;
        self.skip_ws();
        self.expect(&Token::LBrace)?;
        let body = self.parse_body()?;
        Ok(Node::Each { vars, list, body })
    }

    pub(crate) fn parse_while(&mut self) -> Result<Node> {
        self.skip_ws();
        let cond = self.parse_value()?;
        self.skip_ws();
        self.expect(&Token::LBrace)?;
        let body = self.parse_body()?;
        Ok(Node::While { cond, body })
    }

    pub(crate) fn parse_mixin_def(&mut self) -> Result<Node> {
        self.skip_ws();
        let name = self.parse_ident_name()?;
        self.skip_ws();
        let params = if self.peek() == Some(&Token::LParen) {
            self.parse_params()?
        } else {
            Vec::new()
        };
        self.skip_ws();
        self.expect(&Token::LBrace)?;
        let body = self.parse_body()?;
        Ok(Node::MixinDef { name, params, body })
    }

    pub(crate) fn parse_include(&mut self) -> Result<Node> {
        self.skip_ws();
        let name = self.parse_ident_name()?;
        // 命名空间限定 mixin（如 midstream.b-a）
        let name = if self.peek() == Some(&Token::Dot) {
            self.advance();
            let rest = self.parse_ident_name()?;
            format!("{name}.{rest}")
        } else {
            name
        };
        self.skip_ws();
        let args = if self.peek() == Some(&Token::LParen) {
            self.parse_args()?
        } else {
            Vec::new()
        };
        // 检查 @content 块
        let mut content = None;
        self.skip_ws();
        if self.peek() == Some(&Token::LBrace) {
            self.advance();
            content = Some(self.parse_body()?);
        } else if self.peek() == Some(&Token::Semicolon) {
            self.advance();
        }
        Ok(Node::Include {
            name,
            args,
            content,
        })
    }

    pub(crate) fn parse_function_def(&mut self) -> Result<Node> {
        self.skip_ws();
        let name = self.parse_ident_name()?;
        self.skip_ws();
        let params = if self.peek() == Some(&Token::LParen) {
            self.parse_params()?
        } else {
            Vec::new()
        };
        self.skip_ws();
        self.expect(&Token::LBrace)?;
        let body = self.parse_body()?;
        Ok(Node::FunctionDef { name, params, body })
    }

    pub(crate) fn parse_return(&mut self) -> Result<Node> {
        self.skip_ws();
        let value = self.parse_value()?;
        self.skip_ws();
        // @return 可用 ; 结束，也可在函数体末尾以 } 结束（无分号）
        if self.peek() == Some(&Token::Semicolon) {
            self.advance();
        }
        Ok(Node::Return(value))
    }

    pub(crate) fn parse_use(&mut self) -> Result<Node> {
        self.skip_ws();
        let url = self.parse_string_value()?;
        let mut namespace = None;
        let mut star = false;
        let mut config = Vec::new();
        self.skip_ws();
        if self.peek_keyword("as") {
            self.advance(); // 消费 as
            self.skip_ws();
            if self.peek() == Some(&Token::Star) {
                self.advance();
                star = true;
            } else {
                namespace = Some(self.parse_ident_name()?);
            }
        }
        self.skip_ws();
        if self.peek_keyword("with") {
            self.advance(); // 消费 with 关键字
            self.skip_ws();
            self.expect(&Token::LParen)?;
            config = self.parse_config()?;
        }
        self.skip_ws();
        if self.peek() == Some(&Token::Semicolon) {
            self.advance();
        }
        Ok(Node::Use {
            url,
            namespace,
            star,
            config,
        })
    }

    pub(crate) fn parse_forward(&mut self) -> Result<Node> {
        self.skip_ws();
        let url = self.parse_string_value()?;
        let mut show = Vec::new();
        let mut hide = Vec::new();
        let mut prefix = None;
        self.skip_ws();
        if self.peek_keyword("as") {
            self.advance(); // 消费 as
            self.skip_ws();
            // as prefix-*
            if let Some(Token::Ident(s)) = self.peek() {
                prefix = Some(s.clone());
                self.advance();
            }
            if self.peek() == Some(&Token::Star) {
                self.advance();
            }
            self.skip_ws();
        }
        if self.peek_keyword("show") {
            self.advance(); // 消费 show
            show = self.parse_member_list();
        } else if self.peek_keyword("hide") {
            self.advance(); // 消费 hide
            hide = self.parse_member_list();
        }
        self.skip_ws();
        if self.peek() == Some(&Token::Semicolon) {
            self.advance();
        }
        Ok(Node::Forward {
            url,
            show,
            hide,
            prefix,
        })
    }

    pub(crate) fn parse_import(&mut self) -> Result<Node> {
        self.skip_ws();
        let url = self.parse_string_value()?;
        self.skip_ws();
        if self.peek() == Some(&Token::Semicolon) {
            self.advance();
        }
        Ok(Node::Import { url })
    }

    pub(crate) fn parse_extend(&mut self) -> Result<Node> {
        self.skip_ws();
        let mut selector = String::new();
        let mut optional = false;
        while let Some(t) = self.peek() {
            match t {
                Token::Semicolon | Token::RBrace => break,
                Token::Bang => {
                    self.advance();
                    self.skip_ws();
                    if let Some(Token::Ident(s)) = self.peek()
                        && s == "optional" {
                            optional = true;
                            self.advance();
                        }
                }
                Token::Whitespace => {
                    selector.push(' ');
                    self.advance();
                }
                _ => {
                    selector.push_str(&t.to_string());
                    self.advance();
                }
            }
        }
        self.skip_ws();
        if self.peek() == Some(&Token::Semicolon) {
            self.advance();
        }
        Ok(Node::Extend {
            selector: selector.trim().to_string(),
            optional,
        })
    }

    pub(crate) fn parse_at_root(&mut self) -> Result<Node> {
        self.skip_ws();
        let query = if self.peek() == Some(&Token::LParen) {
            self.advance();
            let mut q = String::new();
            while let Some(t) = self.peek() {
                if t == &Token::RParen {
                    break;
                }
                q.push_str(&t.to_string());
                self.advance();
            }
            if self.peek() == Some(&Token::RParen) {
                self.advance();
            }
            Some(q.trim().to_string())
        } else {
            None
        };
        // 可能有选择器前缀
        self.skip_ws();
        if self.peek() != Some(&Token::LBrace) {
            // 选择器 + { body }
            let sel = self.parse_selector()?;
            let _ = sel; // 简化：忽略 at-root 选择器前缀
        }
        self.skip_ws();
        self.expect(&Token::LBrace)?;
        let body = self.parse_body()?;
        Ok(Node::AtRoot { query, body })
    }

    pub(crate) fn parse_warn(&mut self) -> Result<Node> {
        self.skip_ws();
        let v = self.parse_value()?;
        self.skip_ws();
        if self.peek() == Some(&Token::Semicolon) {
            self.advance();
        }
        Ok(Node::Warn(v))
    }
    pub(crate) fn parse_debug(&mut self) -> Result<Node> {
        self.skip_ws();
        let v = self.parse_value()?;
        self.skip_ws();
        if self.peek() == Some(&Token::Semicolon) {
            self.advance();
        }
        Ok(Node::Debug(v))
    }
    pub(crate) fn parse_error(&mut self) -> Result<Node> {
        self.skip_ws();
        let v = self.parse_value()?;
        self.skip_ws();
        if self.peek() == Some(&Token::Semicolon) {
            self.advance();
        }
        Ok(Node::Error(v))
    }

    pub(crate) fn parse_generic_at_rule(&mut self, name: String) -> Result<Node> {
        self.skip_ws();
        let params = if !matches!(
            self.peek(),
            Some(Token::LBrace) | Some(Token::Semicolon) | None
        ) {
            Some(self.parse_at_params()?)
        } else {
            None
        };
        self.skip_ws();
        let body = if self.peek() == Some(&Token::LBrace) {
            self.advance();
            Some(self.parse_body()?)
        } else {
            None
        };
        self.skip_ws();
        if self.peek() == Some(&Token::Semicolon) {
            self.advance();
        }
        Ok(Node::AtRule { name, params, body })
    }

    pub(crate) fn parse_at_params(&mut self) -> Result<String> {
        let mut s = String::new();
        while let Some(t) = self.peek() {
            match t {
                Token::LBrace | Token::Semicolon | Token::Eof => break,
                Token::Comment(_, _) => {
                    self.advance();
                } // 跳过注释
                Token::Whitespace => {
                    // 规范化：多个连续空白合并为单个空格，但不在 ( 后添加空格
                    if !s.ends_with(' ') && !s.ends_with('(') {
                        s.push(' ');
                    }
                    self.advance();
                }
                _ => {
                    s.push_str(&t.to_string());
                    self.advance();
                }
            }
        }
        // 标准化冒号周围空白：": " 前不应有空格，后跟单个空格
        let s = s.replace(" :", ":").replace(":  ", ": ");
        Ok(s.trim().to_string())
    }
}
