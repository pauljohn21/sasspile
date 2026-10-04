//! Reactive compilation pipeline.
//!
//! Module layout:
//! - `bus.rs`:      CompilerBus with multicast subjects and registries
//! - `types.rs`:    Core types (EvalContext, Value, CssStmt, SassOp trait)
//! - `eval.rs`:     Evaluator dispatch + scope pre-analysis
//! - `pipeline/`:   Top-level entry points (from_string, from_string_ast)
//! - `ops/`:        One module per SassOp directive (unified in dispatch.rs)

mod bus;
mod eval;
mod ext;
mod ops;
mod pipeline;
mod types;

pub use bus::{CompilerBus, FnDef, MixinDef, ModuleEvent, ScopeEvent, ValueEvent};
pub use eval::{evaluate, evaluate_to_css, lower_to_css, pre_analysis};
pub use pipeline::compile::{compile_ast, from_string, from_string_ast};
pub use types::{
    AstNode, AstStream, CssStmt, CssStream, EvalContext, SassOp, ScopeId, ScopeKind, Value,
};
