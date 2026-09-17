//! Tokenize Stage — char stream → Token stream
//!
//! 使用 scan_map(Scanner) 累积状态. reducer 委托纯 transition 函数,
//! 不内嵌 if-else 链.push 累积委托 classify_buffer 纯函数返回值组合.
//!
//! 状态机: ScanMode enum 替代 bool flags → 状态空间合法且可穷尽匹配.

use crate::ast::Token;

// ─── 状态机 ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Scanner {
    buf: String,
    mode: ScanMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanMode {
    Normal,
    InString { delim: char },
    MaybeComment, // buf == "/" 等待下一个字符判断行/块注释
    InLineComment,
    InBlockComment,
}

// ─── 纯辅助函数 ───────────────────────────────────────────────────────────

/// 分类 buffer 内容为 Token(s) — 纯函数,无副作用
fn classify_buffer(buf: &str) -> Vec<Token> {
    if buf.is_empty() {
        return vec![];
    }
    match buf {
        "{" => vec![Token::LBrace],
        "}" => vec![Token::RBrace],
        "(" => vec![Token::LParen],
        ")" => vec![Token::RParen],
        "[" => vec![Token::LBracket],
        "]" => vec![Token::RBracket],
        ":" => vec![Token::Colon],
        ";" => vec![Token::Semicolon],
        "," => vec![Token::Comma],
        "." => vec![Token::Dot],
        "$" => vec![Token::Dollar],
        "#" => vec![Token::Hash],
        "@" => vec![Token::At],
        _ => {
            let s = buf;
            if let Some(rest) = s.strip_prefix('@') {
                if rest.is_empty() {
                    vec![Token::At]
                } else {
                    vec![Token::At, Token::Ident(rest.into())]
                }
            } else if let Some(rest) = s.strip_prefix('$') {
                if rest.is_empty() {
                    vec![Token::Dollar]
                } else {
                    vec![Token::Dollar, Token::Ident(rest.into())]
                }
            } else if let Some(rest) = s.strip_prefix("//") {
                vec![Token::Comment(rest.into())]
            } else if is_numeric(s) {
                vec![Token::Number(s.into())]
            } else {
                vec![Token::Ident(s.into())]
            }
        }
    }
}

fn classify_single_char(ch: char) -> Vec<Token> {
    match ch {
        '{' => vec![Token::LBrace],
        '}' => vec![Token::RBrace],
        '(' => vec![Token::LParen],
        ')' => vec![Token::RParen],
        '[' => vec![Token::LBracket],
        ']' => vec![Token::RBracket],
        ':' => vec![Token::Colon],
        ';' => vec![Token::Semicolon],
        ',' => vec![Token::Comma],
        '.' => vec![Token::Dot],
        '$' => vec![Token::Dollar],
        '#' => vec![Token::Hash],
        '@' => vec![Token::At],
        _ => vec![],
    }
}

fn is_boundary(ch: char) -> bool {
    matches!(
        ch,
        '{' | '}' | '(' | ')' | '[' | ']' | ':' | ';' | ',' | '.' | '$' | '#' | '@'
    )
}

fn is_numeric(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-')
}

// ─── 纯状态转移函数 ─────────────────────────────────────────────────────────

/// (buf, mode, ch) → (new_mode, new_buf, emitted_tokens)
/// 纯函数: 无 &mut,无副作用; match 穷尽状态转移.
fn transition(buf: String, mode: ScanMode, ch: char) -> (ScanMode, String, Vec<Token>) {
    use ScanMode::*;

    match mode {
        InBlockComment if ch == '*' && buf.ends_with('/') => {
            // "*/" 结束块注释 (此时 buf 含 "/")
            (Normal, String::new(), vec![Token::Comment(String::new())])
        }
        InBlockComment if ch == '*' => {
            // 可能的 "*" 后面跟 "/": 保留 "*" 在 buffer 里
            let mut new_buf = buf;
            new_buf.push('*');
            (InBlockComment, new_buf, vec![])
        }
        InBlockComment => {
            if buf.len() == 1 && buf == "*" {
                // 上一个是孤立的 "*",不是 "*/" 的一部分 → flush "*" 进 comment
                let mut new_buf = String::new();
                new_buf.push(ch);
                // 注意: 此时不完整,我们用 "/" "*" 之后字符的方式重写
                (InLineComment, new_buf, vec![])

            } else {
                let mut new_buf = buf;
                new_buf.push(ch);
                (InBlockComment, new_buf, vec![])
            }
        }

        InLineComment if ch == '\n' => {
            // 行注释结束 — 去掉末尾积累的,emit Comment + Newline
            (Normal, String::new(), vec![Token::Comment(buf), Token::Newline])
        }

        InLineComment => {
            // 继续累积注释内容
            let mut new_buf = buf;
            new_buf.push(ch);
            (InLineComment, new_buf, vec![])
        }

        InString { delim } if ch == delim => {
            // 结束字符串: 加上 delimiters
            let s = buf + &ch.to_string();
            (Normal, String::new(), vec![Token::String(s)])
        }

        InString { delim } => {
            let mut new_buf = buf;
            new_buf.push(ch);
            (InString { delim }, new_buf, vec![])
        }

        MaybeComment => match ch {
            '/' => {
                // 确认为行注释 — buf 中的 "/" 属于注释,丢弃
                (InLineComment, String::new(), vec![])
            }
            '*' => {
                // 确认为块注释开始 — 丢弃 "/" 和 "*"
                (InBlockComment, String::new(), vec![])
            }
            _ => {
                // Not a comment — flush "/" as operator then reprocess ch in Normal
                let mut emitted = classify_buffer("/");
                let (_, new_buf, more) = transition(String::new(), Normal, ch);
                emitted.extend(more);
                (Normal, new_buf, emitted)
            }
        },

        Normal => match ch {
            c @ '\'' | c @ '"' => {
                // flush buffer + 进入 string mode
                let emitted = classify_buffer(&buf);
                (InString { delim: c }, c.to_string(), emitted)
            }

            c if c.is_whitespace() => {
                let mut emitted = classify_buffer(&buf);
                if c == '\n' {
                    emitted.push(Token::Newline);
                } else {
                    emitted.push(Token::Whitespace);
                }
                (Normal, String::new(), emitted)
            }

            '/' if buf.is_empty() => {
                // Might be comment — hold in buf
                (MaybeComment, "/".to_string(), vec![])
            }

            c if is_boundary(c) && !buf.is_empty() => {
                // flush current buffer + emit boundary as token
                let mut emitted = classify_buffer(&buf);
                emitted.extend(classify_single_char(c));
                (Normal, String::new(), emitted)
            }

            c if is_boundary(c) => {
                // buffer empty — emit boundary directly
                (Normal, String::new(), classify_single_char(c))
            }

            c => {
                // accumulate normal char
                let mut new_buf = buf;
                new_buf.push(c);
                (Normal, new_buf, vec![])
            }
        },
    }
}

// ─── Scanner API（scan_map 适配层）────────────────────────────────────────

impl Scanner {
    pub fn new() -> Self {
        Self {
            buf: String::new(),
            mode: ScanMode::Normal,
        }
    }

    /// scan_map reducer: &mut Scanner, char → Vec<Token>
    /// 内部委托纯 transition,不内嵌 if-else 链.
    pub fn feed(&mut self, ch: char) -> Vec<Token> {
        // mem::take: 把 old String 移出(留下空 String 给 &mut self.buf)
        let buf = std::mem::take(&mut self.buf);
        let (new_mode, new_buf, emitted) = transition(buf, self.mode, ch);
        self.mode = new_mode;
        self.buf = new_buf;
        emitted
    }

    /// Finalize — flush 剩余 buffer（各模式直接 return 对应分类结果）
    pub fn finalize(self) -> Vec<Token> {
        match self.mode {
            ScanMode::InLineComment | ScanMode::InBlockComment => {
                vec![Token::Comment(self.buf)]
            }
            ScanMode::MaybeComment => classify_buffer("/"),
            _ => classify_buffer(&self.buf),
        }
    }
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new()
    }
}
