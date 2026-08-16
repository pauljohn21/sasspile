//! Evaluation errors.

use thiserror::Error;

use crate::value::ValueError;

/// Errors that occur during expression evaluation.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum EvalError {
    /// Variable not found in scope.
    #[error("undefined variable: ${0}")]
    UndefinedVariable(String),

    /// Function or mixin not found.
    #[error("undefined function/mixin: {0}")]
    UndefinedCallable(String),

    /// Wrong number of arguments.
    /// (name, expected_min, expected_max, got)
    #[error("{0} expects {1} argument(s), got {2}")]
    ArityMismatch(String, String, usize),

    /// List index out of bounds.
    #[error("index {0} out of bounds (list has {1} elements)")]
    ListIndexOutOfBounds(usize, usize),

    /// Map key not found.
    #[error("key {0} not found in map")]
    MapKeyNotFound(String),

    /// Attempted division by zero.
    #[error("division by zero")]
    DivisionByZero,

    /// Type mismatch in operation.
    #[error("{0}")]
    TypeError(String),

    /// Maximum call stack depth exceeded.
    #[error("maximum call depth ({0}) exceeded")]
    MaxDepthExceeded(usize),

    /// Propagated value error from underlying ops.
    #[error(transparent)]
    Value(#[from] ValueError),
}

impl EvalError {
    /// Create a type error with formatted message.
    pub fn type_error(expected: &str, got: &str) -> Self {
        Self::TypeError(format!("expected {expected}, got {got}"))
    }
}
