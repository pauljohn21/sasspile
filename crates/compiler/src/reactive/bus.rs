//! CompilerBus — multicast event bus for reactive compilation.
//!
//! Provides three multicast channels:
//! - `var_events`: Value bind/update notifications
//! - `module_events`: Module load and member registration
//! - `css_scope_subject`: CSS scope open/close boundaries
//!
//! Plus shared registries (Rc<RefCell<>>) for mixins and functions.

use rxrust::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt;
use std::rc::Rc;

use crate::reactive::AstNode;

/// Events emitted when a Sass variable is bound or updated.
#[derive(Debug, Clone)]
pub enum ValueEvent {
    /// A new variable binding: scope_id, name, value
    Bind {
        scope_id: u64,
        name: String,
        value: u64,
    },
}

/// Events emitted when modules are loaded or members registered.
#[derive(Debug, Clone)]
pub enum ModuleEvent {
    /// A module has been loaded by name.
    Load { name: String },
}

/// Events emitted to mark CSS scope boundaries.
#[derive(Debug, Clone)]
pub enum ScopeEvent {
    /// Enter a new scope.
    Open { scope_id: u64 },
    /// Exit the current scope.
    Close { scope_id: u64 },
}

/// A registered mixin definition.
#[derive(Debug, Clone)]
pub struct MixinDef {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<AstNode>,
}

/// A registered function definition.
#[derive(Debug, Clone)]
pub struct FnDef {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<AstNode>,
}

/// Inner shared state for registries.
#[derive(Default)]
struct BusInner {
    mixins: HashMap<String, MixinDef>,
    functions: HashMap<String, FnDef>,
}

/// Multicast event bus shared across all pipeline stages.
#[derive(Clone)]
pub struct CompilerBus {
    var_events: LocalSubject<'static, ValueEvent, Infallible>,
    module_events: LocalSubject<'static, ModuleEvent, Infallible>,
    css_scope_subject: LocalSubject<'static, ScopeEvent, Infallible>,
    inner: Rc<RefCell<BusInner>>,
}

impl CompilerBus {
    /// Create a new `CompilerBus` with fresh multicast subjects.
    pub fn new() -> Self {
        Self {
            var_events: Local::subject(),
            module_events: Local::subject(),
            css_scope_subject: Local::subject(),
            inner: Rc::new(RefCell::new(BusInner::default())),
        }
    }

    /// Subscribe to variable bind events.
    pub fn var_events(&self) -> LocalSubject<'static, ValueEvent, Infallible> {
        self.var_events.clone()
    }

    /// Subscribe to module events.
    pub fn module_events(&self) -> LocalSubject<'static, ModuleEvent, Infallible> {
        self.module_events.clone()
    }

    /// Subscribe to CSS scope events.
    pub fn css_scope(&self) -> LocalSubject<'static, ScopeEvent, Infallible> {
        self.css_scope_subject.clone()
    }

    /// Register a mixin definition.
    pub fn register_mixin(&self, def: MixinDef) {
        self.inner.borrow_mut().mixins.insert(def.name.clone(), def);
    }

    /// Look up a mixin by name.
    pub fn lookup_mixin(&self, name: &str) -> Option<MixinDef> {
        self.inner.borrow().mixins.get(name).cloned()
    }

    /// Register a function definition.
    pub fn register_fn(&self, def: FnDef) {
        self.inner.borrow_mut().functions.insert(def.name.clone(), def);
    }

    /// Look up a function by name.
    pub fn lookup_fn(&self, name: &str) -> Option<FnDef> {
        self.inner.borrow().functions.get(name).cloned()
    }
}

impl fmt::Debug for CompilerBus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompilerBus")
            .field("mixins", &self.inner.borrow().mixins.len())
            .field("functions", &self.inner.borrow().functions.len())
            .finish_non_exhaustive()
    }
}

impl Default for CompilerBus {
    fn default() -> Self {
        Self::new()
    }
}
