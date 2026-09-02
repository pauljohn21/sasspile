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
    /// 布尔字面量 `true`。
    True,
    /// 布尔字面量 `false`。
    False,
    /// 空值字面量 `null`。
    Null,
    /// 逻辑与关键字 `and`。
    And,
    /// 逻辑或关键字 `or`。
    Or,
    /// 逻辑非关键字 `not`。
    Not,

    // —— 符号 ——
    /// 左圆括号 `(`。
    LParen,
    /// 右圆括号 `)`。
    RParen,
    /// 左花括号 `{`。
    LBrace,
    /// 右花括号 `}`。
    RBrace,
    /// 左方括号 `[`。
    LBracket,
    /// 右方括号 `]`。
    RBracket,
    /// 冒号 `:`——声明分隔符。
    Colon,
    /// 分号 `;`——语句终止符。
    Semicolon,
    /// 逗号 `,`——列表/参数分隔符。
    Comma,
    /// 点 `.`——小数点或类选择器。
    Dot,
    /// 加号 `+`。
    Plus,
    /// 减号 `-`。
    Minus,
    /// 星号 `*`——乘法或通配选择器。
    Star,
    /// 斜杠 `/`——除法或路径分隔符。
    Slash,
    /// 百分号 `%`——取模或百分比单位。
    Percent,
    /// & 符号——父选择器引用。
    Amp,
    /// 脱字符 `^`。
    Caret,
    /// 波浪号 `~`——同级选择器。
    Tilde,
    /// 感叹号 `!`——`!important` / `!default` / `!global`。
    Bang,
    /// 赋值号 `=`。
    Assign,
    /// 等于比较 `==`。
    Eq,
    /// 不等于比较 `!=`。
    NotEq,
    /// 小于比较 `<`。
    Less,
    /// 大于比较 `>`。
    Greater,
    /// 小于等于比较 `<=`。
    LessEq,
    /// 大于等于比较 `>=`。
    GreaterEq,
    /// 剩余参数展开符 `...`。
    DotDotDot,
    /// 竖线 `|`——`@supports selector(|...)` 语法。
    Pipe,

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
        use crate::parse::ast::Value;
        match self {
            Token::Ident(s) => write!(f, "{}", Value::escape_css_ident(s)),
            Token::Number(s) => write!(f, "{s}"),
            Token::String(s, _q) => {
                let (quote, escaped) = Value::escape_quoted_string(s);
                // 使用 escape_quoted_string 选择的引号（智能避免冲突）
                write!(f, "{quote}{escaped}{quote}")
            }
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
