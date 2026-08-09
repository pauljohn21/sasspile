//! sasspile v2 —— 纯 Rust 函数式 SCSS 编译器。
//!
//! # 核心设计
//!
//! sasspile 采用**类型状态机管线**（Type-State Pattern）构建编译流程，
//! 结合 Iterator、fold 与不可变数据结构，实现零副作用的编译管线。
//!
//! ## 管线流程图
//!
//! ```text
//! Source { content: String }
//!     |
//!     | .lex()?
//!     v
//! Lexed { tokens: Vec<Token> }
//!     |
//!     | .parse()?
//!     v
//! Parsed { ast: Vec<Node> }
//!     |
//!     | .evaluate()?
//!     v
//! Evaluated { root: CssNode }
//!     |
//!     | .serialize(style)
//!     v
//! Serialized { css: String }
//! ```
//!
//! # 使用示例
//!
//! ```
//! use sasspile::{compile_expanded, OutputStyle};
//!
//! let scss = "a { color: red; }";
//! let css = compile_expanded(scss).unwrap();
//! assert_eq!(css, "a {\n  color: red;\n}\n");
//! ```
//!
//! ## 使用自定义输出风格
//!
//! ```
//! use sasspile::{compile, OutputStyle};
//!
//! let css = compile("$w: 10px; a { width: $w; }", OutputStyle::Compressed).unwrap();
//! assert_eq!(css, "a{width:10px;}");
//! ```
//!
//! # 架构说明
//!
//! 每个编译阶段对应一个类型，通过方法调用实现阶段转换。
//! 这种设计确保：
//! - 必须先解析后求值（类型层面保证）
//! - 必须先求值后序列化（类型层面保证）
//! - 编译错误在编译期被阻止

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

/// CSS 输出风格。
///
/// # 示例
///
/// ```
/// use sasspile::{compile, OutputStyle};
///
/// let css = compile("a { color: red; }", OutputStyle::Expanded).unwrap();
/// assert!(css.contains('\n')); // 展开式包含换行
///
/// let css = compile("a { color: red; }", OutputStyle::Compressed).unwrap();
/// assert!(!css.contains('\n')); // 压缩式不包含换行
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStyle {
    /// 展开式——带缩进和换行，便于阅读。
    Expanded,
    /// 压缩式——无空白，最小化输出。
    Compressed,
}

/// 编译 SCSS 源码为 CSS。
///
/// 这是完整的编译管线入口：源码 -> 词法分析 -> 语法分析 -> 求值 -> 序列化。
///
/// # 参数
///
/// - `source`: SCSS 源码字符串。
/// - `style`: 输出风格（展开式或压缩式）。
///
/// # 返回
///
/// 成功返回 CSS 字符串，失败返回 [`SassError`]。
///
/// # 示例
///
/// ```
/// use sasspile::{compile, OutputStyle};
///
/// let css = compile("$primary: blue; a { color: $primary; }", OutputStyle::Expanded).unwrap();
/// assert!(css.contains("color: blue"));
/// ```
///
/// # 错误
///
/// 可能返回的错误类型：
/// - 词法分析错误（如非法字符）
/// - 语法分析错误（如缺少分号）
/// - 求值错误（如未定义变量）
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
///
/// 等价于 `compile(source, OutputStyle::Expanded)`。
///
/// # 示例
///
/// ```
/// use sasspile::compile_expanded;
///
/// let css = compile_expanded("a { color: red; }").unwrap();
/// assert_eq!(css, "a {\n  color: red;\n}\n");
/// ```
pub fn compile_expanded(source: &str) -> Result<String> {
    compile(source, OutputStyle::Expanded)
}

/// 编译 SCSS 源码为 CSS（压缩式）。
///
/// 等价于 `compile(source, OutputStyle::Compressed)`。
///
/// # 示例
///
/// ```
/// use sasspile::compile_compressed;
///
/// let css = compile_compressed("a { color: red; }").unwrap();
/// assert_eq!(css, "a{color:red;}");
/// ```
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
