//! Tokenize Stage Helpers
//!
//! 为 tokenize 阶段提供 ScannerState 和 feed 函数
//! 实际算子链在 pipeline.rs 中通过 scan + flat_map + filter 组装

use crate::ast::Token;

/// 扫描器状态 — 在 scan 闭包之间累积
#[derive(Debug, Clone)]
pub struct ScannerState {
    /// 当前正在累积的 identifier 缓冲区
    ident_buf: String,
    /// 当前正在累积的数字缓冲区
    num_buf: String,
    /// 是否处于字符串上下文
    in_string: bool,
    /// 字符串界定符
    string_delim: char,
    /// 是否处于注释上下文
    in_comment: bool,
    /// 字符串累积区
    string_buf: String,
    /// 是否处于插值上下文
    in_interpolation: bool,
    /// 插值累积区
    interp_buf: String,
    /// 上一个 char 是否为 '#' (用于识别 '#{' 进入插值)
    last_was_hash: bool,
}

impl ScannerState {
    pub fn new() -> Self {
        Self {
            ident_buf: String::new(),
            num_buf: String::new(),
            in_string: false,
            string_delim: '"',
            in_comment: false,
            string_buf: String::new(),
            in_interpolation: false,
            interp_buf: String::new(),
            last_was_hash: false,
        }
    }

    /// 核心扫描: 输入 char, 产出 0..N 个 Token
    ///
    /// 此函数可直接作为 scan 的 reducer 函数:
    /// `.scan_map(ScannerState::new(), |state, ch| state.feed(ch))`
    pub fn feed(&mut self, ch: char) -> Vec<Token> {
        let mut out = Vec::new();

        if self.in_interpolation {
            if ch == '}' {
                self.in_interpolation = false;
                let content = self.interp_buf.clone();
                self.interp_buf.clear();
                out.push(Token::Interpolation(content));
            } else {
                self.interp_buf.push(ch);
            }
            self.last_was_hash = false;
            return out;
        }

        if self.in_string {
            if ch == self.string_delim {
                self.in_string = false;
                let content = self.string_buf.clone();
                self.string_buf.clear();
                out.push(Token::String(content));
            } else {
                self.string_buf.push(ch);
            }
            self.last_was_hash = false;
            return out;
        }

        if ch == '#' {
            if let Some(tok) = self.flush_ident() {
                out.push(tok);
            }
            self.last_was_hash = true;
            return out;
        }

        // 插值: Hash 紧跟 '{' 进入插值模式
        if ch == '{' && self.last_was_hash {
            self.last_was_hash = false;
            self.in_interpolation = true;
            return out;
        }

        if ch == '/' {
            self.in_comment = true;
            self.last_was_hash = false;
            return out;
        }

        if ch == '"' || ch == '\'' {
            self.in_string = true;
            self.string_delim = ch;
            if let Some(tok) = self.flush_ident() {
                out.push(tok);
            }
            out.push(Token::Char(ch));
            self.last_was_hash = false;
            return out;
        }

        if ch.is_alphabetic() || ch == '-' || ch == '_' {
            self.ident_buf.push(ch);
            self.last_was_hash = false;
            return out;
        }

        // 已累积 identifier 或数字,先 flushed
        if let Some(tok) = self.flush_ident() {
            out.push(tok);
        }
        if let Some(tok) = self.flush_num() {
            out.push(tok);
        }
        self.last_was_hash = false;

        // 数字累积
        if ch.is_ascii_digit() {
            self.num_buf.push(ch);
            return out;
        }

        match ch {
            c if c.is_whitespace() => {
                if c == '\n' {
                    out.push(Token::Newline);
                } else {
                    out.push(Token::Whitespace);
                }
            }
            '$' => out.push(Token::Dollar),
            '.' => out.push(Token::Dot),
            ':' => out.push(Token::Colon),
            ';' => out.push(Token::Semicolon),
            ',' => out.push(Token::Comma),
            '(' => out.push(Token::LParen),
            ')' => out.push(Token::RParen),
            '{' => out.push(Token::LBrace),
            '}' => out.push(Token::RBrace),
            '[' => out.push(Token::LBracket),
            ']' => out.push(Token::RBracket),
            '@' => out.push(Token::At),
            c => out.push(Token::Char(c)),
        }

        out
    }

    fn flush_ident(&mut self) -> Option<Token> {
        if self.ident_buf.is_empty() {
            None
        } else {
            let ident = self.ident_buf.clone();
            self.ident_buf.clear();
            Some(Token::Ident(ident))
        }
    }

    fn flush_num(&mut self) -> Option<Token> {
        if self.num_buf.is_empty() {
            None
        } else {
            let num = self.num_buf.clone();
            self.num_buf.clear();
            Some(Token::Number(num))
        }
    }
}

impl Default for ScannerState {
    fn default() -> Self {
        Self::new()
    }
}
