//! 统一错误类型——使用 thiserror 派生。
//!
//! 错误携带源码位置信息（Span），支持精确定位。

use thiserror::Error;

/// 源码位置区间。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Span {
    /// 起始字节偏移。
    pub start: usize,
    /// 结束字节偏移。
    pub end: usize,
}

impl Span {
    /// 创建新的 Span。
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// 单位置 Span。
    pub fn at(pos: usize) -> Self {
        Self { start: pos, end: pos + 1 }
    }
}

/// sasspile 错误类型。
#[derive(Debug, Error)]
pub enum SassError {
    /// 词法错误——扫描阶段遇到无效字符。
    #[error("词法错误: {message} (位置 {pos})")]
    Lex { message: String, pos: usize },

    /// 语法错误——解析阶段结构不匹配。
    #[error("语法错误: 期望 {expected}, 实际 {found}")]
    Parse { expected: String, found: String },

    /// 求值错误——运行时问题。
    #[error("求值错误: {0}")]
    Eval(String),

    /// 类型错误——类型不匹配。
    #[error("类型错误: 期望 {expected}, 实际 {actual}")]
    Type { expected: String, actual: String },

    /// 单位错误——不兼容单位运算。
    #[error("单位错误: {0}")]
    Unit(String),

    /// 未定义变量。
    #[error("未定义变量: ${0}")]
    UndefinedVariable(String),

    /// 未定义 mixin。
    #[error("未定义 mixin: {0}")]
    UndefinedMixin(String),

    /// 未定义函数。
    #[error("未定义函数: {0}")]
    UndefinedFunction(String),

    /// 除零错误。
    #[error("除零错误")]
    DivideByZero,

    /// 模块加载错误。
    #[error("模块错误: {0}")]
    Module(String),

    /// IO 错误。
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
}

/// 结果类型别名。
pub type Result<T> = std::result::Result<T, SassError>;
