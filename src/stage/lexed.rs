//! 阶段 2: Lexed —— Token 序列。
//!
//! 词法分析器的输出，包含从源码中提取的所有 Token。

use super::parsed::Parsed;
use crate::error::Result;
use crate::lex::token::Token;

/// 词法分析产物——有序 Token 序列。
///
/// 由 `Source::lex()` 产生，作为语法分析的输入。
///
/// # 示例
///
/// ```
/// use sasspile::stage::source::Source;
///
/// let lexed = Source::new("a { color: red; }".to_string()).lex().unwrap();
/// let parsed = lexed.parse().unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct Lexed {
    /// Token 列表。
    pub tokens: Vec<Token>,
}

impl Lexed {
    /// 语法分析——Lexed → Parsed。
    ///
    /// 消费自身，返回抽象语法树（AST）或错误。
    ///
    /// # 返回
    /// 成功返回 `Parsed`（包含 AST），失败返回语法错误。
    pub fn parse(self) -> Result<Parsed> {
        use crate::parse::Parser;

        let ast = Parser::parse(&self.tokens)?;
        Ok(Parsed { ast })
    }
}

