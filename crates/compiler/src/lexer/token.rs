//! Token 定义

/// Token 类型 — 区分标识符、数字、字符串等
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    /// 标识符
    Ident,
    /// 数字
    Number,
    /// 字符串
    String,
    /// 变量 ($xxx)
    Variable,
    /// 插值开始
    Interpolation,
    /// 父选择器
    ParentSelector,
    /// 普通字符
    Char(char),
}

/// 词法单元
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// Token 类型
    pub kind: TokenKind,
    /// 源码字节偏移
    pub pos: u32,
    /// 原始文本
    pub text: String,
    /// 行号
    pub line: u32,
    /// 列号
    pub col: u32,
}

impl Token {
    /// Create a new token
    pub fn new(kind: TokenKind, pos: u32, text: impl Into<String>) -> Self {
        Self {
            kind,
            pos,
            text: text.into(),
            line: 0,
            col: 0,
        }
    }

    /// Create with line/col info
    pub fn with_pos(kind: TokenKind, pos: u32, text: impl Into<String>, line: u32, col: u32) -> Self {
        Self {
            kind,
            pos,
            text: text.into(),
            line,
            col,
        }
    }

    /// Returns the char representation for Char tokens
    pub fn char_kind(&self) -> Option<char> {
        match &self.kind {
            TokenKind::Char(c) => Some(*c),
            _ => None,
        }
    }
}
