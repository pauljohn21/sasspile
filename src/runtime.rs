use std::fmt;
use std::sync::Arc;
use crate::bus::CompilerBus;
use crate::types::Value;

#[derive(Clone)]
pub struct EvalContext {
    bus: Arc<CompilerBus>,
    scope_id: u64,
}

impl EvalContext {
    pub fn new(bus: Arc<CompilerBus>, scope_id: u64) -> Self {
        Self { bus, scope_id }
    }
    pub fn bus(&self) -> &CompilerBus { &self.bus }
    pub fn scope_id(&self) -> u64 { self.scope_id }
    pub fn child_scope(&self, local_idx: u64) -> Self {
        Self { bus: self.bus.clone(), scope_id: self.scope_id * 1000 + local_idx }
    }
    pub fn var(&self, name: &str) -> Option<Value> { self.bus.get_var(self.scope_id, name) }
    pub fn bind_var(&self, name: &str, value: Value) { self.bus.set_var(self.scope_id, name, value); }
}

impl fmt::Debug for EvalContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("EvalCtx").field("scope_id", &self.scope_id).finish_non_exhaustive()
    }
}

pub fn create_runtime() -> (Arc<EvalContext>, Arc<CompilerBus>) {
    let bus = Arc::new(CompilerBus::new());
    let ctx = Arc::new(EvalContext::new(bus.clone(), 1));
    (ctx, bus)
}
