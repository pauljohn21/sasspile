//! Parse Stage — Token stream → Node stream
//!
//! AstBuilder 用 enum 状态机,reducer 委托纯 parse_transition 函数.

use crate::ast::{Node, Token};

mod declarations;
mod directives;

// ─── 解析状态机 ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseMode {
    Top,
    InSelector,
    InBody,
    InProperty,
    InValue,
}

#[derive(Debug, Clone)]
pub struct AstBuilder {
    mode: ParseMode,
    selector: Option<String>,
    property: Option<String>,
    value_buf: Vec<Token>,
    _pending_body: Vec<Node>,
}

// ─── 纯辅助函数 ───────────────────────────────────────────────────────────

fn tokens_to_string(tokens: &[Token]) -> String {
    tokens
        .iter()
        .filter_map(|t| match t {
            Token::Ident(s) | Token::Number(s) => Some(s.as_str()),
            Token::String(s) => Some(s.as_str()),
            Token::Interpolation(s) => return Some(s.as_str()),
            Token::Op(o) => Some(o.as_str()),
            Token::Dollar => Some("$"),
            Token::Hash => Some("#"),
            Token::Colon => Some(":"),
            Token::Semicolon => Some(";"),
            Token::Comma => Some(", "),
            Token::Dot => Some("."),
            Token::LParen => Some("("),
            Token::RParen => Some(")"),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// ─── 纯状态转移函数（核心 match）────────────────────────────────────────

/// (mode, selector, property, value_buf, tok) → (new_mode, new_selector, new_property, new_value_buf, emitted)
fn parse_transition(
    mode: ParseMode,
    sel: Option<String>,
    prop: Option<String>,
    value_buf: Vec<Token>,
    tok: Token,
) -> (ParseMode, Option<String>, Option<String>, Vec<Token>, Vec<Node>) {
    use ParseMode::*;
    use Token::*;

    match (mode, tok) {
        (Top, Ident(s)) => (InSelector, Some(s), None, Vec::new(), Vec::new()),
        (Top, Dollar) => (InProperty, None, None, Vec::new(), Vec::new()),
        (Top, At) => (InSelector, None, None, Vec::new(), Vec::new()),
        (Top, Whitespace) => (Top, None, None, Vec::new(), Vec::new()),
        (Top, Newline) => (Top, None, None, Vec::new(), Vec::new()),
        (Top, _) => (Top, None, None, Vec::new(), Vec::new()),

        (InSelector, Ident(seg)) => {
            let merged = merge_segment(sel, &seg, "");
            (InSelector, Some(merged), None, Vec::new(), Vec::new())
        }
        (InSelector, Whitespace) => {
            let merged = merge_segment(sel, " ", "");
            (InSelector, Some(merged), None, Vec::new(), Vec::new())
        }
        (InSelector, Dot) => {
            let merged = merge_segment(sel, ".", "");
            (InSelector, Some(merged), None, Vec::new(), Vec::new())
        }
        (InSelector, Colon) => {
            let merged = merge_segment(sel, ":", "");
            (InSelector, Some(merged), None, Vec::new(), Vec::new())
        }
        (InSelector, Hash) => {
            let merged = merge_segment(sel, "#", "");
            (InSelector, Some(merged), None, Vec::new(), Vec::new())
        }
        (InSelector, LBrace) => (InBody, sel, None, Vec::new(), Vec::new()),
        (InSelector, Newline) => (InSelector, sel, None, Vec::new(), Vec::new()),
        (InSelector, _) => (InSelector, sel, None, Vec::new(), Vec::new()),

        (InBody, Ident(s)) => (InProperty, sel, Some(s), Vec::new(), Vec::new()),
        (InBody, Dollar) => (InProperty, sel, None, Vec::new(), Vec::new()),
        (InBody, At) => (InProperty, sel, None, Vec::new(), Vec::new()),
        (InBody, RBrace) => {
            let rule = make_rule(sel.unwrap_or_default(), Vec::new());
            (Top, None, None, Vec::new(), vec![rule])
        }
        (InBody, Newline) => (InBody, sel, None, Vec::new(), Vec::new()),
        (InBody, Whitespace) => (InBody, sel, None, Vec::new(), Vec::new()),
        (InBody, _) => (InBody, sel, prop, Vec::new(), Vec::new()),

        (InProperty, Colon) => (InValue, sel, prop, Vec::new(), Vec::new()),
        (InProperty, Ident(s)) => {
            let merged_prop = prop.map(|p| format!("{p}-{s}")).or(Some(s));
            (InProperty, sel, merged_prop, Vec::new(), Vec::new())
        }
        (InProperty, Newline) => (InProperty, sel, prop, Vec::new(), Vec::new()),
        (InProperty, Whitespace) => (InProperty, sel, prop, Vec::new(), Vec::new()),
        (InProperty, _) => (InProperty, sel, prop, Vec::new(), Vec::new()),

        (InValue, Semicolon) => {
            let decl = make_decl(prop.unwrap_or_default(), &value_buf);
            (InBody, sel, None, Vec::new(), vec![decl])
        }
        (InValue, Newline) => {
            let decl = make_decl(prop.unwrap_or_default(), &value_buf);
            (InBody, sel, None, Vec::new(), vec![decl])
        }
        (InValue, RBrace) => {
            let decl = make_decl(prop.unwrap_or_default(), &value_buf);
            let rule = make_rule(sel.unwrap_or_default(), vec![decl.clone()]);
            (Top, None, None, Vec::new(), vec![decl, rule])
            // Note: simplified — normally body accumulates decls
        }
        (InValue, Comma) => {
            let mut buf = value_buf;
            buf.push(Comma);
            (InValue, sel, prop, buf, Vec::new())
        }
        (InValue, other) => {
            let mut buf = value_buf;
            buf.push(other);
            (InValue, sel, prop, buf, Vec::new())
        }
    }
}

// ─── 纯构造器（return owned value，无 push 副作用）────────────────────────

fn merge_segment(sel: Option<String>, segment: &str, _sep: &str) -> String {
    match sel {
        Some(s) => format!("{s}{segment}"),
        None => segment.to_string(),
    }
}

fn make_rule(selector: String, body: Vec<Node>) -> Node {
    Node::Rule {
        selector: selector.trim().to_string(),
        body,
    }
}

fn make_decl(prop: String, value_buf: &[Token]) -> Node {
    Node::Declaration {
        prop,
        value: tokens_to_string(value_buf),
    }
}

// ─── AstBuilder API ────────────────────────────────────────────────────────

impl AstBuilder {
    pub fn new() -> Self {
        Self {
            mode: ParseMode::Top,
            selector: None,
            property: None,
            value_buf: Vec::new(),
            _pending_body: Vec::new(),
        }
    }

    pub fn feed(&mut self, token: Token) -> Vec<Node> {
        let (mode, sel, prop, value, emitted) = parse_transition(
            self.mode,
            self.selector.take(),
            self.property.take(),
            std::mem::take(&mut self.value_buf),
            token,
        );
        self.mode = mode;
        self.selector = sel;
        self.property = prop;
        self.value_buf = value;
        emitted
    }

    pub fn finalize(self) -> Vec<Node> {
        match self.mode {
            ParseMode::InValue => vec![make_decl(self.property.unwrap_or_default(), &self.value_buf)],
            ParseMode::InBody => vec![make_rule(self.selector.unwrap_or_default(), Vec::new())],
            _ => Vec::new(),
        }
    }
}

impl Default for AstBuilder {
    fn default() -> Self {
        Self::new()
    }
}
