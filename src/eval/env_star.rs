//! Env star_members / star_imported / 模块 getter 方法。
//!
//! 从 `env_impl.rs` 拆分出来，包含 star import 冲突检测和模块缓存 getter。

use super::env::{Env, ModuleExports};
use imbl::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::Rc;

impl Env {
    // --- star_members / star_imported 方法 ---

    pub(crate) fn add_star_members(mut self, module_name: &str, names: &[&str]) -> Self {
        for name in names {
            self.star_members
                .entry((*name).to_string())
                .or_default()
                .push(module_name.to_string());
        }
        self
    }

    pub(crate) fn star_conflict(&self, name: &str) -> Option<&[String]> {
        self.star_members
            .get(name)
            .filter(|v| v.len() > 1)
            .map(Vec::as_slice)
    }

    pub(crate) fn star_module_loaded(&self, module_name: &str) -> bool {
        self.star_members
            .values()
            .any(|mods| mods.iter().any(|m| m == module_name))
    }

    pub(crate) fn get_star_imported(&self) -> &HashSet<String> {
        &self.star_imported
    }

    pub(crate) fn add_star_imported(mut self, name: String) -> Self {
        self.star_imported.insert(name);
        self
    }

    pub(crate) fn get_css_imports(&self) -> &[String] {
        &self.css_imports
    }

    pub(crate) fn remove_star_imported(self) -> Self {
        let (mut scope, env) = self.mutate_scope();
        for name in &env.star_imported {
            scope.local_vars.remove(name);
            scope.local_mixins.remove(name);
            scope.local_functions.remove(name);
        }
        let mut env = env;
        env.star_imported.clear();
        env.with_scope(scope)
    }

    // --- 模块/extends Rc getter ---

    pub(crate) fn get_loaded_modules(&self) -> &HashSet<PathBuf> {
        &self.loaded_modules
    }

    pub(crate) fn get_module_cache_rc(&self) -> Rc<HashMap<PathBuf, ModuleExports>> {
        self.module_cache.clone()
    }

    pub(crate) fn get_loaded_modules_rc(&self) -> Rc<HashSet<PathBuf>> {
        self.loaded_modules.clone()
    }

    pub(crate) fn get_extends_rc(&self) -> Rc<Vec<(String, String, bool, Option<PathBuf>)>> {
        self.extends.clone()
    }
}
