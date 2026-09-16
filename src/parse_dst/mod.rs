//! Parse Stage Helpers
//!
//! 提供 AstBuilder 和 feed 函数,供 parse 阶段使用。
//! 实际算子链在 pipeline.rs 中通过 scan_map + flat_map 组装。

use tracing;

use crate::ast::{Node, Token};
use crate::tokenize_dst::ScannerState;

mod declarations;
mod directives;

pub use declarations::{parse_block, parse_declarations};

/// 独立解析源字符串为 Node 向量 — 用于 import 时抽取子模块顶层定义 (MixinDef/Variable)
pub fn parse_source(input: &str) -> Vec<Node> {
    let mut scanner = ScannerState::new();
    let mut builder = AstBuilder::new();
    let mut out = Vec::new();
    for ch in input.chars() {
        let toks = scanner.feed(ch);
        for tok in toks {
            out.extend(builder.feed(tok));
        }
    }
    out.extend(builder.flush());
    out
}

/// AST 构建器状态 — 累积 Token 直到可产出完整 Node
#[derive(Debug, Clone)]
pub struct AstBuilder {
    /// Token 缓冲区 (顶层 / selector 累积)
    pub(super) buffer: Vec<Token>,
    /// 嵌套深度 (由 { } 决定)
    pub(super) depth: u32,
    /// 当前规则选择器 (depth > 0 时有效)
    selector: Option<String>,
}

impl AstBuilder {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            depth: 0,
            selector: None,
        }
    }

    /// flush 剩余 buffered token (解析结束时调用)
    pub fn flush(&mut self) -> Vec<Node> {
        if self.buffer.is_empty() {
            return Vec::new();
        }
        // 若有残留 selector (未闭合的 rule),尝试构建 Rule
        if let Some(selector) = self.selector.take() {
            let body_tokens: Vec<Token> = self.buffer.drain(..).collect();
            let body = parse_rule_body(&body_tokens);
            return vec![Node::Rule { selector, body }];
        }
        let nodes = declarations::flush_rule_buffer(&mut self.buffer);
        if nodes.is_empty() {
            declarations::try_flush_variable(&mut self.buffer).map_or(Vec::new(), |n| vec![n])
        } else {
            nodes
        }
    }

    /// feed 一个 token — 返回已解析出的 Node 们 (可能是 0 或多个)
    pub fn feed(&mut self, token: Token) -> Vec<Node> {
        let _span = tracing::info_span!("parse.feed", ?token, depth = self.depth, has_selector = self.selector.is_some()).entered();
        let prev_depth = self.depth;

        // 更新深度
        match &token {
            Token::LBrace => self.depth += 1,
            Token::RBrace => {
                self.depth = self.depth.saturating_sub(1);
            }
            _ => {}
        }
        self.buffer.push(token.clone());

        let mut out = Vec::new();

        // === Rule 闭合: depth 从 1 回到 0 ===
        if prev_depth == 1 && self.depth == 0 {
            if let Some(selector) = self.selector.take() {
                let body_tokens: Vec<Token> = self.buffer.drain(..).collect();
                let body = parse_rule_body(&body_tokens);
                tracing::debug!(selector, body_len = body.len(), "rule closed");
                out.push(Node::Rule { selector, body });
                return out;
            }
            // selector 为 None → 之前是 @mixin/@if 等指令块
            if let Some(node) = directives::try_flush_directive(&mut self.buffer) {
                tracing::debug!(node = ?node, "directive block closed");
                out.push(node);
            }
            return out;
        }

        // === Rule 入口 / 指令块入口: depth 从 0 到 1 ===
        if prev_depth == 0 && self.depth == 1 && self.selector.is_none() {
            // 判断: buffer 含 At → 指令块 (@mixin/@if);否则是 Rule
            if self.buffer.iter().any(|t| *t == Token::At) {
                // 指令块: 尚未闭合,等待 RBrace
                return out;
            }
            // Rule: 提取 selector
            let selector = self
                .buffer
                .iter()
                .filter_map(|t| match t {
                    Token::Ident(s) => Some(s.as_str()),
                    Token::Dot => Some("."),
                    Token::Hash => Some("#"),
                    Token::Colon => Some(":"),
                    Token::String(s) => Some(s.as_str()),
                    Token::Interpolation(s) => Some(s.as_str()),
                    Token::Dollar => Some("$"),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("");
            self.selector = Some(selector);
            self.buffer.clear();
            return out;
        }

        // === depth 0 顶层: 尝试指令 / 变量 ===
        if self.depth == 0 && self.selector.is_none() {
            if self.buffer.iter().any(|t| *t == Token::At) {
                if let Some(node) = directives::try_flush_directive(&mut self.buffer) {
                    out.push(node);
                }
            }
            if let Some(node) = declarations::try_flush_variable(&mut self.buffer) {
                out.push(node);
            }
        }

        out
    }
}

impl Default for AstBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 解析 Rule body tokens: 混合了声明 (prop:value;) 和内联指令 (@include name;)
fn parse_rule_body(tokens: &[Token]) -> Vec<Node> {
    let mut out = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        // 跳过空白
        while i < tokens.len() && matches!(tokens[i], Token::Whitespace | Token::Newline) {
            i += 1;
        }
        if i >= tokens.len() {
            break;
        }

        if tokens[i] == Token::At {
            // 内联指令: At Ident(args) ... Semicolon
            let start = i;
            let instr = match tokens.get(i + 1) {
                Some(Token::Ident(s)) => s.clone(),
                _ => {
                    i += 1;
                    continue;
                }
            };
            // 找到终止符 (Semicolon 或 LBrace 开始块)
            let mut end = i + 2;
            if let Some(off) = tokens[end..]
                .iter()
                .position(|t| matches!(t, Token::Semicolon | Token::Newline))
            {
                end += off + 1;
            } else {
                end = tokens.len();
            }
            let directive_tokens = &tokens[start..end];
            if let Some(node) = directives::build_inline_node(directive_tokens, &instr) {
                out.push(node);
            }
            i = end;
        } else {
            // 声明块: 找到 Semicolon / RBrace 为止
            let start = i;
            while i < tokens.len()
                && !matches!(
                    tokens[i],
                    Token::Semicolon | Token::Newline | Token::RBrace
                )
            {
                i += 1;
            }
            let decl_tokens = &tokens[start..i];
            let decls = parse_declarations(decl_tokens);
            out.extend(decls);
            if i < tokens.len() {
                i += 1; // 跳过 Semicolon / Newline / RBrace
            }
        }
    }

    out
}
