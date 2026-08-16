//! Expression evaluation — AST → Value.
//!
//! Walks parsed expression trees, resolves variables through
//! the symbol table, applies operators, and dispatches
//! function/mixin calls.

pub mod collections;
pub mod error;
pub mod evaluator;
pub mod functions;
pub mod ops;

pub use error::EvalError;
pub use evaluator::EvalContext;
pub use ops::{binary, unary};
