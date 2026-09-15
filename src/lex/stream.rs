//! 流式 Lexer —— 词素发射
//!
//! 核心：`feed: (LexerState, char) -> (LexerState, Vec<Token>)` 是纯函数。
//! scan 累加器 = LexerState，每步 emit 0..N 个 token。
//!
//! 使用 scan + flat_map 模式：
//!   scan 维护 LexerState，返回产出 token 的迭代器
//!   flat_map 将每次产出的 token 展开到输出流

use crate::lex::token::Token;

/// Lexer 累积状态（不可变数据，每次创建新实例）
#[derive(Debug, Clone)]
pub struct LexerState {
    buf: String,
    mode: LexMode,
}

#[derive(Debug, Clone)]
enum LexMode {
    Idle,
    InIdent,
    InNumber { has_dot: bool },
    InString { quote: char },
}

impl Default for LexerState {
    fn default() -> Self {
        Self { buf: String::new(), mode: LexMode::Idle }
    }
}

/// 纯函数：`(LexerState, char) -> (LexerState, Vec<Token>)`
///
/// scan 算子签名: `Fn(acc: S, item: I) -> S`
/// 但 scan 只能 emit 一个 S。为了 emit 多个 token，
/// 我们让 scan 返回 Vec<Token>，然后用 flat_map 展开。
pub fn feed(state: LexerState, ch: char) -> (LexerState, Vec<Token>) {
    match state.mode {
        LexMode::Idle => feed_idle(ch),
        LexMode::InIdent => feed_ident(state.buf, ch),
        LexMode::InNumber { has_dot } => feed_number(state.buf, ch, has_dot),
        LexMode::InString { quote } => feed_string(state.buf, ch, quote),
    }
}

fn feed_idle(ch: char) -> (LexerState, Vec<Token>) {
    match ch {
        ' ' | '\t' | '\n' | '\r' => (LexerState::default(), vec![Token::Whitespace]),
        'a'..='z' | 'A'..='Z' | '_' | '-' => (LexerState::new_ident(ch), vec![]),
        '0'..='9' => (LexerState::new_number(ch), vec![]),
        '"' | '\'' => (LexerState::new_string(ch), vec![]),
        '{' => (LexerState::default(), vec![Token::LBrace]),
        '}' => (LexerState::default(), vec![Token::RBrace]),
        '(' => (LexerState::default(), vec![Token::LParen]),
        ')' => (LexerState::default(), vec![Token::RParen]),
        '[' => (LexerState::default(), vec![Token::LBracket]),
        ']' => (LexerState::default(), vec![Token::RBracket]),
        ':' => (LexerState::default(), vec![Token::Colon]),
        ';' => (LexerState::default(), vec![Token::Semicolon]),
        ',' => (LexerState::default(), vec![Token::Comma]),
        '+' => (LexerState::default(), vec![Token::Plus]),
        '*' => (LexerState::default(), vec![Token::Star]),
        '%' => (LexerState::default(), vec![Token::Percent]),
        '&' => (LexerState::default(), vec![Token::Amp]),
        _ => (LexerState::default(), vec![]),
    }
}

fn feed_ident(buf: String, ch: char) -> (LexerState, Vec<Token>) {
    match ch {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' => {
            let mut new_buf = buf;
            new_buf.push(ch);
            (LexerState { buf: new_buf, mode: LexMode::InIdent }, vec![])
        }
        _ => {
            let token = match buf.as_str() {
                "true" => Token::True,
                "false" => Token::False,
                "null" => Token::Null,
                other => Token::Ident(other.to_string()),
            };
            let (new_state, mut extra) = feed_idle(ch);
            let mut result = vec![token];
            result.append(&mut extra);
            (new_state, result)
        }
    }
}

fn feed_number(buf: String, ch: char, has_dot: bool) -> (LexerState, Vec<Token>) {
    match ch {
        '0'..='9' => {
            let mut new_buf = buf;
            new_buf.push(ch);
            (LexerState { buf: new_buf, mode: LexMode::InNumber { has_dot } }, vec![])
        }
        '.' if !has_dot => {
            let mut new_buf = buf;
            new_buf.push('.');
            (LexerState { buf: new_buf, mode: LexMode::InNumber { has_dot: true } }, vec![])
        }
        _ => {
            let token = Token::Number(buf);
            let (new_state, mut extra) = feed_idle(ch);
            let mut result = vec![token];
            result.append(&mut extra);
            (new_state, result)
        }
    }
}

fn feed_string(buf: String, ch: char, quote: char) -> (LexerState, Vec<Token>) {
    if ch == quote {
        (LexerState::default(), vec![Token::String(buf, quote)])
    } else {
        let mut new_buf = buf;
        new_buf.push(ch);
        (LexerState { buf: new_buf, mode: LexMode::InString { quote } }, vec![])
    }
}

impl LexerState {
    fn new_ident(ch: char) -> Self {
        Self { buf: ch.to_string(), mode: LexMode::InIdent }
    }
    fn new_number(ch: char) -> Self {
        Self { buf: ch.to_string(), mode: LexMode::InNumber { has_dot: false } }
    }
    fn new_string(quote: char) -> Self {
        Self { buf: String::new(), mode: LexMode::InString { quote } }
    }
}
