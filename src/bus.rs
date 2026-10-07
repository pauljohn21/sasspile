use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt;
use std::sync::{Arc, Mutex};
use rxrust::prelude::*;
use crate::types::Value;

#[derive(Debug, Clone)]
pub enum VarEvent {
    Bind { scope_id: u64, name: String, value: Value },
    Update { scope_id: u64, name: String, value: Value },
}

#[derive(Debug, Clone)]
pub enum ModuleEvent {
    Loaded { name: String },
    MemberRegistered { module: String, name: String },
}

#[derive(Debug, Clone)]
pub enum ScopeEvent {
    Open { scope_id: u64, kind: ScopeKind },
    Close { scope_id: u64, kind: ScopeKind },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind { Rule, Media, Supports, Control, Function, Mixin }

#[derive(Debug, Clone)]
pub struct MixinDef {
    pub name: String,
    pub params: Vec<crate::types::Param>,
    pub body: Vec<crate::types::AstNode>,
}

#[derive(Debug, Clone)]
pub struct FnDef {
    pub name: String,
    pub params: Vec<crate::types::Param>,
    pub body: Vec<crate::types::AstNode>,
}

#[derive(Debug, Clone)]
pub struct ModuleDef {
    pub path: String,
    pub members: Vec<String>,
}

#[derive(Default)]
struct BusInner {
    mixins: HashMap<String, MixinDef>,
    functions: HashMap<String, FnDef>,
    modules: HashMap<String, ModuleDef>,
    variables: HashMap<(u64, String), Value>,
    parent_map: HashMap<u64, u64>,
}

#[derive(Clone)]
pub struct CompilerBus {
    var_subject: SharedSubject<'static, VarEvent, Infallible>,
    module_subject: SharedSubject<'static, ModuleEvent, Infallible>,
    scope_subject: SharedSubject<'static, ScopeEvent, Infallible>,
    inner: Arc<Mutex<BusInner>>,
}

impl CompilerBus {
    pub fn new() -> Self {
        Self {
            var_subject: Shared::subject(),
            module_subject: Shared::subject(),
            scope_subject: Shared::subject(),
            inner: Arc::new(Mutex::new(BusInner::default())),
        }
    }

    pub fn var_events(&self) -> SharedSubject<'static, VarEvent, Infallible> {
        self.var_subject.clone()
    }
    pub fn module_events(&self) -> SharedSubject<'static, ModuleEvent, Infallible> {
        self.module_subject.clone()
    }
    pub fn scope_events(&self) -> SharedSubject<'static, ScopeEvent, Infallible> {
        self.scope_subject.clone()
    }
    pub fn set_var(&self, scope_id: u64, name: &str, value: Value) {
        let mut g = self.inner.lock().unwrap();
        let ev = if g.variables.contains_key(&(scope_id, name.to_string())) {
            VarEvent::Update { scope_id, name: name.into(), value: value.clone() }
        } else {
            VarEvent::Bind { scope_id, name: name.into(), value: value.clone() }
        };
        g.variables.insert((scope_id, name.into()), value);
        drop(g);
        let mut subj = self.var_subject.clone();
        subj.next(ev);
    }
    pub fn bind_var_silent(&self, scope_id: u64, name: String, value: Value) {
        let mut g = self.inner.lock().unwrap();
        g.variables.insert((scope_id, name), value);
    }
    pub fn register_parent(&self, child: u64, parent: u64) {
        let mut g = self.inner.lock().unwrap();
        g.parent_map.insert(child, parent);
    }
    pub fn get_var(&self, ctx: &crate::runtime::EvalContext, name: &str) -> Option<Value> {
        let g = self.inner.lock().unwrap();
        // Walk parent chain: scope_id → parent_id → ...
        let mut current_id = Some(ctx.scope_id());
        while let Some(s) = current_id {
            if let Some(v) = g.variables.get(&(s, name.into())) { return Some(v.clone()); }
            current_id = g.parent_map.get(&s).copied();
        }
        None
    }

    /// Look up a variable in a specific scope without walking parent chain.
    pub fn get_var_by_id(&self, scope_id: u64, name: &str) -> Option<Value> {
        let g = self.inner.lock().unwrap();
        g.variables.get(&(scope_id, name.into())).cloned()
    }
    pub fn register_mixin(&self, d: MixinDef) { self.inner.lock().unwrap().mixins.insert(d.name.clone(), d); }
    pub fn lookup_mixin(&self, n: &str) -> Option<MixinDef> { self.inner.lock().unwrap().mixins.get(n).cloned() }
    pub fn register_fn(&self, d: FnDef) { self.inner.lock().unwrap().functions.insert(d.name.clone(), d); }
    pub fn lookup_fn(&self, n: &str) -> Option<FnDef> { self.inner.lock().unwrap().functions.get(n).cloned() }
    pub fn register_module(&self, d: ModuleDef) { self.inner.lock().unwrap().modules.insert(d.path.clone(), d); }
    pub fn lookup_module(&self, p: &str) -> Option<ModuleDef> { self.inner.lock().unwrap().modules.get(p).cloned() }
}

impl Default for CompilerBus { fn default() -> Self { Self::new() } }

impl fmt::Debug for CompilerBus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let i = self.inner.lock().unwrap();
        f.debug_struct("Bus").field("mixins", &i.mixins.len()).field("fns", &i.functions.len()).finish_non_exhaustive()
    }
}
