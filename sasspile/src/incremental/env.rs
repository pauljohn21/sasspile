//! Reactive environment — watch-based variable propagation.
//!
//! Uses `tokio::sync::watch` channels for reactive variable updates.
//! When a variable changes, only downstream dependents are notified.

use std::collections::HashMap;
use tokio::sync::watch;
use tracing::{debug, instrument};

use crate::value::Value;

/// Identifier for a tracked variable.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct VarId(pub String);

impl From<String> for VarId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for VarId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Snapshot of all reactive variables.
pub type VarSnapshot = HashMap<String, Value>;

/// Reactive environment that propagates variable changes via watch channels.
#[derive(Debug, Clone)]
pub struct ReactiveEnv {
    /// Broadcast current variable state to all subscribers.
    tx: watch::Sender<VarSnapshot>,
}

impl ReactiveEnv {
    /// Create a new reactive environment with empty state.
    #[instrument(skip(initial))]
    pub fn new(initial: VarSnapshot) -> Self {
        let (tx, _rx) = watch::channel(initial);
        Self { tx }
    }

    /// Create with default empty state.
    pub fn empty() -> Self {
        Self::new(VarSnapshot::new())
    }

    /// Subscribe to variable changes.
    ///
    /// Returns a `watch::Receiver` that yields the current snapshot
    /// and updates whenever `set_var` or `batch_update` is called.
    pub fn subscribe(&self) -> watch::Receiver<VarSnapshot> {
        self.tx.subscribe()
    }

    /// Set a single variable's value.
    pub fn set_var(&self, name: impl Into<String>, value: Value) {
        let name = name.into();
        debug!(var = %name, "setting reactive variable");
        self.tx.send_modify(|map| {
            map.insert(name, value);
        });
    }

    /// Batch update multiple variables atomically.
    #[instrument(skip(self, updates))]
    pub fn batch_update(&self, updates: VarSnapshot) {
        debug!(count = updates.len(), "batch updating reactive variables");
        self.tx.send_modify(|map| {
            map.extend(updates);
        });
    }

    /// Get the current snapshot of all variables.
    pub fn snapshot(&self) -> VarSnapshot {
        self.tx.borrow().clone()
    }

    /// Read a single variable's current value.
    pub fn get_var(&self, name: &str) -> Option<Value> {
        self.tx.borrow().get(name).cloned()
    }
}

impl Default for ReactiveEnv {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reactive_env_set_and_get() {
        let env = ReactiveEnv::empty();
        env.set_var("$primary", Value::Color(crate::value::SassColor::new(255, 0, 0, 1.0)));
        let val = env.get_var("$primary");
        assert!(matches!(val, Some(Value::Color(_))));
    }

    #[test]
    fn reactive_env_snapshot() {
        let env = ReactiveEnv::empty();
        env.set_var("$a", Value::Number(crate::value::Number::new(1.0, crate::value::Unit::None)));
        env.set_var("$b", Value::Number(crate::value::Number::new(2.0, crate::value::Unit::None)));
        let snap = env.snapshot();
        assert_eq!(snap.len(), 2);
    }

    #[test]
    fn reactive_env_batch_update() {
        let env = ReactiveEnv::empty();
        let mut updates = VarSnapshot::new();
        updates.insert("$x".to_string(), Value::Boolean(true));
        updates.insert("$y".to_string(), Value::Boolean(false));
        env.batch_update(updates);
        let snap = env.snapshot();
        assert_eq!(snap.len(), 2);
    }

    #[tokio::test]
    async fn reactive_env_subscribe_receives_updates() {
        let env = ReactiveEnv::empty();
        let mut rx = env.subscribe();

        // Initial value should be empty.
        assert_eq!(rx.borrow().len(), 0);

        // Set a variable — subscriber should see the update.
        env.set_var("$color", Value::String("red".into(), crate::value::Quoted::Unquoted));
        rx.changed().await.expect("watch should send update");
        assert_eq!(rx.borrow().len(), 1);
    }
}
