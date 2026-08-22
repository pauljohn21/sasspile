//! scss-rs — 纯 Rust SCSS 编译器。
//!
//! ## 架构
//!
//! ```text
//! Source ──► Lexed ──► Parsed ──► Evaluated ──► Serialized
//! ```
//!
//! 每个阶段是不可变类型，阶段转换是 `TryFrom` 实现。
//! 所有内部类型 `pub(crate)`，仅公开 4 个编译入口函数。

mod error;
mod source;
mod lex;
mod parse;
mod eval;
mod css;

use std::path::{Path, PathBuf};

pub use error::{SassError, Result};
pub use css::OutputStyle;

/// 编译字符串源码。
pub fn compile(input: &str, style: OutputStyle) -> Result<String> {
    let source = source::Source::new(input);
    let lexed = source.lex()?;
    let parsed = lexed.parse()?;
    let evaluated = parsed.evaluate()?;
    Ok(evaluated.serialize(style).into_string())
}

/// 编译字符串——展开模式。
pub fn compile_expanded(input: &str) -> Result<String> {
    compile(input, OutputStyle::Expanded)
}

/// 编译文件。
pub fn compile_file(path: &Path, style: OutputStyle) -> Result<String> {
    let source = source::Source::from_file(path)?;
    let lexed = source.lex()?;
    let parsed = lexed.parse()?;
    let evaluated = parsed.evaluate()?;
    Ok(evaluated.serialize(style).into_string())
}

/// 编译文件——指定 load paths。
pub fn compile_file_with_paths(
    path: &Path,
    load_paths: &[PathBuf],
    style: OutputStyle,
) -> Result<String> {
    let source = source::Source::from_file(path)?.with_load_paths(load_paths);
    let lexed = source.lex()?;
    let parsed = lexed.parse()?;
    let evaluated = parsed.evaluate()?;
    Ok(evaluated.serialize(style).into_string())
}
