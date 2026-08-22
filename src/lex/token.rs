//! Token 类型定义。

/// 引号风格。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuoteStyle {
    None,
    Single,
    Double,
}

/// 词法 token。
#[derive(Debug, Clone)]
pub enum Token {
    // 字面量
    Ident(String),
    Number(String, Option<String>),  // raw value, unit
    String(String, QuoteStyle),
    HexColor(String),  // #ff0000
    Variable(String),  // $name

    // 标点
    LBrace, RBrace,       // { }
    LParen, RParen,       // ( )
    LBracket, RBracket,  // [ ]
    Colon, Semicolon,     // : ;
    Comma, Dot,           // , .
    Hash,                 // #
    At,                   // @
    Ampersand,            // &

    // 运算符
    Plus, Minus, Star, Slash, Percent,  // + - * / %
    Eq, Gt, Lt, Gte, Lte,               // = > < >= <=
    NotEq,                               // !=
    Arrow,                               // =>

    // 特殊
    Interp(String),      // #{...}
    Comment(String),       // /* */
    SilentComment(String), // //
    Eof,
}

impl Token {
    pub fn as_str(&self) -> &str {
        match self {
            Token::Ident(s) | Token::String(s, _) | Token::HexColor(s) | Token::Variable(s) => s,
            Token::Number(n, _) => n,
            Token::Interp(s) | Token::Comment(s) | Token::SilentComment(s) => s,
            _ => "",
        }
    }
}
