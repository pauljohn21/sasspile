//! 阶段 1: Source —— 原始 SCSS 源码。
//!
//! 这是编译管线的起点，封装待编译的 SCSS 源码文本。

use super::lexed::Lexed;
use crate::error::Result;

/// 编译管线的起点——封装原始源码文本。
///
/// # 示例
///
/// ```
/// use sasspile::stage::source::Source;
///
/// let source = Source::new("a { color: red; }".to_string());
/// let lexed = source.lex().unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct Source {
    /// 原始 SCSS 源码。
    pub text: String,
}

impl Source {
    /// 创建新的 Source 实例。
    ///
    /// # 参数
    /// - `text`: SCSS 源码字符串。
    pub fn new(text: String) -> Self {
        Self { text }
    }

    /// 词法分析——Source → Lexed。
    ///
    /// 消费自身，返回 Token 序列或错误。
    ///
    /// # 返回
    /// 成功返回 `Lexed`（包含 Token 序列），失败返回词法错误。
    ///
    /// # 示例
    ///
    /// ```
    /// use sasspile::stage::source::Source;
    ///
    /// let source = Source::new("a { color: red; }".to_string());
    /// let lexed = source.lex().unwrap();
    /// ```
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

