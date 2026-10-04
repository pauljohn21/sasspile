//! Directive operator implementations.
//!
//! All AST node types dispatch through a single `SassOp for AstNode` impl
//! in `dispatch.rs`. Submodules here exist for future per-directive logic
//! extraction (e.g., control-flow helpers, module loading).

mod dispatch;