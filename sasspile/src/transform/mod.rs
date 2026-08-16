//! Transform stage — AST → AST semantic expansion.
//!
//! Runs after Parse and before CSS Gen to resolve variables,
//! expand mixins/control-flow, and produce a fully evaluable AST.

pub mod control_flow;
pub mod expand;
pub mod mixins;
pub mod variables;

use tracing::info_span;

use crate::semantic::{DefinitionRegistry, SymbolTable};
use crate::source::SourceSpan;
use crate::{Node, Result, Stylesheet};

/// Maximum recursion depth for mixin inclusion.
pub const MAX_CALL_DEPTH: usize = 64;

/// Maximum loop iterations (for @while protection).
pub const MAX_LOOP_ITERATIONS: usize = 10000;

/// Transform context holding the symbol table and definition registry.
///
/// Created once per `transform_stylesheet` call, it carries all state
/// needed to perform the semantic transformation.
#[derive(Debug)]
pub struct TransformCtx {
    /// Scope-aware variable storage.
    pub(crate) symbols: SymbolTable,
    /// Mixin and function definition registry.
    pub(crate) definitions: DefinitionRegistry,
    /// Current mixin include call depth.
    pub(crate) call_depth: usize,
}

impl TransformCtx {
    /// Create a new transform context with empty global scope.
    pub fn new() -> Self {
        Self {
            symbols: SymbolTable::new(),
            definitions: DefinitionRegistry::new(),
            call_depth: 0,
        }
    }

    /// Transform a stylesheet — entry point for the transform stage.
    ///
    /// Takes ownership of the parsed AST, returns a new AST with all
    /// semantic constructs (variables, mixins, control flow) expanded.
    pub fn transform_stylesheet(&mut self, stylesheet: Stylesheet) -> Result<Stylesheet> {
        let span = info_span!("transform_stylesheet", nodes = stylesheet.nodes.len());
        let _enter = span.enter();

        // Phase 1: Collect all definitions (variables, mixins, functions).
        self.collect_definitions(&stylesheet.nodes)?;

        // Phase 2: Expand the AST recursively.
        let expanded = self.expand_nodes(&stylesheet.nodes)?;

        Ok(Stylesheet { nodes: expanded })
    }

    /// Collect variable/mixin/function definitions from a node list.
    fn collect_definitions(&mut self, nodes: &[Node]) -> Result<()> {
        let span = info_span!("collect_definitions", count = nodes.len());
        let _enter = span.enter();
        variables::collect_definitions(self, nodes)
    }

    /// Recursively expand AST nodes.
    pub(crate) fn expand_nodes(&mut self, nodes: &[Node]) -> Result<Vec<Node>> {
        let span = info_span!("expand_nodes", count = nodes.len());
        let _enter = span.enter();
        expand::expand_nodes(self, nodes)
    }

    /// Create a source span placeholder for synthesized nodes.
    #[allow(dead_code)]
    pub(crate) fn placeholder_span(&self) -> SourceSpan {
        SourceSpan::new(0, 0)
    }

    /// Register a mixin definition (suppress diagnostics).
    pub(crate) fn register_mixin(&mut self, def: &crate::parser::MixinDef) {
        let mut diags = crate::diagnostics::Diagnostics::new();
        let _ = self.definitions.register_mixin(def, &mut diags);
    }

    /// Register a function definition (suppress diagnostics).
    pub(crate) fn register_function(&mut self, def: &crate::parser::FunctionDef) {
        let mut diags = crate::diagnostics::Diagnostics::new();
        let _ = self.definitions.register_function(def, &mut diags);
    }
}

impl Default for TransformCtx {
    fn default() -> Self {
        Self::new()
    }
}
