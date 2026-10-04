//! Core reactive types: EvalContext, Value, CssStmt, SassOp trait.

use rxrust::prelude::*;
use std::convert::Infallible;
use std::fmt;
use std::rc::Rc;

use crate::reactive::{CompilerBus, ValueEvent};

/// Monotonic scope identifier.
pub type ScopeId = u64;

/// CSS output statement produced by the evaluator.
#[derive(Debug, Clone)]
pub enum CssStmt {
    /// A simple `property: value;` declaration.
    Decl { property: String, value: String },
    /// A wrapped media query with inner statements.
    Media { query: String, inner: Vec<CssStmt> },
    /// A wrapped supports query with inner statements.
    Supports { query: String, inner: Vec<CssStmt> },
    /// Inner style rule.
    Rule { selector: String, inner: Vec<CssStmt> },
}

impl CssStmt {
    /// Returns true if this statement produces no visible CSS output.
    pub fn is_invisible(&self) -> bool {
        match self {
            CssStmt::Decl { .. } => false,
            CssStmt::Media { inner, .. }
            | CssStmt::Supports { inner, .. }
            | CssStmt::Rule { inner, .. } => inner.iter().all(CssStmt::is_invisible),
        }
    }
}

/// Simplified Sass value type for the reactive pipeline.
#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    List(Vec<Value>),
    Null,
}

/// Scope kind for ScopeEvent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    /// `@media` query scope.
    Media,
    /// `@supports` query scope.
    Supports,
    /// Nested rule scope.
    Rule,
    /// Control-flow scope (`@if`, `@for`, `@each`, `@while`).
    Control,
}

/// Type-erased local observable for AST nodes (cloneable).
pub type AstStream = LocalBoxedObservableClone<'static, AstNode, Infallible>;

/// Type-erased local observable for CSS statements (cloneable).
pub type CssStream = LocalBoxedObservableClone<'static, CssStmt, Infallible>;

/// Compiler evaluation context.
///
/// Replaces `Rc<RefCell<Environment>>` — holds only an immutable bus reference
/// and the current scope id.
#[derive(Clone)]
pub struct EvalContext {
    pub bus: CompilerBus,
    pub scope_id: ScopeId,
}

impl EvalContext {
    /// Create a new `EvalContext` for the given bus and scope.
    pub fn new(bus: CompilerBus, scope_id: ScopeId) -> Self {
        Self { bus, scope_id }
    }

    /// Create a child scope with a new scope id.
    pub fn child_scope(&self, local_idx: u64) -> Self {
        let child_id = self.scope_id * 1000 + local_idx;
        Self {
            bus: self.bus.clone(),
            scope_id: child_id,
        }
    }

    /// Subscribe to variable events relevant to the current scope.
    ///
    /// Returns a boxed local observable of `ValueEvent` items that are
    /// in the current scope or any ancestor scope.
    pub fn ver(
        &self,
        _name: &str,
    ) -> LocalBoxedObservableClone<'static, ValueEvent, Infallible> {
        let current = self.scope_id;
        self.bus
            .var_events()
            .filter(move |evt| matches!(evt, ValueEvent::Bind { scope_id, .. } if *scope_id <= current))
            .box_it_clone()
    }
}

impl fmt::Debug for EvalContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EvalContext")
            .field("scope_id", &self.scope_id)
            .finish_non_exhaustive()
    }
}

/// Trait implemented by each AST node that acts as a directive operator.
///
/// The operator signature is `AstNode → AstNode` (intermediate), not directly
/// `AstNode → CssStmt`, because directives like `@for` may need multiple
/// expansion passes.
pub trait SassOp {
    /// Consume `self` and produce a boxed operator function.
    fn into_operator(
        self,
        ctx: Rc<EvalContext>,
    ) -> Box<dyn Fn(AstStream) -> AstStream>;
}

/// An AST node in the reactive pipeline.
#[derive(Debug, Clone)]
pub enum AstNode {
    /// A variable declaration: `$name: value;`
    VariableDecl { name: String, value: Value },
    /// A CSS style declaration: `property: value;`
    StyleDecl { property: String, value: String },
    /// `@media` rule with query and inner block.
    Media { query: String, inner: Vec<AstNode> },
    /// `@supports` rule with query and inner block.
    Supports { query: String, inner: Vec<AstNode> },
    /// A rule set: `{ ... }`
    RuleSet {
        selector: String,
        inner: Vec<AstNode>,
    },
    /// `@if` conditional
    If {
        cond: Box<AstNode>,
        then_branch: Vec<AstNode>,
        else_branch: Vec<AstNode>,
    },
    /// `@for` loop: `@for $i from 1 through 3 { ... }`
    For {
        var: String,
        from: f64,
        through: f64,
        body: Vec<AstNode>,
    },
    /// `@each` loop: `@each $item in $list { ... }`
    Each {
        var: String,
        body: Vec<AstNode>,
    },
    /// `@while` loop: `@while $i < 10 { ... }`
    While {
        cond: Box<AstNode>,
        body: Vec<AstNode>,
    },
    /// `@warn` directive — emits a diagnostic message.
    Warn { message: String },
    /// `@debug` directive — emits a debug-level expression evaluation.
    Debug { expr: Box<AstNode>, message: String },
    /// Lowered CSS statement — produced by operators as their final output.
    Css(CssStmt),
    /// `@mixin` definition — registers a reusable style block.
    Mixin {
        name: String,
        params: Vec<String>,
        body: Vec<AstNode>,
    },
    /// `@include` — expands a registered mixin at the call site.
    MixinCall { name: String, args: Vec<String> },
    /// `@function` definition — registers a callable function.
    FunctionDecl {
        name: String,
        params: Vec<String>,
        body: Vec<AstNode>,
    },
    /// `@return` — yields a value from a function.
    Return { value: Box<AstNode> },
    /// `@use` rule — imports a module's members into the current scope.
    UseRule { path: String },
    /// Placeholder for unimplemented nodes.
    Placeholder,
}
