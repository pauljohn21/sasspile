//! 阶段 1: Source —— 原始 SCSS 源码。
//!
//! 这是编译管线的起点，封装待编译的 SCSS 源码文本。
//! 可携带文件路径（用于 @use/@import 解析）。

use super::lexed::Lexed;
use crate::error::Result;
use std::path::PathBuf;

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
    /// 源文件路径（可选——用于 @use/@import 解析和 plain CSS 检测）。
    pub base_path: Option<PathBuf>,
    /// 加载路径（可选——用于 @use/@import 模块搜索）。
    pub load_paths: Vec<PathBuf>,
}

impl Source {
    /// 从字符串创建 Source（无文件路径）。
    ///
    /// # 参数
    /// - `text`: SCSS 源码字符串。
    pub fn new(text: String) -> Self {
        Self { text, base_path: None, load_paths: vec![] }
    }

    /// 从文件创建 Source——读取文件内容并携带路径。
    ///
    /// # 参数
    /// - `path`: SCSS 文件路径。
    ///
    /// # 错误
    /// 返回 [`SassError`] 如果文件不存在或读取失败。
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Ok(Self { text, base_path: Some(path.clone()), load_paths: vec![] })
    }

    /// 设置加载路径——用于 @use/@import 模块搜索。
    ///
    /// 消费自身，返回带加载路径的 Source。
    pub fn with_load_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.load_paths = paths;
        self
    }

    /// 词法分析——Source → Lexed。
    ///
    /// 消费自身，返回 Token 序列或错误。
    ///
    /// # 返回
    /// 成功返回 `Lexed`（包含 Token 序列和路径信息），失败返回词法错误。
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

        Ok(Lexed { tokens, base_path: self.base_path, load_paths: self.load_paths })
    }
}
