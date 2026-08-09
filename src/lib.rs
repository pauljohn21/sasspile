//! sasspile v2 —— 纯 Rust 函数式 SCSS 编译器。
//!
//! 核心设计：类型状态机管线 + Iterator + fold + 不可变数据。
//!
//! ```text
//! Source { text }
//!     .lex()?        -> Lexed
//!     .parse()?      -> Parsed
//!     .evaluate()?   -> Evaluated
//!     .serialize()   -> Serialized
//! ```

// 模块声明
pub mod css;
pub mod error;
pub mod eval;
pub mod lex;
pub mod parse;
pub mod stage;

// 重导出常用类型
pub use css::node::CssNode;
pub use error::{Result, SassError};
pub use stage::serialized::Serialized;
pub use stage::source::Source;

/// 输出风格。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStyle {
    /// 展开式——带缩进和换行。
    Expanded,
    /// 压缩式——无空白。
    Compressed,
}

/// 编译 SCSS 源码为 CSS（完整管线）。
///
/// # 参数
/// - `source`: SCSS 源码字符串。
/// - `style`: 输出风格。
///
/// # 返回
/// 成功返回 CSS 字符串，失败返回 [`SassError`]。
pub fn compile(source: &str, style: OutputStyle) -> Result<String> {
    let css = Source::new(source.to_string())
        .lex()?
        .parse()?
        .evaluate()?
        .serialize(style)
        .css;
    Ok(css)
}

/// 编译 SCSS 源码为 CSS（展开式）。
pub fn compile_expanded(source: &str) -> Result<String> {
    compile(source, OutputStyle::Expanded)
}

/// 编译 SCSS 源码为 CSS（压缩式）。
pub fn compile_compressed(source: &str) -> Result<String> {
    compile(source, OutputStyle::Compressed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_rule() {
        let css = compile_expanded("a { color: red; }").unwrap();
        assert_eq!(css, "a {\n  color: red;\n}\n");
    }

    #[test]
    fn test_compile_compressed() {
        let css = compile_compressed("a { color: red; }").unwrap();
        assert_eq!(css, "a{color:red;}");
    }

    #[test]
    fn test_compile_with_comment() {
        let css = compile_expanded("/* license */\na { color: red; }").unwrap();
        assert!(css.contains("license"));
        assert!(css.contains("color: red"));
    }

    #[test]
    fn test_compile_variable() {
        let css = compile_expanded("$w: 10px; a { width: $w; }").unwrap();
        assert!(css.contains("width: 10px"));
    }

    #[test]
    fn test_error_on_invalid_syntax() {
        let result = compile_expanded("a { : }");
        assert!(result.is_err());
    }
}
