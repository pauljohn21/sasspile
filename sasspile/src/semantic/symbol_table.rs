//! Symbol table — scope stack with name lookup and shadowing rules.
//!
//! Supports three scope kinds: Global, Local (rule/mixin bodies),
//! and Param (function/mixin parameter scopes). Implements standard
//! lexical scoping with variable shadowing.

use std::collections::HashMap;

use crate::source::SourceSpan;
use crate::value::Value;

/// Scope kinds determine shadowing and visibility rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    /// Global scope — top-level definitions visible everywhere.
    Global,
    /// Local scope — within rule blocks or mixin bodies.
    Local,
    /// Parameter scope — function/mixin parameters and local variables.
    Param,
}

/// A single scope level in the stack.
#[derive(Debug, Clone)]
pub struct Scope {
    /// The kind of scope.
    pub kind: ScopeKind,
    /// Variable bindings in this scope.
    pub bindings: HashMap<String, SymbolEntry>,
}

/// Metadata for a single symbol binding.
#[derive(Debug, Clone)]
pub struct SymbolEntry {
    /// The current value (if resolved).
    pub value: Option<Value>,
    /// Whether the variable can be reassigned.
    pub is_mutable: bool,
    /// Source location of definition.
    pub defined_at: SourceSpan,
}

impl SymbolEntry {
    /// Create a new immutable entry.
    pub fn new(value: Option<Value>, defined_at: SourceSpan) -> Self {
        Self {
            value,
            is_mutable: false,
            defined_at,
        }
    }

    /// Create a new mutable entry (for variables that may be reassigned).
    pub fn mutable(value: Option<Value>, defined_at: SourceSpan) -> Self {
        Self {
            value,
            is_mutable: true,
            defined_at,
        }
    }
}

impl Scope {
    /// Create an empty scope of the given kind.
    pub fn new(kind: ScopeKind) -> Self {
        Self {
            kind,
            bindings: HashMap::new(),
        }
    }

    /// Create a global scope.
    pub fn global() -> Self {
        Self::new(ScopeKind::Global)
    }

    /// Insert a binding, returning the previous entry if any.
    pub fn define(&mut self, name: String, entry: SymbolEntry) -> Option<SymbolEntry> {
        self.bindings.insert(name, entry)
    }

    /// Look up a binding in this scope only.
    pub fn lookup(&self, name: &str) -> Option<&SymbolEntry> {
        self.bindings.get(name)
    }

    /// Look up a binding mutably.
    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut SymbolEntry> {
        self.bindings.get_mut(name)
    }

    /// Check if a name is defined in this scope.
    pub fn contains(&self, name: &str) -> bool {
        self.bindings.contains_key(name)
    }
}

/// Scope stack for lexical name resolution.
#[derive(Debug, Clone)]
pub struct SymbolTable {
    /// Stack of scopes; the last is the current innermost scope.
    scopes: Vec<Scope>,
}

impl SymbolTable {
    /// Create a new symbol table with a global scope.
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::global()],
        }
    }

    /// Push a new local scope.
    pub fn push_local(&mut self) {
        self.scopes.push(Scope::new(ScopeKind::Local));
    }

    /// Push a new parameter scope (for function/mixin bodies).
    pub fn push_param(&mut self) {
        self.scopes.push(Scope::new(ScopeKind::Param));
    }

    /// Pop the current scope (panics if at global).
    pub fn pop(&mut self) -> Scope {
        assert!(
            self.scopes.len() > 1,
            "cannot pop the global scope"
        );
        self.scopes.pop().expect("scope stack underflow")
    }

    /// Look up a name through the scope stack (innermost first).
    pub fn lookup(&self, name: &str) -> Option<&SymbolEntry> {
        self.scopes.iter().rev().find_map(|s| s.lookup(name))
    }

    /// Look up an entry mutably in the current (innermost) scope.
    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut SymbolEntry> {
        self.scopes.last_mut()?.lookup_mut(name)
    }

    /// Look up a name mutably (only in current scope).
    pub fn lookup_current(&mut self, name: &str) -> Option<&mut SymbolEntry> {
        self.scopes.last_mut()?.lookup_mut(name)
    }

    /// Define a binding in the current (innermost) scope.
    pub fn define_current(&mut self, name: String, entry: SymbolEntry) -> Option<SymbolEntry> {
        self.scopes
            .last_mut()
            .expect("must have at least one scope")
            .define(name, entry)
    }

    /// Check if a name is already defined in the current scope.
    pub fn is_defined_in_current(&self, name: &str) -> bool {
        self.scopes
            .last()
            .map(|s| s.contains(name))
            .unwrap_or(false)
    }

    /// Get the current scope kind.
    pub fn current_kind(&self) -> ScopeKind {
        self.scopes
            .last()
            .map(|s| s.kind)
            .unwrap_or(ScopeKind::Global)
    }

    /// Depth of the scope stack (1 = global only).
    pub fn depth(&self) -> usize {
        self.scopes.len()
    }

    /// Check if we're at global scope only.
    pub fn is_global(&self) -> bool {
        self.scopes.len() == 1
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
