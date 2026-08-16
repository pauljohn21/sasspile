//! Dependency graph — tracks variable/mixin/function dependencies.
//!
//! Maps each compiled node to its dependencies so that when a variable changes,
//! only affected nodes are re-evaluated.

use std::collections::{HashMap, HashSet};

use tracing::{debug, instrument};

/// Node identifier in the dependency graph.
pub type NodeId = u64;

/// A dependency edge from a node to a variable it references.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Dependency {
    /// Depends on a variable.
    Var(String),
    /// Depends on a mixin definition.
    Mixin(String),
    /// Depends on a function definition.
    Function(String),
}

/// Bidirectional dependency graph for incremental invalidation.
///
/// Tracks:
/// * `node_deps`: NodeId → set of dependencies it reads
/// * `dep_nodes`: Dependency → set of nodes that depend on it
///
/// When a variable changes, `dependents_of` returns the nodes to recompile.
#[derive(Debug, Default)]
pub struct DependencyGraph {
    /// Forward edges: node → its dependencies.
    node_deps: HashMap<NodeId, HashSet<Dependency>>,
    /// Reverse edges: dependency → nodes that reference it.
    dep_nodes: HashMap<Dependency, HashSet<NodeId>>,
    /// Monotonic counter for node IDs.
    next_id: u64,
}

impl DependencyGraph {
    /// Create a new empty dependency graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new node and return its NodeId.
    #[instrument(skip(self))]
    pub fn register_node(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        self.node_deps.insert(id, HashSet::new());
        debug!(node_id = id, "registered dependency node");
        id
    }

    /// Register a dependency for a node.
    #[instrument(skip(self))]
    pub fn add_dependency(&mut self, node: NodeId, dep: Dependency) {
        if let Some(deps) = self.node_deps.get_mut(&node) {
            deps.insert(dep.clone());
        }
        self.dep_nodes
            .entry(dep.clone())
            .or_default()
            .insert(node);
        debug!(node_id = node, ?dep, "added dependency edge");
    }

    /// Register multiple dependencies at once.
    pub fn add_dependencies(&mut self, node: NodeId, deps: impl IntoIterator<Item = Dependency>) {
        for dep in deps {
            self.add_dependency(node, dep);
        }
    }

    /// Get all nodes that depend on a given dependency.
    pub fn dependents_of(&self, dep: &Dependency) -> Vec<NodeId> {
        self.dep_nodes
            .get(dep)
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Get all dependencies of a node.
    pub fn dependencies_of(&self, node: NodeId) -> Vec<&Dependency> {
        self.node_deps
            .get(&node)
            .map(|set| set.iter().collect())
            .unwrap_or_default()
    }

    /// Remove a node and all its edges from the graph.
    #[instrument(skip(self))]
    pub fn remove_node(&mut self, node: NodeId) {
        if let Some(deps) = self.node_deps.remove(&node) {
            for dep in &deps {
                if let Some(nodes) = self.dep_nodes.get_mut(dep) {
                    nodes.remove(&node);
                    if nodes.is_empty() {
                        self.dep_nodes.remove(dep);
                    }
                }
            }
        }
        debug!(node_id = node, "removed node from dependency graph");
    }

    /// Find all nodes affected by a set of changed dependencies (transitive).
    #[instrument(skip(self))]
    pub fn affected_nodes(&self, changed_deps: &[Dependency]) -> HashSet<NodeId> {
        let mut affected = HashSet::new();
        for dep in changed_deps {
            for &node in self.dep_nodes.get(dep).unwrap_or(&HashSet::new()) {
                affected.insert(node);
            }
        }
        debug!(
            changed = changed_deps.len(),
            affected = affected.len(),
            "computed affected nodes"
        );
        affected
    }

    /// Total number of registered nodes.
    pub fn node_count(&self) -> usize {
        self.node_deps.len()
    }

    /// Total number of tracked dependencies.
    pub fn dep_count(&self) -> usize {
        self.dep_nodes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depgraph_basic() {
        let mut graph = DependencyGraph::new();
        let node_a = graph.register_node();
        let node_b = graph.register_node();

        graph.add_dependency(node_a, Dependency::Var("$color".into()));
        graph.add_dependency(node_b, Dependency::Var("$color".into()));
        graph.add_dependency(node_b, Dependency::Mixin("button".into()));

        let color_deps = graph.dependents_of(&Dependency::Var("$color".into()));
        assert_eq!(color_deps.len(), 2);
        assert!(color_deps.contains(&node_a));
        assert!(color_deps.contains(&node_b));
    }

    #[test]
    fn depgraph_remove_node() {
        let mut graph = DependencyGraph::new();
        let node = graph.register_node();
        graph.add_dependency(node, Dependency::Var("$x".into()));

        assert_eq!(graph.node_count(), 1);
        graph.remove_node(node);
        assert_eq!(graph.node_count(), 0);

        let deps = graph.dependents_of(&Dependency::Var("$x".into()));
        assert!(deps.is_empty());
    }

    #[test]
    fn depgraph_affected_nodes() {
        let mut graph = DependencyGraph::new();
        let a = graph.register_node();
        let b = graph.register_node();
        let c = graph.register_node();

        graph.add_dependency(a, Dependency::Var("$primary".into()));
        graph.add_dependency(b, Dependency::Var("$secondary".into()));
        graph.add_dependency(c, Dependency::Var("$primary".into()));

        let changed = vec![Dependency::Var("$primary".into())];
        let affected = graph.affected_nodes(&changed);
        assert_eq!(affected.len(), 2);
        assert!(affected.contains(&a));
        assert!(affected.contains(&c));
        assert!(!affected.contains(&b));
    }

    #[test]
    fn depgraph_dependencies_of() {
        let mut graph = DependencyGraph::new();
        let node = graph.register_node();
        graph.add_dependencies(
            node,
            vec![
                Dependency::Var("$a".into()),
                Dependency::Mixin("foo".into()),
            ],
        );

        let deps = graph.dependencies_of(node);
        assert_eq!(deps.len(), 2);
    }
}
