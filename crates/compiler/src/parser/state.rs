//! ParserState — 语法分析内部状态

use crate::lexer::Token;
use super::ast::SassAstNode;

/// 语法分析状态，含 Token 缓冲区与游标（支持前瞻/回溯）
#[derive(Debug, Default, Clone)]
pub struct ParserState {
    /// Token 缓冲区（已消费的 Token 保留用于前瞻）
    tokens: Vec<Token>,
    /// 当前游标位置
    cursor: usize,
    /// 已完成的 AST 节点
    completed: Vec<SassAstNode>,
}

impl ParserState {
    /// 创建初始状态
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            cursor: 0,
            completed: Vec::new(),
        }
    }

    /// 推送一个 Token 到缓冲区
    pub fn push_token(&mut self, tok: Token) {
        self.tokens.push(tok);
    }

    /// 查看当前位置的 Token（不移动游标）
    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.cursor)
    }

    /// 查看相对当前位置第 n 个 Token（不移动游标）
    pub fn peek_n(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.cursor + n)
    }

    /// 消费一个 Token（移动游标）
    pub fn advance(&mut self) -> Option<&Token> {
        if self.cursor < self.tokens.len() {
            self.cursor += 1;
            self.tokens.get(self.cursor - 1)
        } else {
            None
        }
    }

    /// 设置游标位置（支持回溯）
    pub fn set_cursor(&mut self, pos: usize) {
        self.cursor = pos;
    }

    /// 获取当前游标位置
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Token 缓冲区是否耗尽
    pub fn is_eof(&self) -> bool {
        self.cursor >= self.tokens.len()
    }

    /// 推送一个完成的节点
    pub fn push_node(&mut self, node: SassAstNode) {
        self.completed.push(node);
    }

    /// 从缓冲区取出已完成的节点
    pub fn take_completed(&mut self) -> Vec<SassAstNode> {
        std::mem::take(&mut self.completed)
    }

    /// Buffer length
    pub fn tokens_len(&self) -> usize {
        self.tokens.len()
    }
}
