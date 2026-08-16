//! Module resolution — @use/@forward dependency graph with cycle detection.
//!
//! Builds a directed graph of module dependencies and detects cycles
//! at semantic analysis time (before evaluation).

use std::collections::{HashMap, HashSet};

use crate::source::SourceSpan;

/// Represents a single module in the dependency graph.
#[derive(Debug, Clone)]
pub struct Module {
    /// Canonical module URL or name.
    pub url: String,
    /// Optional namespace alias.
    pub namespace: Option<String>,
    /// Modules this module depends on (via @use or @forward).
    pub dependencies: Vec<String>,
    /// Where this module was first referenced.
    pub referenced_at: SourceSpan,
    /// Whether the module has been fully loaded.
    pub loaded: bool,
}

impl Module {
    /// Create a new module entry.
    pub fn new(url: String, referenced_at: SourceSpan) -> Self {
        Self {
            url,
            namespace: None,
            dependencies: Vec::new(),
            referenced_at,
            loaded: false,
        }
    }

    /// Add a dependency if not already present.
    pub fn add_dependency(&mut self, dep_url: String) {
        if !self.dependencies.contains(&dep_url) {
            self.dependencies.push(dep_url);
        }
    }
}

/// Dependency graph for module resolution and cycle detection.
#[derive(Debug, Clone)]
pub struct ModuleGraph {
    /// Map from module URL to module definition.
    modules: HashMap<String, Module>,
    /// Reverse index: URL -> URLs that depend on it.
    reverse_deps: HashMap<String, Vec<String>>,
}

/// Result of a cycle check.
#[derive(Debug, Clone)]
pub enum CycleCheck {
    /// No cycles detected.
    Clean,
    /// Cycle found — contains the offending path.
    Cycle(Vec<String>),
}

impl ModuleGraph {
    /// Create an empty module graph.
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            reverse_deps: HashMap::new(),
        }
    }

    /// Register a module if not already present. Returns the module ID.
    pub fn register(&mut self, url: String, span: SourceSpan) -> String {
        if !self.modules.contains_key(&url) {
            let module = Module::new(url.clone(), span);
            self.modules.insert(url.clone(), module);
            self.reverse_deps
                .entry(url.clone())
                .or_default();
        }
        url
    }

    /// Add a dependency edge: `from` depends on `to`.
    pub fn add_dependency(&mut self, from: &str, to: &str) {
        if let Some(module) = self.modules.get_mut(from) {
            module.add_dependency(to.to_string());
        }
        self.reverse_deps
            .entry(to.to_string())
            .or_default()
            .push(from.to_string());
    }

    /// Get a module by URL.
    pub fn get(&self, url: &str) -> Option<&Module> {
        self.modules.get(url)
    }

    /// Get a module mutably.
    pub fn get_mut(&mut self, url: &str) -> Option<&mut Module> {
        self.modules.get_mut(url)
    }

    /// Get modules that depend on the given URL.
    pub fn reverse_dependencies(&self, url: &str) -> &[String] {
        self.reverse_deps
            .get(url)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Check for cycles in the graph starting from a given module.
    pub fn check_cycles_from(&self, start: &str) -> CycleCheck {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        self.dfs_check(start, &mut visited, &mut stack)
    }

    /// Check for cycles across the entire graph.
    pub fn check_all_cycles(&self) -> CycleCheck {
        let mut visited = HashSet::new();
        for url in self.modules.keys() {
            let mut stack = Vec::new();
            match self.dfs_check(url, &mut visited, &mut stack) {
                CycleCheck::Cycle(path) => return CycleCheck::Cycle(path),
                CycleCheck::Clean => {}
            }
        }
        CycleCheck::Clean
    }

    /// Depth-first search for cycle detection.
    fn dfs_check(
        &self,
        current: &str,
        visited: &mut HashSet<String>,
        stack: &mut Vec<String>,
    ) -> CycleCheck {
        if stack.contains(&current.to_string()) {
            // Found cycle — extract the cycle path.
            let cycle_start =
                stack.iter().position(|s| s == current).unwrap_or(0);
            let mut cycle = stack[cycle_start..].to_vec();
            cycle.push(current.to_string());
            return CycleCheck::Cycle(cycle);
        }

        if visited.contains(current) {
            return CycleCheck::Clean;
        }

        visited.insert(current.to_string());
        stack.push(current.to_string());

        if let Some(module) = self.modules.get(current) {
            for dep in &module.dependencies {
                if self.modules.contains_key(dep) {
                    match self.dfs_check(dep, visited, stack) {
                        CycleCheck::Cycle(path) => return CycleCheck::Cycle(path),
                        CycleCheck::Clean => {}
                    }
                }
            }
        }

        stack.pop();
        CycleCheck::Clean
    }

    /// Topological sort of modules (returns error if cycles exist).
    pub fn topological_sort(&self) -> Result<Vec<String>, Vec<String>> {
        match self.check_all_cycles() {
            CycleCheck::Cycle(path) => return Err(path),
            CycleCheck::Clean => {}
        }

        let mut result = Vec::new();
        let mut visited = HashSet::new();

        for url in self.modules.keys() {
            self.topo_dfs(url, &mut visited, &mut result);
        }

        // Post-order DFS naturally yields dependencies-first order.
        Ok(result)
    }

    fn topo_dfs(
        &self,
        current: &str,
        visited: &mut HashSet<String>,
        result: &mut Vec<String>,
    ) {
        if visited.contains(current) {
            return;
        }
        visited.insert(current.to_string());

        if let Some(module) = self.modules.get(current) {
            for dep in &module.dependencies {
                if self.modules.contains_key(dep) {
                    self.topo_dfs(dep, visited, result);
                }
            }
        }

        result.push(current.to_string());
    }

    /// Number of registered modules.
    pub fn len(&self) -> usize {
        self.modules.len()
    }

    /// Whether the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }
}

impl Default for ModuleGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Namespace resolution for loaded modules.
#[derive(Debug, Clone, Default)]
pub struct NamespaceRegistry {
    /// Map from namespace to module URL.
    namespaces: HashMap<String, String>,
}

impl NamespaceRegistry {
    /// Create an empty namespace registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a namespace -> URL mapping.
    pub fn register(&mut self, ns: String, url: String) {
        self.namespaces.insert(ns, url);
    }

    /// Resolve a namespace to its module URL.
    pub fn resolve(&self, ns: &str) -> Option<&String> {
        self.namespaces.get(ns)
    }

    /// Get the underlying map.
    pub fn map(&self) -> &HashMap<String, String> {
        &self.namespaces
    }
}
