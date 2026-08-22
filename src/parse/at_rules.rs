//! @-rule 解析（@use / @forward / @import / @include / @mixin / @if 等）。

use crate::error::{Result, SassError};
use crate::lex::Token;
use crate::parse::{Node, Parser};
use crate::eval::value::Value;

impl Parser {
    /// 解析 @ 开头的 at-rule。
    pub fn parse_at_rule(&mut self) -> Result<Node> {
        self.bump(); // @
        let name = match self.bump() {
            Token::Ident(s) => s,
            t => return Err(SassError::parse(format!("Expected ident after @, got {:?}", t))),
        };

        match name.as_str() {
            "if" => self.parse_if(),
            "for" => self.parse_for(),
            "each" => self.parse_each(),
            "while" => self.parse_while(),
            "mixin" => self.parse_mixin_def(),
            "function" => self.parse_function_def(),
            "include" => self.parse_include(),
            "content" => {
                self.consume_until_semicolon();
                Ok(Node::Content)
            }
            "use" => self.parse_use(),
            "forward" => self.parse_forward(),
            "import" => self.parse_import(),
            "extend" => self.parse_extend(),
            "at-root" => self.parse_at_root(),
            "warn" => {
                let v = self.parse_value()?;
                self.consume_until_semicolon();
                Ok(Node::Warn(v))
            }
            "debug" => {
                let v = self.parse_value()?;
                self.consume_until_semicolon();
                Ok(Node::Debug(v))
            }
            "error" => {
                let v = self.parse_value()?;
                self.consume_until_semicolon();
                Ok(Node::Error(v))
            }
            "return" => {
                let v = self.parse_value()?;
                self.consume_until_semicolon();
                Ok(Node::Return(v))
            }
            _ => self.parse_generic_at_rule(&name),
        }
    }

    fn parse_if(&mut self) -> Result<Node> {
        let cond = self.parse_value()?;
        self.eat(&Token::LBrace)?;
        let body = self.parse_body()?;
        self.eat(&Token::RBrace)?;

        let mut branches = vec![(cond, body)];
        let mut else_body = None;

        // @else if / @else
        loop {
            // 检查 @else
            if matches!(self.peek(), Token::At) {
                let saved = self.pos;
                self.bump(); // @
                if let Token::Ident(s) = self.peek() {
                    if s == "else" {
                        self.bump(); // else
                        // else if?
                        if let Token::Ident(s2) = self.peek() {
                            if s2 == "if" {
                                self.bump(); // if
                                let cond2 = self.parse_value()?;
                                self.eat(&Token::LBrace)?;
                                let body2 = self.parse_body()?;
                                self.eat(&Token::RBrace)?;
                                branches.push((cond2, body2));
                                continue;
                            }
                        }
                        // plain else
                        self.eat(&Token::LBrace)?;
                        let body2 = self.parse_body()?;
                        self.eat(&Token::RBrace)?;
                        else_body = Some(body2);
                        break;
                    }
                }
                // 不是 @else，恢复
                self.pos = saved;
            }
            break;
        }

        Ok(Node::If { branches, else_body })
    }

    fn parse_for(&mut self) -> Result<Node> {
        // $var from X through/to Y { body }
        let var = match self.bump() {
            Token::Variable(n) => n,
            t => return Err(SassError::parse(format!("Expected variable after @for, got {:?}", t))),
        };
        // from
        if let Token::Ident(s) = self.peek() {
            if s != "from" {
                return Err(SassError::parse("Expected 'from' in @for"));
            }
            self.bump();
        }
        let from = self.parse_value()?;
        // through / to
        let inclusive = match self.peek() {
            Token::Ident(s) if s == "through" => { self.bump(); true }
            Token::Ident(s) if s == "to" => { self.bump(); false }
            _ => return Err(SassError::parse("Expected 'through' or 'to' in @for")),
        };
        let to = self.parse_value()?;
        self.eat(&Token::LBrace)?;
        let body = self.parse_body()?;
        self.eat(&Token::RBrace)?;
        Ok(Node::For { var, from, to, inclusive, body })
    }

    fn parse_each(&mut self) -> Result<Node> {
        // $var in list { body }
        let mut vars = Vec::new();
        loop {
            match self.bump() {
                Token::Variable(n) => vars.push(n),
                Token::Ident(s) if s == "in" => break,
                Token::Comma => continue,
                t => return Err(SassError::parse(format!("Expected variable or 'in' in @each, got {:?}", t))),
            }
        }
        let list = self.parse_value()?;
        self.eat(&Token::LBrace)?;
        let body = self.parse_body()?;
        self.eat(&Token::RBrace)?;
        Ok(Node::Each { vars, list, body })
    }

    fn parse_while(&mut self) -> Result<Node> {
        let cond = self.parse_value()?;
        self.eat(&Token::LBrace)?;
        let body = self.parse_body()?;
        self.eat(&Token::RBrace)?;
        Ok(Node::While { cond, body })
    }

    fn parse_mixin_def(&mut self) -> Result<Node> {
        let name = match self.bump() {
            Token::Ident(s) => s,
            t => return Err(SassError::parse(format!("Expected mixin name, got {:?}", t))),
        };
        let params = self.parse_params()?;
        self.eat(&Token::LBrace)?;
        let body = self.parse_body()?;
        self.eat(&Token::RBrace)?;
        Ok(Node::MixinDef { name, params, body })
    }

    fn parse_function_def(&mut self) -> Result<Node> {
        let name = match self.bump() {
            Token::Ident(s) => s,
            t => return Err(SassError::parse(format!("Expected function name, got {:?}", t))),
        };
        let params = self.parse_params()?;
        self.eat(&Token::LBrace)?;
        let body = self.parse_body()?;
        self.eat(&Token::RBrace)?;
        Ok(Node::FunctionDef { name, params, body })
    }

    fn parse_include(&mut self) -> Result<Node> {
        let name = match self.bump() {
            Token::Ident(s) => s,
            t => return Err(SassError::parse(format!("Expected mixin name in @include, got {:?}", t))),
        };
        let args = self.parse_args()?;

        // optional @content block
        let content = if matches!(self.peek(), Token::LBrace) {
            self.bump();
            let body = self.parse_body()?;
            self.eat(&Token::RBrace)?;
            Some(body)
        } else {
            self.consume_until_semicolon();
            None
        };

        Ok(Node::Include { name, args, content })
    }

    fn parse_use(&mut self) -> Result<Node> {
        let url = match self.bump() {
            Token::String(s, _) => s,
            t => return Err(SassError::parse(format!("Expected string in @use, got {:?}", t))),
        };

        let namespace = if let Token::Ident(s) = self.peek() {
            if s == "as" {
                self.bump();
                match self.bump() {
                    Token::Ident(n) => Some(n),
                    Token::Star => Some("*".to_string()),
                    t => return Err(SassError::parse(format!("Expected namespace after 'as', got {:?}", t))),
                }
            } else { None }
        } else { None };

        let star = namespace.as_deref() == Some("*");

        // with (config)
        let config = if let Token::Ident(s) = self.peek() {
            if s == "with" {
                self.bump();
                self.parse_config()?
            } else { Vec::new() }
        } else { Vec::new() };

        self.consume_until_semicolon();
        Ok(Node::Use { url, namespace, star, config })
    }

    fn parse_forward(&mut self) -> Result<Node> {
        let url = match self.bump() {
            Token::String(s, _) => s,
            t => return Err(SassError::parse(format!("Expected string in @forward, got {:?}", t))),
        };

        let mut show = Vec::new();
        let mut hide = Vec::new();
        let mut prefix = None;

        if let Token::Ident(s) = self.peek() {
            match s.as_str() {
                "show" => {
                    self.bump();
                    show = self.parse_name_list()?;
                }
                "hide" => {
                    self.bump();
                    hide = self.parse_name_list()?;
                }
                "as" => {
                    self.bump();
                    prefix = match self.bump() {
                        Token::Ident(p) => Some(p),
                        t => return Err(SassError::parse(format!("Expected prefix after 'as', got {:?}", t))),
                    };
                }
                _ => {}
            }
        }

        let config = if let Token::Ident(s) = self.peek() {
            if s == "with" {
                self.bump();
                self.parse_config()?
            } else { Vec::new() }
        } else { Vec::new() };

        self.consume_until_semicolon();
        Ok(Node::Forward { url, show, hide, prefix, config })
    }

    fn parse_import(&mut self) -> Result<Node> {
        let url = match self.bump() {
            Token::String(s, _) => s,
            t => return Err(SassError::parse(format!("Expected string in @import, got {:?}", t))),
        };

        // modifier (plain css)
        let modifier = if let Token::Ident(s) = self.peek() {
            let m = s.clone();
            self.bump();
            Some(m)
        } else { None };

        self.consume_until_semicolon();
        Ok(Node::Import { url, modifier })
    }

    fn parse_extend(&mut self) -> Result<Node> {
        // 收集选择器
        let mut sel = String::new();
        while !matches!(self.peek(), Token::Semicolon | Token::Eof) {
            sel.push_str(&match self.bump() {
                Token::Ident(s) => s,
                Token::Variable(s) => format!("${s}"),
                Token::Dot => ".".to_string(),
                Token::Hash => "#".to_string(),
                Token::Colon => ":".to_string(),
                Token::Ampersand => "&".to_string(),
                t => t.as_str().to_string(),
            });
        }
        self.consume_until_semicolon();
        let optional = sel.ends_with("!optional");
        if optional { sel = sel.trim_end_matches("!optional").trim().to_string(); }
        Ok(Node::Extend { selector: sel.trim().to_string(), optional })
    }

    fn parse_at_root(&mut self) -> Result<Node> {
        // (query) or body
        let query = if matches!(self.peek(), Token::LParen) {
            self.bump();
            let mut q = String::new();
            while !matches!(self.peek(), Token::RParen) {
                q.push_str(self.bump().as_str());
            }
            self.bump();
            Some(q)
        } else { None };

        self.eat(&Token::LBrace)?;
        let body = self.parse_body()?;
        self.eat(&Token::RBrace)?;
        Ok(Node::AtRoot { query, body })
    }

    fn parse_generic_at_rule(&mut self, name: &str) -> Result<Node> {
        // 收集 params 直到 { 或 ;
        let mut params = String::new();
        loop {
            match self.peek() {
                Token::LBrace | Token::Semicolon | Token::Eof => break,
                t => {
                    params.push_str(t.as_str());
                    self.bump();
                }
            }
        }

        let body = if matches!(self.peek(), Token::LBrace) {
            self.bump();
            let b = self.parse_body()?;
            self.eat(&Token::RBrace)?;
            Some(b)
        } else {
            None
        };
        if matches!(self.peek(), Token::Semicolon) { self.bump(); }

        Ok(Node::AtRule {
            name: name.to_string(),
            params: params.trim().to_string(),
            body,
        })
    }

    fn parse_config(&mut self) -> Result<Vec<(String, Value)>> {
        let mut config = Vec::new();
        self.eat(&Token::LParen)?;
        if matches!(self.peek(), Token::RParen) {
            self.bump();
            return Ok(config);
        }
        loop {
            let name = match self.bump() {
                Token::Variable(n) => n,
                t => return Err(SassError::parse(format!("Expected config var, got {:?}", t))),
            };
            self.eat(&Token::Colon)?;
            let value = self.parse_value()?;
            config.push((name, value));
            match self.peek() {
                Token::Comma => { self.bump(); }
                Token::RParen => { self.bump(); break; }
                t => return Err(SassError::parse(format!("Expected , or ), got {:?}", t))),
            }
        }
        Ok(config)
    }

    fn parse_name_list(&mut self) -> Result<Vec<String>> {
        let mut names = Vec::new();
        loop {
            match self.bump() {
                Token::Ident(n) => names.push(n),
                Token::Comma => continue,
                _ => break,
            }
        }
        Ok(names)
    }

    fn consume_until_semicolon(&mut self) {
        while !matches!(self.peek(), Token::Semicolon | Token::Eof) {
            self.bump();
        }
        if matches!(self.peek(), Token::Semicolon) {
            self.bump();
        }
    }
}
