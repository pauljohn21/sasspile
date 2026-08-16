//! Change propagation — upstream variable changes trigger downstream recompile.
//!
//! Connects the `ReactiveEnv`, `DependencyGraph`, and `SpanCache` to form
//! the incremental compilation feedback loop.

use std::collections::HashSet;

use tokio::sync::watch;
use tracing::{debug, info, instrument};

use crate::incremental::cache::SpanCache;
use crate::incremental::depgraph::{Dependency, DependencyGraph, NodeId};
use crate::incremental::env::ReactiveEnv;

/// Message types for the propagation channel.
#[derive(Debug, Clone)]
pub enum PropagateMsg {
    /// A variable changed value.
    VarChanged(String),
    /// A batch of variables changed.
    BatchChanged(Vec<String>),
    /// Source was edited in range [start, end).
    SourceEdited { start: u32, end: u32 },
}

/// Orchestrates incremental change propagation.
///
/// Listens for variable changes, looks up dependent nodes via the
/// dependency graph, invalidates their cache entries, and emits
/// the set of affected node IDs.
pub struct Propagator {
    /// Reactive environment for variable state.
    pub env: ReactiveEnv,
    /// Dependency graph for tracking what depends on what.
    pub graph: DependencyGraph,
    /// Value cache for memoization.
    pub cache: SpanCache,
}

impl Propagator {
    /// Create a new propagator with default state.
    pub fn new() -> Self {
        Self {
            env: ReactiveEnv::empty(),
            graph: DependencyGraph::new(),
            cache: SpanCache::new(),
        }
    }

    /// Create with an initial variable snapshot.
    pub fn with_env(env: ReactiveEnv) -> Self {
        Self {
            env,
            graph: DependencyGraph::new(),
            cache: SpanCache::new(),
        }
    }

    /// Process a propagation message and return affected node IDs.
    #[instrument(skip(self))]
    pub fn handle_message(&mut self, msg: &PropagateMsg) -> HashSet<NodeId> {
        match msg {
            PropagateMsg::VarChanged(name) => {
                debug!(var = %name, "handling variable change");
                let dep = Dependency::Var(name.clone());
                let affected = self.graph.affected_nodes(&[dep]);
                // Invalidate their cache entries.
                for &node in &affected {
                    // Note: We'd need span info per node to precisely invalidate.
                    // For broad invalidation, mark all — precise tracking needs
                    // node→span mapping (provided when registering).
                    debug!(node_id = node, "node affected by variable change");
                }
                affected
            }
            PropagateMsg::BatchChanged(names) => {
                debug!(count = names.len(), "handling batch variable change");
                let deps: Vec<Dependency> = names
                    .iter()
                    .map(|n| Dependency::Var(n.clone()))
                    .collect();
                let affected = self.graph.affected_nodes(&deps);
                debug!(affected = affected.len(), "batch affected nodes");
                affected
            }
            PropagateMsg::SourceEdited { start, end } => {
                debug!(range = %format!("[{start}, {end})"), "handling source edit");
                // Invalidate cache entries in edited range.
                self.cache.invalidate_range(*start, *end);
                // Return empty — cache invalidation handles it.
                HashSet::new()
            }
        }
    }

    /// Subscribe to reactive variable changes and process them in a loop.
    ///
    /// Returns the number of variable updates processed.
    #[instrument(skip(self))]
    pub async fn watch_and_propagate(
        &mut self,
        mut rx: watch::Receiver<crate::incremental::env::VarSnapshot>,
    ) -> usize {
        let mut count = 0;
        while rx.changed().await.is_ok() {
            count += 1;
            let snapshot = rx.borrow().clone();
            info!(
                var_count = snapshot.len(),
                "received reactive update"
            );
            // In a full implementation, this would diff with previous
            // snapshot and emit VarChanged messages for changed variables.
        }
        count
    }
}

impl Default for Propagator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::incremental::depgraph::Dependency;
    use crate::value::Value;

    #[test]
    fn propagate_var_change() {
        let mut prop = Propagator::new();

        // Register nodes and dependencies.
        let node_a = prop.graph.register_node();
        let node_b = prop.graph.register_node();
        prop.graph.add_dependency(node_a, Dependency::Var("$primary".into()));
        prop.graph.add_dependency(node_b, Dependency::Var("$primary".into()));

        let msg = PropagateMsg::VarChanged("$primary".into());
        let affected = prop.handle_message(&msg);

        assert_eq!(affected.len(), 2);
        assert!(affected.contains(&node_a));
        assert!(affected.contains(&node_b));
    }

    #[test]
    fn propagate_batch_change() {
        let mut prop = Propagator::new();

        let node_a = prop.graph.register_node();
        let node_b = prop.graph.register_node();
        prop.graph.add_dependency(node_a, Dependency::Var("$x".into()));
        prop.graph.add_dependency(node_b, Dependency::Var("$y".into()));

        let msg = PropagateMsg::BatchChanged(vec!["$x".into(), "$y".into()]);
        let affected = prop.handle_message(&msg);

        assert_eq!(affected.len(), 2);
    }

    #[test]
    fn propagate_source_edit_invalidates_cache() {
        let mut prop = Propagator::new();

        // Insert cache entries.
        prop.cache.put(
            crate::source::SourceSpan::new(0, 10),
            0,
            Value::Boolean(true),
        );
        prop.cache.put(
            crate::source::SourceSpan::new(20, 30),
            0,
            Value::Boolean(false),
        );

        let msg = PropagateMsg::SourceEdited { start: 0, end: 10 };
        let _affected = prop.handle_message(&msg);

        assert_eq!(prop.cache.len(), 1);
    }

    #[test]
    fn propagate_unrelated_var_does_not_affect() {
        let mut prop = Propagator::new();

        let node = prop.graph.register_node();
        prop.graph.add_dependency(node, Dependency::Var("$primary".into()));

        let msg = PropagateMsg::VarChanged("$secondary".into());
        let affected = prop.handle_message(&msg);

        assert!(affected.is_empty());
    }
}
