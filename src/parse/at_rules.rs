//! @规则解析。
//!
//! 包含 parse_at_rule/parse_if/parse_for/parse_each/parse_while 等所有 @ 规则解析方法。

use super::Parser;
use super::ast::*;
use crate::error::{Result, SassError};
use crate::lex::token::Token;
use crate::parse::at_rule_kinds::AtRuleKind;

impl<'tok> Parser<'tok> {
    // —— @规则解析 ——
    pub(crate) fn parse_at_rule(&mut self, name: String) -> Result<Node> {
        self.advance(); // 消费 @rule
        match AtRuleKind::from_str(&name) {
            AtRuleKind::If => self.parse_if(),
            AtRuleKind::For => self.parse_for(),
            AtRuleKind::Each => self.parse_each(),
            AtRuleKind::While => self.parse_while(),
            AtRuleKind::Mixin => self.parse_mixin_def(),
            AtRuleKind::Include => self.parse_include(),
            AtRuleKind::Content => {
                self.skip_ws();
                if self.peek() == Some(&Token::Semicolon) {
                    self.advance();
                }
                Ok(Node::Content)
            }
            AtRuleKind::Function => self.parse_function_def(),
            AtRuleKind::Return => self.parse_return(),
            AtRuleKind::Use => self.parse_use(),
            AtRuleKind::Forward => self.parse_forward(),
            AtRuleKind::Import => self.parse_import(),
            AtRuleKind::Extend => self.parse_extend(),
            AtRuleKind::AtRoot => self.parse_at_root(),
            AtRuleKind::Warn => self.parse_warn(),
            AtRuleKind::Debug => self.parse_debug(),
            AtRuleKind::Error => self.parse_error(),
            AtRuleKind::Other(n) => self.parse_generic_at_rule(n),
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
        // 函数名验证——基于 Sass 官方规范
        // 参考: https://sass-lang.com/documentation/breaking-changes/function-name/
        // 参考: https://sass-lang.com/documentation/breaking-changes/type-function/
        //
        // 当前实现 Phase 1 行为（匹配 sass-spec）:
        // 1. url/expression/element 全小写禁止（一直如此），大写/混合大小写允许（Phase 2 才禁止）
        // 2. and/or/not 全小写禁止（Sass 运算符关键字），大写允许
        // 3. type 任何大小写组合都禁止（CSS Values and Units 5 保留函数名）
        // 4. Vendor prefix 放宽: -prefix-url/-expression/-and/-or/-not 允许
        //    但 -prefix-element 仍然禁止（不在放宽列表中）

        // type 大小写不敏感都禁止
        if name.eq_ignore_ascii_case("type") {
            return Err(SassError::Eval(
                "This name is reserved for the plain-CSS function.".into(),
            ));
        }

        // 全小写保留名检测
        if name == name.to_ascii_lowercase() {
            match name.as_str() {
                "url" | "expression" | "element" | "and" | "or" | "not" => {
                    return Err(SassError::Eval("Invalid function name.".into()));
                }
                _ => {}
            }
        }

        // Vendor prefix 检查: -prefix-element 仍然禁止（全小写时）
        // （-prefix-url/-expression/-and/-or/-not 已放宽，不报错）
        if name == name.to_ascii_lowercase()
            && name.starts_with('-')
            && name.ends_with("-element")
        {
            return Err(SassError::Eval("Invalid function name.".into()));
        }
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
            self.advance(); // 消费 with
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
        // @forward 不能在 mixin/function/style_rule 内使用
        if self.in_body {
            return Err(SassError::Eval("This at-rule is not allowed here.".into()));
        }
        // @forward 必须在其他规则之前
        if self.saw_other_rule {
            return Err(SassError::Eval("@forward rules must be written before any other rules.".into()));
        }
        self.skip_ws();
        // @forward 的 URL 必须是字符串字面量
        let url = match self.peek() {
            Some(Token::String(s, _)) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => return Err(SassError::Eval("Expected string.".into())),
        };
        let mut show = Vec::new();
        let mut hide = Vec::new();
        let mut prefix = None;
        self.skip_ws();
        if self.peek_keyword("as") {
            self.advance();
            self.skip_ws();
            match self.peek() {
                Some(Token::Ident(s)) => {
                    prefix = Some(s.clone());
                    self.advance();
                }
                _ => return Err(SassError::Eval("Expected identifier.".into())),
            }
            self.skip_ws();
            if self.peek() == Some(&Token::Star) {
                self.advance();
            } else {
                return Err(SassError::Eval("expected \"*\".".into()));
            }
            self.skip_ws();
        }
        if self.peek_keyword("show") {
            self.advance();
            show = self.parse_member_list()?;
            if show.is_empty() {
                return Err(SassError::Eval("Expected variable, mixin, or function name".into()));
            }
            self.skip_ws();
            if self.peek_keyword("hide") {
                return Err(SassError::Eval("expected \";\".".into()));
            }
        } else if self.peek_keyword("hide") {
            self.advance();
            hide = self.parse_member_list()?;
            if hide.is_empty() {
                return Err(SassError::Eval("Expected variable, mixin, or function name".into()));
            }
            self.skip_ws();
            if self.peek_keyword("show") {
                return Err(SassError::Eval("expected \";\".".into()));
            }
        }
        self.skip_ws();
        let mut config = Vec::new();
        if self.peek_keyword("with") {
            self.advance();
            self.skip_ws();
            self.expect(&Token::LParen)?;
            config = self.parse_config()?;
            self.skip_ws();
            if self.peek_keyword("as") || self.peek_keyword("show") || self.peek_keyword("hide") {
                return Err(SassError::Eval("expected \";\".".into()));
            }
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
            config,
        })
    }

    pub(crate) fn parse_import(&mut self) -> Result<Node> {
        // 支持 @import "a", "b", "c" 多值语法
        // 也支持 @import "url" modifier（CSS media/supports 修饰符）
        let mut urls = Vec::new();
        loop {
            self.skip_ws();
            let url = self.parse_string_value()?;
            urls.push(url);
            self.skip_ws();
            if self.peek() == Some(&Token::Comma) {
                self.advance();
                continue;
            }
            break;
        }
        // 捕获 URL 后面的修饰符（CSS @import media query 等，到分号为止）
        self.skip_ws();
        let modifier = if !matches!(self.peek(), Some(Token::Semicolon) | Some(Token::RBrace) | None) {
            let mut s = String::new();
            while let Some(t) = self.peek() {
                match t {
                    Token::Semicolon | Token::LBrace | Token::RBrace | Token::Eof => break,
                    Token::Comment(_, _) => { self.advance(); }
                    Token::Whitespace => { s.push(' '); self.advance(); }
                    _ => { s.push_str(&t.to_string()); self.advance(); }
                }
            }
            s.trim().to_string()
        } else {
            String::new()
        };
        self.skip_ws();
        if self.peek() == Some(&Token::Semicolon) {
            self.advance();
        }
        // 多值 @import：拼接为多行 CSS @import 透传
        // sasspile 的 parse_node 只返回一个 Node，多值通过拼接 URL 处理
        let url = if urls.len() == 1 {
            urls.into_iter().next().unwrap()
        } else {
            // 多值 CSS @import 拼接为 "a", "b" 格式
            urls.iter().map(|u| format!("\"{u}\"")).collect::<Vec<_>>().join("\", \"")
        };
        Ok(Node::Import { url, modifier })
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
                Token::Comment(_, _) => {
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
                    s.push(' ');
                    self.advance();
                }
                _ => {
                    s.push_str(&t.to_string());
                    self.advance();
                }
            }
        }
        // 标准化冒号周围空白
        let s = s.replace(" : ", ": ").replace(":  ", ": ");
        Ok(s.trim().to_string())
    }
}
