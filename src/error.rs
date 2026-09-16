//! CompileError 枚举 — 所有编译阶段错误类型的统一表达
//!
//! 违反 MUST: 错误封装为 Result 值在流中传播，不调用 observer.error()

use core::fmt;

/// 编译错误类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum CompileError {
    /// 值缺失（如空 body）
    MissingValue {
        message: String,
    },
    /// 非法输入（如语法错误）
    InvalidInput {
        message: String,
    },
    /// 未定义变量引用
    UndefinedVariable {
        name: String,
    },
    /// 模块加载失败
    ModuleLoadFailure {
        path: String,
        reason: String,
    },
    /// @extend 规则冲突
    ExtendConflict {
        selector: String,
        reason: String,
    },
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingValue { message } => write!(f, "missing value: {message}"),
            Self::InvalidInput { message } => write!(f, "invalid input: {message}"),
            Self::UndefinedVariable { name } => write!(f, "undefined variable: ${name}"),
            Self::ModuleLoadFailure { path, reason } => {
                write!(f, "module load failure: {path}: {reason}")
            }
            Self::ExtendConflict { selector, reason } => {
                write!(f, "extend conflict: {selector}: {reason}")
            }
        }
    }
}

impl core::error::Error for CompileError {}
