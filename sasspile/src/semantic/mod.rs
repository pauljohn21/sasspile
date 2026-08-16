//! Semantic analysis — validates AST structure and collects symbols.
//!
//! Runs after parsing to:
//! - Build scope-aware symbol tables
//! - Resolve @use/@forward module dependencies
//! - Validate @extend targets exist
//! - Register mixin and function definitions with arity

mod definitions;
mod extend;
mod module;
mod symbol_table;

pub use definitions::{
    DefinitionKind, DefinitionRegistry, DuplicateInfo,
    FunctionEntry, MixinEntry,
};
pub use extend::{collect_extends, SelectorRegistry};
pub use module::{CycleCheck, Module, ModuleGraph, NamespaceRegistry};
pub use symbol_table::{Scope, ScopeKind, SymbolEntry, SymbolTable};

