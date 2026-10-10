use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use crate::types::Value;

// Note: VarEvent/ModuleEvent/ScopeEvent and SharedSubject were removed because
// they were unused in production — nobody subscribed to these events. The event
// emission in set_var() was cloning value + name for every variable bind
// (thousands of times during Bootstrap compilation) with zero consumers.
// This was a GC thinking anti-pattern: creating reactive infrastructure "just in case".

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
    inner: Arc<Mutex<BusInner>>,
}

impl CompilerBus {
    pub fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(BusInner::default())) }
    }

    pub fn set_var(&self, scope_id: u64, name: &str, value: Value) {
        let mut g = self.inner.lock().unwrap();
        g.variables.insert((scope_id, name.into()), value);
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
    pub fn register_mixin(&self, d: MixinDef) { tracing::trace!(mixin_name = %d.name, "register_mixin"); self.inner.lock().unwrap().mixins.insert(d.name.clone(), d); }
    pub fn lookup_mixin(&self, n: &str) -> Option<MixinDef> { let r = self.inner.lock().unwrap().mixins.get(n).cloned(); tracing::trace!(mixin_name = %n, found = r.is_some(), "lookup_mixin"); r }
    pub fn register_fn(&self, d: FnDef) { tracing::trace!(fn_name = %d.name, "register_fn"); self.inner.lock().unwrap().functions.insert(d.name.clone(), d); }
    pub fn lookup_fn(&self, n: &str) -> Option<FnDef> { let r = self.inner.lock().unwrap().functions.get(n).cloned(); tracing::trace!(fn_name = %n, found = r.is_some(), "lookup_fn"); r }
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
