//! 阶段 2: Lexed —— Token 序列。

use super::parsed::Parsed;
use crate::error::Result;
use crate::lex::token::Token;

/// 词法分析产物——有序 Token 序列。
#[derive(Debug, Clone)]
pub struct Lexed {
    /// Token 列表。
    pub tokens: Vec<Token>,
}

impl Lexed {
    /// 语法分析——Lexed → Parsed。
    pub fn parse(self) -> Result<Parsed> {
        use crate::parse::Parser;

        let ast = Parser::parse(&self.tokens)?;
        Ok(Parsed { ast })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::token::Token;

    #[test]
    fn test_lexed_parse() {
        let lexed = Lexed {
            tokens: vec![
                Token::Ident("a".to_string()),
                Token::Whitespace,
                Token::LBrace,
                Token::Ident("color".to_string()),
                Token::Colon,
                Token::Ident("red".to_string()),
                Token::Semicolon,
                Token::RBrace,
            ],
        };
        let parsed = lexed.parse();
        assert!(parsed.is_ok());
    }
}
