//! Token 定义——词法分析器的产出。

/// 词法单元。
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // —— 字面量 ——
    /// 标识符——`color`, `border-radius`。
    Ident(String),
    /// 数字字面量——`16px`, `3.14`, `50%`。
    Number(String),
    /// 字符串字面量——已去引号，附带引号字符。
    String(String, char),
    /// 颜色/ID hash——`#ff0000`、`#main`。
    Hash(String),
    /// `#{...}` 插值内容。
    Interp(String),
    /// 注释内容——`/* ... */` 或 `// ...`。
    Comment(String, bool), // (text, is_silent)  is_silent = //

    // —— 关键字 ——
    True,
    False,
    Null,
    And,
    Or,
    Not,

    // —— 符号 ——
    LParen,      // (
    RParen,      // )
    LBrace,      // {
    RBrace,      // }
    LBracket,    // [
    RBracket,    // ]
    Colon,       // :
    Semicolon,   // ;
    Comma,       // ,
    Dot,         // .
    Plus,        // +
    Minus,       // -
    Star,        // *
    Slash,       // /
    Percent,     // %
    Amp,         // &
    Caret,       // ^
    Tilde,       // ~
    Bang,        // !
    Assign,      // =
    Eq,          // ==
    NotEq,       // !=
    Less,        // <
    Greater,     // >
    LessEq,      // <=
    GreaterEq,   // >=
    DotDotDot,   // ...
    Pipe,        // |  (for @supports selector(|...)

    // —— 特殊 ——
    /// `@规则`——`@import`, `@media`。
    AtRule(String),
    /// `$变量`——`$color`, `$width`。
    Dollar(String),
    /// 空白字符。
    Whitespace,
    /// 文件结束。
    Eof,
}

impl Token {
    /// 判断是否为可忽略的 token（空白、EOF）。
    pub fn is_trivia(&self) -> bool {
        matches!(self, Token::Whitespace | Token::Eof)
    }

    /// 判断是否为空白。
    pub fn is_whitespace(&self) -> bool {
        matches!(self, Token::Whitespace)
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Ident(s) => write!(f, "{s}"),
            Token::Number(s) => write!(f, "{s}"),
            Token::String(s, q) => write!(f, "{q}{s}{q}"),
            Token::Hash(s) => write!(f, "#{s}"),
            Token::Interp(s) => write!(f, "#{{{s}}}"),
            Token::Comment(s, false) => write!(f, "/*{s}*/"),
            Token::Comment(s, true) => write!(f, "//{s}"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::Null => write!(f, "null"),
            Token::And => write!(f, "and"),
            Token::Or => write!(f, "or"),
            Token::Not => write!(f, "not"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::Colon => write!(f, ":"),
            Token::Semicolon => write!(f, ";"),
            Token::Comma => write!(f, ","),
            Token::Dot => write!(f, "."),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::Amp => write!(f, "&"),
            Token::Caret => write!(f, "^"),
            Token::Tilde => write!(f, "~"),
            Token::Bang => write!(f, "!"),
            Token::Assign => write!(f, "="),
            Token::Eq => write!(f, "=="),
            Token::NotEq => write!(f, "!="),
            Token::Less => write!(f, "<"),
            Token::Greater => write!(f, ">"),
            Token::LessEq => write!(f, "<="),
            Token::GreaterEq => write!(f, ">="),
            Token::DotDotDot => write!(f, "..."),
            Token::Pipe => write!(f, "|"),
            Token::AtRule(s) => write!(f, "@{s}"),
            Token::Dollar(s) => write!(f, "${s}"),
            Token::Whitespace => write!(f, " "),
            Token::Eof => write!(f, "EOF"),
        }
    }
}
