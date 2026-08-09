//! Token 定义——词法分析器的产出。
//!
//! 每个 Token 代表源码中的一个语法单位，如标识符、数字、符号等。

/// 词法单元。
///
/// # 示例
///
/// ```
/// use sasspile::lex::token::Token;
///
/// assert!(matches!(Token::Ident("color".to_string()), Token::Ident(_)));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// 标识符——`color`, `border-radius`。
    Ident(String),

    /// 数字字面量——`16px`, `3.14`, `50%`。
    Number(String),

    /// 字符串字面量——已去引号。
    String(String),

    /// 颜色 hash——`#ff0000`。
    Hash(String),

    /// 注释——`/* ... */` 的内容。
    Comment(String),

    // —— 关键字 ——
    /// `true`
    True,
    /// `false`
    False,
    /// `null`
    Null,

    // —— 符号 ——
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// `:`
    Colon,
    /// `;`
    Semicolon,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `%`
    Percent,
    /// `!`
    Bang,
    /// `=`
    Assign,
    /// `==`
    Eq,
    /// `!=`
    NotEq,
    /// `<`
    Less,
    /// `>`
    Greater,
    /// `<=`
    LessEq,
    /// `>=`
    GreaterEq,

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
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Ident(s) => write!(f, "{s}"),
            Token::Number(s) => write!(f, "{s}"),
            Token::String(s) => write!(f, "{s:?}"),
            Token::Hash(s) => write!(f, "#{s}"),
            Token::Comment(s) => write!(f, "/*{s}*/"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::Null => write!(f, "null"),
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
            Token::Bang => write!(f, "!"),
            Token::Assign => write!(f, "="),
            Token::Eq => write!(f, "=="),
            Token::NotEq => write!(f, "!="),
            Token::Less => write!(f, "<"),
            Token::Greater => write!(f, ">"),
            Token::LessEq => write!(f, "<="),
            Token::GreaterEq => write!(f, ">="),
            Token::AtRule(s) => write!(f, "@{s}"),
            Token::Dollar(s) => write!(f, "${s}"),
            Token::Whitespace => write!(f, " "),
            Token::Eof => write!(f, "EOF"),
        }
    }
}
