//! 统一错误类型。
//!
//! 使用 `thiserror` 派生 `std::error::Error` trait。
//!
//! # 错误处理示例
//!
//! ```
//! use sasspile::{compile_expanded, SassError};
//!
//! match compile_expanded("a { color: $undefined; }") {
//!     Ok(css) => println!("{}", css),
//!     Err(SassError::UndefinedVariable(name)) => {
//!         eprintln!("错误: 变量 '{}' 未定义", name);
//!     }
//!     Err(e) => eprintln!("编译失败: {}", e),
//! }
//! ```

use thiserror::Error;

/// sasspile 错误类型。
#[derive(Debug, Error)]
pub enum SassError {
    /// 词法错误——扫描阶段遇到无效字符。
    #[error("词法错误: {message} (位置 {position})")]
    LexError {
        /// 错误描述。
        message: String,
        /// 源码字节位置。
        position: usize,
    },

    /// 语法错误——解析阶段结构不匹配。
    #[error("语法错误: 期望 {expected}, 实际 {found}")]
    ParseError {
        /// 期望的 token。
        expected: String,
        /// 实际遇到的 token。
        found: String,
    },

    /// 求值错误——运行时问题。
    #[error("求值错误: {0}")]
    EvalError(String),

    /// 类型错误——类型不匹配。
    #[error("类型错误: 期望 {expected}, 实际 {actual}")]
    TypeError {
        /// 期望的类型名。
        expected: String,
        /// 实际的类型名。
        actual: String,
    },

    /// 单位错误——不兼容单位转换。
    #[error("单位错误: 无法将 {from} 转换为 {to}")]
    UnitError {
        /// 源单位。
        from: String,
        /// 目标单位。
        to: String,
    },

    /// 未定义变量。
    #[error("未定义变量: {0}")]
    UndefinedVariable(String),

    /// IO 错误。
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
}

/// 结果类型别名。
pub type Result<T> = std::result::Result<T, SassError>;
