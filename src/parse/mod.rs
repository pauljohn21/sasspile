//! 语法分析器——递归下降 + Pratt 表达式解析。

pub mod ast;

use ast::*;
use crate::error::{Result, SassError};
use crate::lex::token::Token;
use tracing::{instrument, trace, warn};

/// 语法分析器。
pub struct Parser<'tok> {
    tokens: &'tok [Token],
    pos: usize,
}

impl<'tok> Parser<'tok> {
    /// 创建新的 Parser。
    pub fn new(tokens: &'tok [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    /// 解析入口。
    pub fn parse(tokens: &'tok [Token]) -> Result<Ast> {
        let mut p = Self::new(tokens);
        let mut nodes = Vec::new();
        while !p.at_end() {
            p.skip_ws();
            if p.at_end() {
                break;
            }
            nodes.push(p.parse_node()?);
        }
        Ok(Ast { nodes })
    }

    // —— 基础操作 ——
    fn peek(&self) -> Option<&Token> { self.tokens.get(self.pos) }
    fn peek_n(&self, n: usize) -> Option<&Token> { self.tokens.get(self.pos + n) }
    fn advance(&mut self) -> Option<&Token> {
        let t = self.tokens.get(self.pos);
        if !matches!(t, Some(Token::Eof) | None) { self.pos += 1; }
        t
    }
    fn at_end(&self) -> bool {
        let mut i = self.pos;
        while matches!(self.tokens.get(i), Some(Token::Whitespace)) { i += 1; }
        matches!(self.tokens.get(i), None | Some(Token::Eof))
    }
    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(Token::Whitespace) | Some(Token::Comment(_, _))) {
            self.pos += 1;
        }
    }

    fn expect(&mut self, tok: &Token) -> Result<()> {
        self.skip_ws();
        if self.peek() == Some(tok) {
            self.advance();
            Ok(())
        } else {
            let found = self.peek().map(|t| t.to_string()).unwrap_or("EOF".into());
            warn!(expected = %tok, found = %found, pos = self.pos, "expect failed");
            Err(SassError::Parse {
                expected: tok.to_string(),
                found,
            })
        }
    }

    // —— 节点解析 ——
    #[instrument(skip(self), fields(pos = self.pos))]
    fn parse_node(&mut self) -> Result<Node> {
        self.skip_ws();
        let peek_str = self.peek().map(|t| t.to_string()).unwrap_or_else(|| "EOF".into());
        trace!(peek = %peek_str, "parse_node");
        match self.peek() {
            Some(Token::AtRule(name)) => self.parse_at_rule(name.clone()),
            Some(Token::Dollar(_)) => self.parse_variable(),
            Some(Token::Comment(t, silent)) => {
                let node = Node::Comment(t.clone(), *silent);
                self.advance();
                Ok(node)
            }
            Some(Token::Whitespace) => { self.advance(); self.parse_node() }
            _ => self.parse_rule_or_decl(),
        }
    }

    /// Lookahead: { 先出现 → 规则, ; 或 } 先出现 → 声明。
    fn is_rule(&self) -> bool {
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
                Token::Whitespace => { i += 1; }
                _ => { i += 1; }
            }
        }
        true
    }

    #[instrument(skip(self), fields(pos = self.pos))]
    fn parse_rule_or_decl(&mut self) -> Result<Node> {
        let is_r = self.is_rule();
        trace!(is_rule = is_r, "parse_rule_or_decl");
        if is_r {
            self.parse_rule()
        } else {
            self.parse_decl()
        }
    }

    fn parse_rule(&mut self) -> Result<Node> {
        let selector = self.parse_selector()?;
        self.skip_ws();
        self.expect(&Token::LBrace)?;
        let body = self.parse_body()?;
        Ok(Node::Rule { selector, body })
    }

    fn parse_selector(&mut self) -> Result<String> {
        let mut s = String::new();
        while let Some(t) = self.peek() {
            match t {
                Token::LBrace => break,
                Token::Whitespace => { s.push(' '); self.advance(); }
                Token::Comment(_, _) => { self.advance(); } // 跳过注释
                _ => { s.push_str(&t.to_string()); self.advance(); }
            }
        }
        Ok(s.trim().to_string())
    }

    fn parse_decl(&mut self) -> Result<Node> {
        let property = self.parse_property()?;
        self.skip_ws();
        self.expect(&Token::Colon)?;
        self.skip_ws();
        let value = self.parse_value()?;
        let important = self.check_important()?;
        self.skip_ws();
        if self.peek() == Some(&Token::Semicolon) { self.advance(); }
        Ok(Node::Decl { property, value, important })
    }

    fn parse_property(&mut self) -> Result<String> {
        let mut s = String::new();
        while let Some(t) = self.peek() {
            match t {
                Token::Colon | Token::Whitespace | Token::RBrace | Token::Semicolon => break,
                _ => { s.push_str(&t.to_string()); self.advance(); }
            }
        }
        Ok(s)
    }

    fn check_important(&mut self) -> Result<bool> {
        self.skip_ws();
        if self.peek() == Some(&Token::Bang) {
            self.advance();
            self.skip_ws();
            if let Some(Token::Ident(s)) = self.peek() {
                if s == "important" { self.advance(); return Ok(true); }
            }
        }
        Ok(false)
    }

    fn parse_variable(&mut self) -> Result<Node> {
        let name = match self.peek() {
            Some(Token::Dollar(n)) => { let n = n.clone(); self.advance(); n }
            _ => return Err(SassError::Parse { expected: "$var".into(), found: "other".into() }),
        };
        self.skip_ws();
        self.expect(&Token::Colon)?;
        self.skip_ws();
        let value = self.parse_value()?;
        let flags = self.parse_var_flags()?;
        self.skip_ws();
        if self.peek() == Some(&Token::Semicolon) { self.advance(); }
        Ok(Node::Variable { name, value, flags })
    }

    fn parse_var_flags(&mut self) -> Result<VarFlags> {
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

    fn parse_body(&mut self) -> Result<Vec<Node>> {
        let mut nodes = Vec::new();
        loop {
            self.skip_ws();
            match self.peek() {
                Some(Token::RBrace) | None | Some(Token::Eof) => break,
                _ => nodes.push(self.parse_node()?),
            }
        }
        self.skip_ws();
        if self.peek() == Some(&Token::RBrace) { self.advance(); }
        Ok(nodes)
    }

    // —— @规则解析 ——
    fn parse_at_rule(&mut self, name: String) -> Result<Node> {
        self.advance(); // 消费 @rule
        match name.as_str() {
            "if" => self.parse_if(),
            "for" => self.parse_for(),
            "each" => self.parse_each(),
            "while" => self.parse_while(),
            "mixin" => self.parse_mixin_def(),
            "include" => self.parse_include(),
            "content" => { self.skip_ws(); if self.peek() == Some(&Token::Semicolon) { self.advance(); } Ok(Node::Content) }
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

    fn parse_if(&mut self) -> Result<Node> {
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
        Ok(Node::If { branches, else_body })
    }

    fn parse_for(&mut self) -> Result<Node> {
        self.skip_ws();
        let var = match self.peek() {
            Some(Token::Dollar(n)) => { let n = n.clone(); self.advance(); n }
            _ => return Err(SassError::Parse { expected: "$var".into(), found: "other".into() }),
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
        Ok(Node::For { var, from, to, inclusive, body })
    }

    fn parse_each(&mut self) -> Result<Node> {
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
            if self.peek() == Some(&Token::Comma) { self.advance(); } else { break; }
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

    fn parse_while(&mut self) -> Result<Node> {
        self.skip_ws();
        let cond = self.parse_value()?;
        self.skip_ws();
        self.expect(&Token::LBrace)?;
        let body = self.parse_body()?;
        Ok(Node::While { cond, body })
    }

    fn parse_mixin_def(&mut self) -> Result<Node> {
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

    fn parse_include(&mut self) -> Result<Node> {
        self.skip_ws();
        let name = self.parse_ident_name()?;
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
        Ok(Node::Include { name, args, content })
    }

    fn parse_function_def(&mut self) -> Result<Node> {
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

    fn parse_return(&mut self) -> Result<Node> {
        self.skip_ws();
        let value = self.parse_value()?;
        self.skip_ws();
        if self.peek() == Some(&Token::Semicolon) { self.advance(); }
        Ok(Node::Return(value))
    }

    fn parse_use(&mut self) -> Result<Node> {
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
            self.skip_ws();
            self.expect(&Token::LParen)?;
            config = self.parse_config()?;
        }
        self.skip_ws();
        if self.peek() == Some(&Token::Semicolon) { self.advance(); }
        Ok(Node::Use { url, namespace, star, config })
    }

    fn parse_forward(&mut self) -> Result<Node> {
        self.skip_ws();
        let url = self.parse_string_value()?;
        let mut show = Vec::new();
        let mut hide = Vec::new();
        let mut prefix = None;
        self.skip_ws();
        if self.peek_keyword("as") {
            self.skip_ws();
            // as prefix-*
            if let Some(Token::Ident(s)) = self.peek() {
                prefix = Some(s.clone());
                self.advance();
            }
            if self.peek() == Some(&Token::Star) { self.advance(); }
        } else if self.peek_keyword("show") {
            show = self.parse_member_list();
        } else if self.peek_keyword("hide") {
            hide = self.parse_member_list();
        }
        self.skip_ws();
        if self.peek() == Some(&Token::Semicolon) { self.advance(); }
        Ok(Node::Forward { url, show, hide, prefix })
    }

    fn parse_import(&mut self) -> Result<Node> {
        self.skip_ws();
        let url = self.parse_string_value()?;
        self.skip_ws();
        if self.peek() == Some(&Token::Semicolon) { self.advance(); }
        Ok(Node::Import { url })
    }

    fn parse_extend(&mut self) -> Result<Node> {
        self.skip_ws();
        let mut selector = String::new();
        let mut optional = false;
        while let Some(t) = self.peek() {
            match t {
                Token::Semicolon | Token::RBrace => break,
                Token::Bang => {
                    self.advance();
                    self.skip_ws();
                    if let Some(Token::Ident(s)) = self.peek() {
                        if s == "optional" { optional = true; self.advance(); }
                    }
                }
                Token::Whitespace => { selector.push(' '); self.advance(); }
                _ => { selector.push_str(&t.to_string()); self.advance(); }
            }
        }
        self.skip_ws();
        if self.peek() == Some(&Token::Semicolon) { self.advance(); }
        Ok(Node::Extend { selector: selector.trim().to_string(), optional })
    }

    fn parse_at_root(&mut self) -> Result<Node> {
        self.skip_ws();
        let query = if self.peek() == Some(&Token::LParen) {
            self.advance();
            let mut q = String::new();
            while let Some(t) = self.peek() {
                if t == &Token::RParen { break; }
                q.push_str(&t.to_string());
                self.advance();
            }
            if self.peek() == Some(&Token::RParen) { self.advance(); }
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

    fn parse_warn(&mut self) -> Result<Node> {
        self.skip_ws();
        let v = self.parse_value()?;
        self.skip_ws();
        if self.peek() == Some(&Token::Semicolon) { self.advance(); }
        Ok(Node::Warn(v))
    }
    fn parse_debug(&mut self) -> Result<Node> {
        self.skip_ws();
        let v = self.parse_value()?;
        self.skip_ws();
        if self.peek() == Some(&Token::Semicolon) { self.advance(); }
        Ok(Node::Debug(v))
    }
    fn parse_error(&mut self) -> Result<Node> {
        self.skip_ws();
        let v = self.parse_value()?;
        self.skip_ws();
        if self.peek() == Some(&Token::Semicolon) { self.advance(); }
        Ok(Node::Error(v))
    }

    fn parse_generic_at_rule(&mut self, name: String) -> Result<Node> {
        self.skip_ws();
        let params = if !matches!(self.peek(), Some(Token::LBrace) | Some(Token::Semicolon) | None) {
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
        if self.peek() == Some(&Token::Semicolon) { self.advance(); }
        Ok(Node::AtRule { name, params, body })
    }

    fn parse_at_params(&mut self) -> Result<String> {
        let mut s = String::new();
        while let Some(t) = self.peek() {
            match t {
                Token::LBrace | Token::Semicolon | Token::Eof => break,
                Token::Whitespace => { s.push(' '); self.advance(); }
                _ => { s.push_str(&t.to_string()); self.advance(); }
            }
        }
        Ok(s.trim().to_string())
    }

    // —— 参数解析 ——
    fn parse_params(&mut self) -> Result<Vec<Param>> {
        self.expect(&Token::LParen)?;
        let mut params = Vec::new();
        loop {
            self.skip_ws();
            if self.peek() == Some(&Token::RParen) { break; }
            let name = match self.peek() {
                Some(Token::Dollar(n)) => { let n = n.clone(); self.advance(); n }
                _ => return Err(SassError::Parse { expected: "$param".into(), found: "other".into() }),
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
            params.push(Param { name, default, rest });
            self.skip_ws();
            if self.peek() == Some(&Token::Comma) { self.advance(); } else { break; }
        }
        self.skip_ws();
        if self.peek() == Some(&Token::RParen) { self.advance(); }
        Ok(params)
    }

    fn parse_args(&mut self) -> Result<Vec<Arg>> {
        self.expect(&Token::LParen)?;
        let mut args = Vec::new();
        loop {
            self.skip_ws();
            if self.peek() == Some(&Token::RParen) { break; }
            // 检查关键字参数 $name: value
            // 检查关键字参数 $name: value
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
                _ => false,
            };
            let (name, value) = if is_kwarg {
                let n = match self.peek() {
                    Some(Token::Dollar(n)) => { let n = n.clone(); self.advance(); n }
                    _ => unreachable!(),
                };
                self.skip_ws();
                self.advance(); // 消费 :
                self.skip_ws();
                (Some(n), self.parse_expr(0)?)
            } else {
                (None, self.parse_expr(0)?)
            };
            let spread = if self.peek() == Some(&Token::DotDotDot) {
                self.advance();
                true
            } else {
                false
            };
            args.push(Arg { name, value, spread });
            self.skip_ws();
            if self.peek() == Some(&Token::Comma) { self.advance(); } else { break; }
        }
        self.skip_ws();
        if self.peek() == Some(&Token::RParen) { self.advance(); }
        Ok(args)
    }

    fn parse_config(&mut self) -> Result<Vec<(String, Value)>> {
        let mut config = Vec::new();
        loop {
            self.skip_ws();
            if self.peek() == Some(&Token::RParen) { break; }
            let name = match self.peek() {
                Some(Token::Dollar(n)) => { let n = n.clone(); self.advance(); n }
                _ => return Err(SassError::Parse { expected: "$var".into(), found: "other".into() }),
            };
            self.skip_ws();
            self.expect(&Token::Colon)?;
            self.skip_ws();
            let value = self.parse_value()?;
            config.push((name, value));
            self.skip_ws();
            if self.peek() == Some(&Token::Comma) { self.advance(); } else { break; }
        }
        if self.peek() == Some(&Token::RParen) { self.advance(); }
        Ok(config)
    }

    fn parse_member_list(&mut self) -> Vec<String> {
        let mut members = Vec::new();
        while let Some(t) = self.peek() {
            match t {
                Token::Semicolon | Token::LBrace => break,
                Token::Dollar(n) => { members.push(n.clone()); self.advance(); }
                Token::Ident(n) => { members.push(n.clone()); self.advance(); }
                Token::Whitespace | Token::Comma => { self.advance(); }
                _ => break,
            }
        }
        members
    }

    // —— 辅助方法 ——
    fn parse_ident_name(&mut self) -> Result<String> {
        self.skip_ws();
        match self.peek() {
            Some(Token::Ident(s)) => { let s = s.clone(); self.advance(); Ok(s) }
            _ => Err(SassError::Parse { expected: "标识符".into(), found: "other".into() }),
        }
    }

    fn parse_string_value(&mut self) -> Result<String> {
        self.skip_ws();
        match self.peek() {
            Some(Token::String(s, _)) => { let s = s.clone(); self.advance(); Ok(s) }
            Some(Token::Ident(s)) => { let s = s.clone(); self.advance(); Ok(s) }
            _ => Err(SassError::Parse { expected: "字符串".into(), found: "other".into() }),
        }
    }

    fn peek_keyword(&self, kw: &str) -> bool {
        matches!(self.peek(), Some(Token::Ident(s)) if s == kw)
    }

    fn expect_keyword(&mut self, kw: &str) -> Result<()> {
        if self.peek_keyword(kw) {
            self.advance();
            Ok(())
        } else {
            Err(SassError::Parse { expected: kw.into(), found: "other".into() })
        }
    }

    // —— Pratt 表达式解析 ——
    /// 解析值表达式（顶层，到 ; 或 } 停止）。
    pub fn parse_value(&mut self) -> Result<Value> {
        let first = self.parse_expr(0)?;
        self.skip_ws();
        // 逗号分隔列表
        if self.peek() == Some(&Token::Comma) {
            let mut items = vec![first];
            while self.peek() == Some(&Token::Comma) {
                self.advance();
                self.skip_ws();
                items.push(self.parse_expr(0)?);
                self.skip_ws();
            }
            return Ok(Value::List(items, Separator::Comma, false));
        }
        Ok(first)
    }

    fn parse_expr(&mut self, min_bp: u8) -> Result<Value> {
        self.skip_ws();
        let mut lhs = self.parse_prefix()?;
        loop {
            self.skip_ws();
            let (op, bp) = match self.peek_binding_power() {
                Some(v) => v,
                None => {
                    // 空格分隔列表——仅顶层（min_bp=0）
                    if min_bp == 0 && self.is_value_start() {
                        let mut items = vec![lhs.clone()];
                        loop {
                            self.skip_ws();
                            if !self.is_value_start() { break; }
                            if self.peek_binding_power().is_some() { break; }
                            items.push(self.parse_prefix()?);
                        }
                        if items.len() > 1 {
                            lhs = Value::List(items, Separator::Space, false);
                            continue; // 继续检查后续运算符（如 / ）
                        }
                    }
                    break;
                }
            };
            if bp < min_bp { break; }
            self.advance(); // 消费运算符
            self.skip_ws();
            let rhs = self.parse_expr(bp + 1)?;
            lhs = Value::BinOp(Box::new(BinOp { op, left: lhs, right: rhs }));
        }
        Ok(lhs)
    }

    /// 检查当前 token 是否是值起始 token（排除关键字）。
    fn is_value_start(&self) -> bool {
        if let Some(Token::Ident(s)) = self.peek() {
            if matches!(s.as_str(), "through" | "from" | "to" | "and" | "or" | "not" | "in" | "with" | "show" | "hide" | "as" | "using" | "else") {
                return false;
            }
        }
        matches!(self.peek(),
            Some(Token::Number(_)) |
            Some(Token::String(_, _)) |
            Some(Token::Ident(_)) |
            Some(Token::Hash(_)) |
            Some(Token::Dollar(_)) |
            Some(Token::Interp(_)) |
            Some(Token::LParen) |
            Some(Token::LBracket) |
            Some(Token::Minus) |
            Some(Token::Percent) |
            Some(Token::True) |
            Some(Token::False) |
            Some(Token::Null)
        )
    }

    fn parse_prefix(&mut self) -> Result<Value> {
        self.skip_ws();
        match self.peek() {
            Some(Token::Number(s)) => {
                let v = parse_number(s)?;
                self.advance();
                Ok(v)
            }
            Some(Token::String(s, q)) => {
                let v = Value::String(s.clone(), *q == '"' || *q == '\'');
                self.advance();
                Ok(v)
            }
            Some(Token::Hash(s)) => {
                let v = Value::Color(parse_hash_color(s));
                self.advance();
                Ok(v)
            }
            Some(Token::Dollar(name)) => {
                let v = Value::Variable(name.clone());
                self.advance();
                Ok(v)
            }
            Some(Token::Ident(s)) => {
                let name = s.clone();
                self.advance();
                self.skip_ws();
                // 检查 module.function() 或 module.$var 语法
                if self.peek() == Some(&Token::Dot) {
                    self.advance();
                    self.skip_ws();
                    // module.$var
                    if let Some(Token::Dollar(var_name)) = self.peek() {
                        let var_name = var_name.clone();
                        self.advance();
                        return Ok(Value::Variable(format!("{name}.{var_name}")));
                    }
                    if let Some(Token::Ident(member)) = self.peek() {
                        let member = member.clone();
                        self.advance();
                        self.skip_ws();
                        // module.function()
                        if self.peek() == Some(&Token::LParen) {
                            let args = self.parse_args()?;
                            return Ok(Value::Call(format!("{name}.{member}"), args));
                        }
                        // module.member（非调用）
                        return Ok(Value::String(format!("{name}.{member}"), false));
                    }
                }
                if self.peek() == Some(&Token::LParen) {
                    // CSS 原生函数——原样保留内容，不解析参数
                    if matches!(name.as_str(), "calc" | "clamp" | "env" | "var") {
                        self.advance(); // 消费 (
                        let mut content = String::new();
                        let mut depth = 1;
                        while let Some(t) = self.peek() {
                            match t {
                                Token::LParen => { depth += 1; content.push('('); self.advance(); }
                                Token::RParen => { depth -= 1; if depth == 0 { break; } content.push(')'); self.advance(); }
                                Token::Whitespace => { content.push(' '); self.advance(); }
                                _ => { content.push_str(&t.to_string()); self.advance(); }
                            }
                        }
                        self.skip_ws();
                        if self.peek() == Some(&Token::RParen) { self.advance(); }
                        return Ok(Value::Calc(format!("{name}({content})")));
                    }
                    let args = self.parse_args()?;
                    Ok(Value::Call(name, args))
                } else {
                    Ok(Value::String(name, false))
                }
            }
            Some(Token::Interp(s)) => {
                let v = Value::Interp(s.clone());
                self.advance();
                Ok(v)
            }
            Some(Token::True) => { self.advance(); Ok(Value::Bool(true)) }
            Some(Token::False) => { self.advance(); Ok(Value::Bool(false)) }
            Some(Token::Null) => { self.advance(); Ok(Value::Null) }
            Some(Token::LParen) => {
                self.advance();
                self.skip_ws();
                // 空 Map 或列表
                if self.peek() == Some(&Token::RParen) {
                    self.advance();
                    return Ok(Value::List(vec![], Separator::Comma, false));
                }
                let first = self.parse_expr(0)?;
                self.skip_ws();
                if self.peek() == Some(&Token::Colon) {
                    // Map
                    self.advance();
                    self.skip_ws();
                    let val = self.parse_expr(0)?;
                    let mut pairs = vec![(first, val)];
                    self.skip_ws();
                    while self.peek() == Some(&Token::Comma) {
                        self.advance();
                        self.skip_ws();
                        if self.peek() == Some(&Token::RParen) { break; } // 尾随逗号
                        let k = self.parse_expr(0)?;
                        self.skip_ws();
                        self.expect(&Token::Colon)?;
                        self.skip_ws();
                        let v = self.parse_expr(0)?;
                        pairs.push((k, v));
                        self.skip_ws();
                    }
                    self.expect(&Token::RParen)?;
                    Ok(Value::Map(pairs))
                } else {
// 分组或列表
let mut items = vec![first];
let mut saw_comma = false;
let sep = loop {
self.skip_ws();
match self.peek() {
Some(Token::Comma) => { self.advance(); saw_comma = true; self.skip_ws(); if self.peek() == Some(&Token::RParen) { break Separator::Comma; } }
Some(Token::RParen) => break if saw_comma { Separator::Comma } else { Separator::Space },
// 空格分隔的值——继续解析
Some(Token::Number(_)) | Some(Token::String(_, _)) | Some(Token::Ident(_)) | Some(Token::Hash(_)) | Some(Token::Dollar(_)) | Some(Token::Interp(_)) | Some(Token::LParen) => {
items.push(self.parse_expr(0)?);
}
_ => break if saw_comma { Separator::Comma } else { Separator::Space },
}
self.skip_ws();
if self.peek() == Some(&Token::RParen) { break if saw_comma { Separator::Comma } else { Separator::Space }; }
};
                    self.skip_ws();
                    if self.peek() == Some(&Token::RParen) { self.advance(); }
                    if items.len() == 1 && !saw_comma {
                        Ok(items.into_iter().next().unwrap())
                    } else {
                        Ok(Value::List(items, sep, false))
                    }
                }
            }
            Some(Token::Minus) => {
                self.advance();
                self.skip_ws();
                let v = self.parse_prefix()?;
                Ok(Value::UnaryOp(UnaryOp::Neg, Box::new(v)))
            }
            Some(Token::Not) => {
                self.advance();
                self.skip_ws();
                let v = self.parse_prefix()?;
                Ok(Value::UnaryOp(UnaryOp::Not, Box::new(v)))
            }
            Some(Token::LBracket) => {
                // bracketed list
                self.advance();
                let mut items = Vec::new();
                loop {
                    self.skip_ws();
                    if self.peek() == Some(&Token::RBracket) { break; }
                    items.push(self.parse_expr(0)?);
                    self.skip_ws();
                    if self.peek() == Some(&Token::Comma) { self.advance(); } else { break; }
                }
            if self.peek() == Some(&Token::RBracket) { self.advance(); }
            Ok(Value::List(items, Separator::Comma, true))
            }
            Some(Token::Amp) => {
                let v = Value::String("&".to_string(), false);
                self.advance();
                Ok(v)
            }
            Some(Token::Star) => {
                let v = Value::String("*".to_string(), false);
                self.advance();
                Ok(v)
            }
            Some(Token::Percent) => {
                // % 作为独立值 = 字符串 %
                self.advance();
                Ok(Value::String("%".to_string(), false))
            }
            _ => {
                // 尝试解析为标识符字符串——但不消费终止符
                match self.peek() {
                    Some(t) if matches!(t, Token::RBrace | Token::RParen | Token::Semicolon | Token::RBracket | Token::Comma) => {
                        Ok(Value::Null)
                    }
                    Some(t) => {
                        let v = Value::String(t.to_string(), false);
                        self.advance();
                        Ok(v)
                    }
                    None => Ok(Value::Null),
                }
            }
        }
    }

    fn peek_binding_power(&self) -> Option<(BinOpKind, u8)> {
        match self.peek() {
            Some(Token::Or) => Some((BinOpKind::Or, 1)),
            Some(Token::And) => Some((BinOpKind::And, 2)),
            Some(Token::Eq) => Some((BinOpKind::Eq, 3)),
            Some(Token::NotEq) => Some((BinOpKind::NotEq, 3)),
            Some(Token::Less) => Some((BinOpKind::Lt, 3)),
            Some(Token::Greater) => Some((BinOpKind::Gt, 3)),
            Some(Token::LessEq) => Some((BinOpKind::LtEq, 3)),
            Some(Token::GreaterEq) => Some((BinOpKind::GtEq, 3)),
            Some(Token::Plus) => Some((BinOpKind::Add, 4)),
            Some(Token::Minus) => Some((BinOpKind::Sub, 4)),
            Some(Token::Star) => Some((BinOpKind::Mul, 5)),
            Some(Token::Slash) => Some((BinOpKind::Div, 5)),
            Some(Token::Percent) => Some((BinOpKind::Mod, 5)),
            _ => None,
        }
    }
}

/// 解析数字字符串为 Value::Number。
fn parse_number(s: &str) -> Result<Value> {
    let (num_part, unit) = if let Some(idx) = s.find(|c: char| c.is_ascii_alphabetic() || c == '%') {
        (&s[..idx], Some(s[idx..].to_string()))
    } else {
        (s, None)
    };
    match num_part.parse::<f64>() {
        Ok(n) => Ok(Value::Number(n, unit)),
        Err(_) => Err(SassError::Parse { expected: "数字".into(), found: s.to_string() }),
    }
}

/// 解析 #hash 字符串为 Color。
fn parse_hash_color(s: &str) -> Color {
    let bytes = s.as_bytes();
    match bytes.len() {
        3 => Color::rgb(hex2(bytes[0], bytes[0]), hex2(bytes[1], bytes[1]), hex2(bytes[2], bytes[2])),
        4 => Color::rgba(
            hex2(bytes[1], bytes[1]), hex2(bytes[2], bytes[2]), hex2(bytes[3], bytes[3]),
            hex1(bytes[0]) as f32 / 15.0,
        ),
        6 => Color::rgb(hex2(bytes[0], bytes[1]), hex2(bytes[2], bytes[3]), hex2(bytes[4], bytes[5])),
        8 => Color::rgba(
            hex2(bytes[0], bytes[1]), hex2(bytes[2], bytes[3]), hex2(bytes[4], bytes[5]),
            hex2(bytes[6], bytes[7]) as f32 / 255.0,
        ),
        _ => Color::default(),
    }
}

fn hex2(hi: u8, lo: u8) -> u8 { (hex1(hi) << 4) | hex1(lo) }
fn hex1(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::Lexer;

    fn parse(input: &str) -> Ast {
        let tokens: Vec<_> = Lexer::new(input)
            .filter(|t| !matches!(t.as_ref(), Ok(Token::Whitespace) | Ok(Token::Eof)))
            .map(|t| t.unwrap())
            .collect();
        Parser::parse(&tokens).unwrap()
    }

    #[test]
    fn test_parse_rule() {
        let ast = parse("a { color: red; }");
        assert_eq!(ast.nodes.len(), 1);
        assert!(matches!(&ast.nodes[0], Node::Rule { selector, .. } if selector == "a"));
    }

    #[test]
    fn test_parse_variable() {
        let ast = parse("$w: 10px;");
        match &ast.nodes[0] {
            Node::Variable { name, value, .. } => {
                assert_eq!(name, "w");
                assert_eq!(*value, Value::Number(10.0, Some("px".to_string())));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_parse_if() {
        let ast = parse("@if true { a { color: red; } }");
        assert!(matches!(&ast.nodes[0], Node::If { .. }));
    }

    #[test]
    fn test_parse_mixin() {
        let ast = parse("@mixin foo($x) { color: $x; }");
        assert!(matches!(&ast.nodes[0], Node::MixinDef { name, .. } if name == "foo"));
    }

    #[test]
    fn test_parse_expr_precedence() {
        let ast = parse("$x: 1 + 2 * 3;");
        match &ast.nodes[0] {
            Node::Variable { value: Value::BinOp(b), .. } => {
                assert_eq!(b.op, BinOpKind::Add);
                assert!(matches!(&b.right, Value::BinOp(rb) if rb.op == BinOpKind::Mul));
            }
            _ => panic!("期望 BinOp"),
        }
    }
}
