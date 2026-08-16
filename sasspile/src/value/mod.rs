//! Sass value types.
//!
//! All values are immutable and shareable across Tokio tasks via Arc.
//! The Value enum represents every possible Sass runtime value.

mod coerce;
mod color;
mod error;
mod number;
mod ops;

pub use color::SassColor;
pub use error::ValueError;
pub use number::{Number, Unit};

use std::sync::Arc;

/// Sass value enumeration.
#[derive(Debug, Clone)]
pub enum Value {
    /// Numeric value with optional unit.
    Number(Number),
    /// String (quoted or unquoted).
    String(String, Quoted),
    /// Boolean value.
    Boolean(bool),
    /// Sass null (distinct from Option::None).
    Null,
    /// sRGB color.
    Color(SassColor),
    /// List with separator.
    List(Vec<Value>, Separator),
    /// Key-value map.
    Map(Vec<(Value, Value)>),
    /// Argument list with trailing keyword args.
    ArgList(Vec<Value>),
    /// Function reference by name.
    Function(String),
    /// calc() expression (deferred).
    Calculation(String),
    /// Error sentinel for short-circuit propagation.
    Error(String),
}

/// Whether a string is quoted or unquoted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quoted {
    Quoted,
    Unquoted,
}

/// List separator (CSS output formatting).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Separator {
    Comma,
    Space,
    Slash,
    Undecided,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a, _), Value::String(b, _)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Color(a), Value::Color(b)) => a == b,
            (Value::List(a, sa), Value::List(b, sb)) => a == b && sa == sb,
            (Value::Map(a), Value::Map(b)) => a.len() == b.len(),
            _ => false,
        }
    }
}

/// Convenience alias for Arc<Value>.
pub type SharedValue = Arc<Value>;
