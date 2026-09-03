//! 阶段 2: Lexed —— Token 序列。
//!
//! 词法分析器的输出，包含从源码中提取的所有 Token。
//! 携带文件路径和加载路径，传递给后续阶段。

use super::parsed::Parsed;
use crate::error::Result;
use crate::lex::token::Token;
use std::path::PathBuf;

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
    /// 源文件路径（透传给 Parsed）。
    pub base_path: Option<PathBuf>,
    /// 加载路径（透传给 Parsed）。
    pub load_paths: Vec<PathBuf>,
}

impl Lexed {
    /// 语法分析——Lexed → Parsed。
    ///
    /// 消费自身，返回抽象语法树（AST）或错误。
    ///
    /// # Errors
    ///
    /// 返回 [`SassError`] 如果语法分析遇到错误。
    pub fn parse(self) -> Result<Parsed> {
        use crate::parse::Parser;

        let ast = Parser::parse(&self.tokens)?;
        Ok(Parsed {
            ast,
            base_path: self.base_path,
            load_paths: self.load_paths,
        })
    }
}
