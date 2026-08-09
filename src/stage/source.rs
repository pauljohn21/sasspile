//! 阶段 1: Source —— 原始 SCSS 源码。

use super::lexed::Lexed;
use crate::error::Result;

/// 编译管线的起点——封装原始源码文本。
#[derive(Debug, Clone)]
pub struct Source {
    /// 原始 SCSS 源码。
    pub text: String,
}

impl Source {
    /// 创建新的 Source 实例。
    pub fn new(text: String) -> Self {
        Self { text }
    }

    /// 词法分析——Source → Lexed。
    ///
    /// 消费自身，返回 Token 序列或错误。
    pub fn lex(self) -> Result<Lexed> {
        use crate::lex::Lexer;
        use crate::lex::token::Token;

        // 保留 Whitespace 令牌——@media 等 @规则的参数中需要它
        let tokens: Vec<_> = Lexer::new(&self.text)
            .filter(|t| !matches!(t.as_ref(), Ok(Token::Eof)))
            .collect::<Result<Vec<_>>>()?;

        Ok(Lexed { tokens })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_creation() {
        let src = Source::new("a { color: red; }".to_string());
        assert_eq!(src.text, "a { color: red; }");
    }

    #[test]
    fn test_source_to_lexed() {
        let src = Source::new("a".to_string());
        let lexed = src.lex().unwrap();
        assert_eq!(lexed.tokens.len(), 1);
    }
}
