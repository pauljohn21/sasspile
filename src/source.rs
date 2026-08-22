//! Source — 源码文本封装 + 文件加载。

use crate::error::{Result, SassError};
use std::fs;
use std::path::{Path, PathBuf};

/// 源码文本 + 文件路径 + load paths。
pub(crate) struct Source {
    pub(crate) text: String,
    pub(crate) base_path: Option<PathBuf>,
    pub(crate) load_paths: Vec<PathBuf>,
}

impl Source {
    /// 从字符串创建。
    pub fn new(text: &str) -> Self {
        Self { text: text.to_string(), base_path: None, load_paths: Vec::new() }
    }

    /// 从文件读取。
    pub fn from_file(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)
            .map_err(|e| SassError { message: format!("Cannot read {}: {e}", path.display()), kind: crate::error::ErrorKind::Io })?;
        Ok(Self {
            text,
            base_path: Some(path.to_path_buf()),
            load_paths: Vec::new(),
        })
    }

    /// 设置 load paths（@use/@import 搜索路径）。
    pub fn with_load_paths(mut self, paths: &[PathBuf]) -> Self {
        self.load_paths = paths.to_vec();
        self
    }

    /// 词法分析——Source → Lexed。
    pub fn lex(self) -> Result<crate::lex::Lexed> {
        crate::lex::Lexed::try_from(self)
    }
}
