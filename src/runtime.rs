use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use crate::bus::CompilerBus;
use crate::types::Value;

/// Monotonic scope ID generator (global, lives in EvalContext via Arc).
#[derive(Clone)]
struct ScopeCounter {
    next: Arc<AtomicU64>,
}

impl ScopeCounter {
    fn new(start: u64) -> Self { Self { next: Arc::new(AtomicU64::new(start)) } }
    fn allocate(&self) -> u64 { self.next.fetch_add(1, Ordering::SeqCst) }
}

#[derive(Clone)]
pub struct EvalContext {
    bus: Arc<CompilerBus>,
    scope_id: u64,
    counter: ScopeCounter,
    /// Captured caller scope for @content expansion inside mixins.
    /// When emitting @content nodes, this scope is used instead of the mixin's scope.
    pub content_scope: Option<Arc<EvalContext>>,
    /// Return value slot for @function/@return.
    /// Set by @return inside function body (including nested @while/@for/@each),
    /// read by eval_function_body_opt after loop execution.
    return_value: Arc<Mutex<Option<Value>>>,
}

impl EvalContext {
    pub fn new(bus: Arc<CompilerBus>, scope_id: u64) -> Self {
        Self { bus, scope_id, counter: ScopeCounter::new(scope_id + 1), content_scope: None, return_value: Arc::new(Mutex::new(None)) }
    }
    pub fn bus(&self) -> &CompilerBus { &self.bus }
    pub fn scope_id(&self) -> u64 { self.scope_id }
    pub fn child_scope(&self, _local_idx: u64) -> Self {
        let new_id = self.counter.allocate();
        // Register parent mapping in bus so get_var can chain
        self.bus.register_parent(new_id, self.scope_id);
        // Note: content_scope is NOT propagated — each mixin call sets its own
        // return_value is NOT propagated — each function call has its own slot
        Self { bus: self.bus.clone(), scope_id: new_id, counter: self.counter.clone(), content_scope: None, return_value: Arc::new(Mutex::new(None)) }
    }
    pub fn var(&self, name: &str) -> Option<Value> { self.bus.get_var(self, name) }
    pub fn bind_var(&self, name: &str, value: Value) { self.bus.set_var(self.scope_id, name, value); }
    /// Set the return value for this function call scope.
    pub fn set_return_value(&self, value: Value) {
        *self.return_value.lock().unwrap() = Some(value);
    }
    /// Get the return value if @return was hit.
    pub fn get_return_value(&self) -> Option<Value> {
        self.return_value.lock().unwrap().clone()
    }
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
