//! Lexer — 词法分析，产出 Token 流。

use crate::error::{Result, SassError};
use crate::source::Source;
use std::path::PathBuf;

pub(crate) mod token;
mod scan;

pub use token::Token;

/// 词法分析完成。
pub(crate) struct Lexed {
    pub(crate) tokens: Vec<Token>,
    pub(crate) base_path: Option<PathBuf>,
    pub(crate) load_paths: Vec<PathBuf>,
}

impl TryFrom<Source> for Lexed {
    type Error = SassError;

    fn try_from(source: Source) -> Result<Self> {
        let mut scanner = scan::Scanner::new(&source.text);
        let mut tokens = Vec::new();
        while let Some(token) = scanner.next_token()? {
            tokens.push(token);
        }
        Ok(Self {
            tokens,
            base_path: source.base_path,
            load_paths: source.load_paths,
        })
    }
}

impl Lexed {
    /// 语法分析——Lexed → Parsed。
    pub fn parse(self) -> Result<crate::parse::Parsed> {
        crate::parse::Parsed::try_from(self)
    }
}
