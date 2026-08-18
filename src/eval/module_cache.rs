//! Module evaluation cache — stores evaluated module results so that
//! repeated `@use` of the same module doesn't re-evaluate or re-emit CSS.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::env::{Mixin, ModuleEnv, UserFunction};
use crate::value::Value;

/// An evaluated module's public members + CSS output.
///
/// Stored in `ModuleCache` after the first `@use` of a given file path.
/// Subsequent `@use` of the same path reuse this cached result.
#[derive(Clone)]
pub struct EvaluatedModule {
    /// Public variables (non-private, i.e. names not starting with `-`).
    pub variables: HashMap<String, Value>,
    /// Public user-defined functions.
    pub functions: HashMap<String, UserFunction>,
    /// Public mixins.
    pub mixins: HashMap<String, Mixin>,
    /// CSS output produced by evaluating the module (emitted only once).
    pub css_output: Vec<crate::eval::CssRule>,
    /// Whether `with (...)` config was used when first loading this module.
    pub configured: bool,
}

impl EvaluatedModule {
    /// Build a `ModuleEnv` from the cached public members.
    pub fn to_module_env(&self) -> ModuleEnv {
        ModuleEnv {
            variables: self.variables.clone(),
            functions: self.functions.clone(),
            mixins: self.mixins.clone(),
        }
    }
}

/// Cache of evaluated modules, keyed by resolved file path.
///
/// Created once per compilation (in `evaluate` / `evaluate_with_dir`) and
/// passed through the evaluation chain.
pub struct ModuleCache {
    entries: HashMap<PathBuf, EvaluatedModule>,
}

impl ModuleCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Check if a module is already cached.
    pub fn contains(&self, path: &PathBuf) -> bool {
        self.entries.contains_key(path)
    }

    /// Get a cached module.
    pub fn get(&self, path: &PathBuf) -> Option<&EvaluatedModule> {
        self.entries.get(path)
    }

    /// Insert a freshly evaluated module into the cache.
    pub fn insert(&mut self, path: PathBuf, module: EvaluatedModule) {
        let span = tracing::debug_span!(
            "module_cache_store",
            stage = "eval",
            path = %path.display(),
            var_count = module.variables.len(),
            func_count = module.functions.len(),
            mixin_count = module.mixins.len(),
        );
        let _enter = span.enter();
        self.entries.insert(path, module);
    }

    /// Check if a module was already loaded (hit) — logs a trace event.
    pub fn log_hit(&self, path: &PathBuf) {
        let span = tracing::debug_span!(
            "module_cache_hit",
            stage = "eval",
            path = %path.display(),
        );
        let _enter = span.enter();
        tracing::trace!(
            stage = "eval",
            module = "cache",
            path = %path.display(),
            "module cache hit — reusing evaluated result"
        );
    }
}

impl Default for ModuleCache {
    fn default() -> Self {
        Self::new()
    }
}
