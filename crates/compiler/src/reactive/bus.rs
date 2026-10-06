//! CompilerBus — multicast event bus for reactive compilation.
//!
//! Provides three multicast channels (all `SharedSubject` for Send + Sync):
//! - `var_events`: Value bind/update notifications
//! - `module_events`: Module load and member registration
//! - `css_scope_subject`: CSS scope open/close boundaries
//!
//! Mixin and function registries are stored in `Arc<Mutex<HashMap>>` so they
//! can be shared across threads. Registrations are also broadcast via
//! `SharedSubject` so any subscriber can react to new definitions appearing.

use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt;
use std::sync::{Arc, Mutex};

use rxrust::prelude::*;

use crate::reactive::AstNode;

/// Events emitted when a Sass variable is bound or updated.
#[derive(Debug, Clone)]
pub enum ValueEvent {
    /// A new variable binding: scope_id, name, value
    Bind { scope_id: u64, name: String, value: u64 },
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

/// Inner shared state for registries (thread-safe via Arc<Mutex>).
#[derive(Default)]
struct BusInner {
    mixins: HashMap<String, MixinDef>,
    functions: HashMap<String, FnDef>,
}

/// Multicast event bus shared across all pipeline stages.
///
/// Uses `SharedSubject` for all event channels so that events can be
/// broadcast across thread boundaries. Registries use `Arc<Mutex<HashMap>>`
/// for thread-safe access.
#[derive(Clone)]
pub struct CompilerBus {
    var_events: SharedSubject<'static, ValueEvent, Infallible>,
    module_events: SharedSubject<'static, ModuleEvent, Infallible>,
    css_scope_subject: SharedSubject<'static, ScopeEvent, Infallible>,
    inner: Arc<Mutex<BusInner>>,
}

impl CompilerBus {
    /// Create a new `CompilerBus` with fresh multicast subjects.
    pub fn new() -> Self {
        Self {
            var_events: Shared::subject(),
            module_events: Shared::subject(),
            css_scope_subject: Shared::subject(),
            inner: Arc::new(Mutex::new(BusInner::default())),
        }
    }

    /// Subscribe to variable bind events.
    pub fn var_events(&self) -> SharedSubject<'static, ValueEvent, Infallible> {
        self.var_events.clone()
    }

    /// Subscribe to module events.
    pub fn module_events(&self) -> SharedSubject<'static, ModuleEvent, Infallible> {
        self.module_events.clone()
    }

    /// Subscribe to CSS scope events.
    pub fn css_scope(&self) -> SharedSubject<'static, ScopeEvent, Infallible> {
        self.css_scope_subject.clone()
    }

    /// Register a mixin definition (thread-safe).
    pub fn register_mixin(&self, def: MixinDef) {
        self.inner.lock().unwrap().mixins.insert(def.name.clone(), def);
    }

    /// Look up a mixin by name (thread-safe).
    pub fn lookup_mixin(&self, name: &str) -> Option<MixinDef> {
        self.inner.lock().unwrap().mixins.get(name).cloned()
    }

    /// Register a function definition (thread-safe).
    pub fn register_fn(&self, def: FnDef) {
        self.inner.lock().unwrap().functions.insert(def.name.clone(), def);
    }

    /// Look up a function by name (thread-safe).
    pub fn lookup_fn(&self, name: &str) -> Option<FnDef> {
        self.inner.lock().unwrap().functions.get(name).cloned()
    }
}

impl fmt::Debug for CompilerBus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = self.inner.lock().unwrap();
        f.debug_struct("CompilerBus")
            .field("mixins", &inner.mixins.len())
            .field("functions", &inner.functions.len())
            .finish_non_exhaustive()
    }
}

impl Default for CompilerBus {
    fn default() -> Self {
        Self::new()
    }
}
