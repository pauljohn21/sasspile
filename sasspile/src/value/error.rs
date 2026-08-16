//! Value-level errors.

use thiserror::Error;

/// Errors that can occur during value operations.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ValueError {
    /// Incompatible units in arithmetic.
    #[error("incompatible units: {0} and {1}")]
    IncompatibleUnits(String, String),

    /// Division by zero.
    #[error("division by zero")]
    DivisionByZero,

    /// Invalid operand for unary operation.
    #[error("cannot apply {0} to {1}")]
    InvalidOperand(&'static str, &'static str),

    /// Invalid operands for binary operation.
    #[error("cannot apply {0} to {1} and {2}")]
    InvalidOperands(&'static str, &'static str, &'static str),
}
