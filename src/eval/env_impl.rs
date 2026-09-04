//! Env 方法实现——move 语义 + Rc<Scope> 父链。

use super::env::{Env, FunctionDef, MixinDef, ModuleExports};
use super::scope::Scope;
use crate::parse::ast::{Node, Param, Value};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::Rc;

impl Env {
    /// 获取 scope `的可变所有权——try_unwrap` 零 clone，fallback clone。
    fn mutate_scope(mut self) -> (Scope, Self) {
        let current = std::mem::take(&mut self.current);
        let scope = Rc::try_unwrap(current).unwrap_or_else(|rc| (*rc).clone());
        (scope, self)
    }

    /// 从 scope 和 self 重建 Env。
    fn with_scope(self, scope: Scope) -> Self {
        Self {
            current: Rc::new(scope),
            ..self
        }
    }

    pub fn incr_depth(mut self) -> Self {
        self.depth += 1;
        self
    }

    pub fn bind(self, name: String, value: Value) -> Self {
        let (mut scope, env) = self.mutate_scope();
        scope.local_vars.insert(name, value);
        env.with_scope(scope)
    }

    pub fn lookup(&self, name: &str) -> Option<&Value> {
        self.current.lookup(name)
    }

    pub fn has_var(&self, name: &str) -> bool {
        self.current.has_var(name)
    }

    pub(crate) fn define_mixin(self, name: String, def: MixinDef) -> Self {
        self.define_local_mixin(name, def)
    }

    pub(crate) fn define_local_mixin(self, name: String, def: MixinDef) -> Self {
        let (mut scope, env) = self.mutate_scope();
        scope.local_mixins.insert(name, def);
        env.with_scope(scope)
    }

    pub(crate) fn define_forwarded_mixin(self, name: String, def: MixinDef) -> Self {
        let (mut scope, env) = self.mutate_scope();
        scope.forwarded_mixins.insert(name, def);
        env.with_scope(scope)
    }

    /// 直接写入 `forwarded_vars（用于` @forward 绑定变量）。
    pub(crate) fn define_forwarded_var(self, name: String, val: Value) -> Self {
        let (mut scope, env) = self.mutate_scope();
        scope.forwarded_vars.insert(name, val);
        env.with_scope(scope)
    }

    pub(crate) fn get_mixin(&self, name: &str) -> Option<&MixinDef> {
        self.current.get_mixin(name)
    }

    pub(crate) fn get_mixin_ref_data(
        &self,
        name: &str,
    ) -> Option<(Vec<Param>, Vec<Node>, Vec<String>)> {
        self.current.get_mixin(name).map(|m| {
            let ns_keys: Vec<String> = m.captured_namespaces.keys().cloned().collect();
            (m.params.clone(), m.body.clone(), ns_keys)
        })
    }

    pub(crate) fn define_function(self, name: String, def: FunctionDef) -> Self {
        self.define_local_function(name, def)
    }

    pub(crate) fn define_local_function(self, name: String, def: FunctionDef) -> Self {
        let (mut scope, env) = self.mutate_scope();
        scope.local_functions.insert(name, def);
        env.with_scope(scope)
    }

    pub(crate) fn define_forwarded_function(self, name: String, def: FunctionDef) -> Self {
        let (mut scope, env) = self.mutate_scope();
        scope.forwarded_functions.insert(name, def);
        env.with_scope(scope)
    }

    pub(crate) fn get_function(&self, name: &str) -> Option<&FunctionDef> {
        self.current.get_function(name)
    }

    pub fn set_content(mut self, content: Vec<Node>, content_env: Env) -> Self {
        self.content = Some(Rc::new(content));
        self.content_env = Some(Rc::new(content_env));
        self
    }

    pub fn get_content(&self) -> Option<(&[Node], &Env)> {
        self.content
            .as_ref()
            .map(|c| c.as_slice())
            .zip(self.content_env.as_ref().map(std::convert::AsRef::as_ref))
    }

    pub fn add_module(mut self, name: String) -> Self {
        if !self.builtin_modules.contains(&name) {
            self.builtin_modules.push(name.clone());
        }
        let ns_name = name.strip_prefix("sass:").unwrap_or(&name).to_string();
        if let Some(exports) = super::module_helpers::builtin_module_exports(&name) {
            self.namespaces.insert(ns_name, Rc::new(exports));
        }
        self
    }

    pub(crate) fn add_namespace(mut self, ns: String, exports: ModuleExports) -> Self {
        self.namespaces.insert(ns, Rc::new(exports));
        self
    }

    pub(crate) fn get_namespace(&self, ns: &str) -> Option<&ModuleExports> {
        self.namespaces.get(ns).map(std::convert::AsRef::as_ref)
    }

    pub fn with_base_path(mut self, path: PathBuf) -> Self {
        self.base_path = Some(path);
        self
    }

    pub fn add_extend(
        self,
        extender: String,
        target: String,
        optional: bool,
        module: Option<PathBuf>,
    ) -> Self {
        let mut extends = (*self.extends).clone();
        extends.push((extender, target, optional, module));
        Self {
            extends: Rc::new(extends),
            ..self
        }
    }

    pub fn get_extends(&self) -> &[(String, String, bool, Option<PathBuf>)] {
        &self.extends
    }

    pub fn with_selector(mut self, sel: String) -> Self {
        self.current_selector = Some(sel);
        self
    }

    pub fn get_selector(&self) -> Option<&str> {
        self.current_selector.as_deref()
    }

    pub fn with_load_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.load_paths = paths;
        self
    }

    pub(crate) fn get_load_paths(&self) -> &[PathBuf] {
        &self.load_paths
    }

    pub(crate) fn get_module_cache(&self) -> &HashMap<PathBuf, ModuleExports> {
        &self.module_cache
    }

    pub(crate) fn with_module_cache(mut self, cache: HashMap<PathBuf, ModuleExports>) -> Self {
        self.module_cache = Rc::new(cache);
        self
    }

    pub fn with_plain_css(mut self, plain_css: bool) -> Self {
        self.plain_css = plain_css;
        self
    }

    pub(crate) fn with_depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    pub(crate) fn with_loaded_modules(mut self, loaded: HashSet<PathBuf>) -> Self {
        self.loaded_modules = Rc::new(loaded);
        self
    }

    pub(crate) fn with_extends(
        mut self,
        extends: Vec<(String, String, bool, Option<PathBuf>)>,
    ) -> Self {
        self.extends = Rc::new(extends);
        self
    }

    pub(crate) fn add_pending_config(mut self, key: String, val: Value) -> Self {
        self.pending_config.insert(key, val);
        self
    }

    pub(crate) fn get_pending_config(&self) -> &HashMap<String, Value> {
        &self.pending_config
    }

    pub(crate) fn add_consumed_config(mut self, key: String) -> Self {
        self.consumed_config.insert(key);
        self
    }

    pub(crate) fn get_consumed_config(&self) -> &HashSet<String> {
        &self.consumed_config
    }

    pub(crate) fn with_consumed_config(mut self, config: HashSet<String>) -> Self {
        self.consumed_config = config;
        self
    }

    pub(crate) fn add_global_write(self, name: String, val: Value) -> Self {
        let (mut scope, env) = self.mutate_scope();
        scope.global_writes.insert(name, val);
        env.with_scope(scope)
    }

    pub(crate) fn get_base_path(&self) -> Option<&PathBuf> {
        self.base_path.as_ref()
    }

    pub(crate) fn get_depth(&self) -> usize {
        self.depth
    }

    pub(crate) fn is_plain_css(&self) -> bool {
        self.plain_css
    }

    pub(crate) fn get_namespaces(&self) -> &HashMap<String, Rc<ModuleExports>> {
        &self.namespaces
    }

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

    // --- Scope 操作 ---

    /// 进入子作用域——创建空子 scope，parent 指向当前 scope。零 clone。
    pub(crate) fn enter_scope(self) -> Self {
        let parent = self.current.clone();
        let new_scope = Scope::new_child(parent);
        Self {
            current: Rc::new(new_scope),
            ..self
        }
    }

    /// 退出作用域——恢复父 scope，传播 !global 和新增 mixin/function。
    #[allow(clippy::needless_for_each)]
    pub(crate) fn exit_scope(self) -> Self {
        let parent = self.current.parent.clone();
        match parent {
            Some(parent_scope) => {
                let child_scope = match Rc::try_unwrap(self.current) {
                    Ok(s) => s,
                    Err(rc) => (*rc).clone(),
                };
                let mut new_parent = match Rc::try_unwrap(parent_scope) {
                    Ok(s) => s,
                    Err(rc) => (*rc).clone(),
                };
                // 传播命名空间变量（含 .）+ !global 变量
                new_parent.local_vars.extend(
                    child_scope
                        .local_vars
                        .into_iter()
                        .filter(|(name, _)| name.contains('.')),
                );
                new_parent.local_vars.extend(child_scope.global_writes);
                // 传播新增 mixin/function
                child_scope
                    .local_mixins
                    .into_iter()
                    .for_each(|(name, def)| {
                        new_parent.local_mixins.entry(name).or_insert(def);
                    });
                child_scope
                    .local_functions
                    .into_iter()
                    .for_each(|(name, def)| {
                        new_parent.local_functions.entry(name).or_insert(def);
                    });
                // 传播新增 forwarded 成员
                child_scope
                    .forwarded_mixins
                    .into_iter()
                    .for_each(|(name, def)| {
                        new_parent.forwarded_mixins.entry(name).or_insert(def);
                    });
                child_scope
                    .forwarded_functions
                    .into_iter()
                    .for_each(|(name, def)| {
                        new_parent.forwarded_functions.entry(name).or_insert(def);
                    });
                new_parent.forwarded_vars.extend(child_scope.forwarded_vars);
                Self {
                    current: Rc::new(new_parent),
                    ..self
                }
            }
            None => self, // root scope，无需退出
        }
    }

    /// 合并 forwarded 表到 local `表（std::mem::take` 模式）。
    #[allow(clippy::needless_for_each)]
    pub(crate) fn merge_forwarded_to_local(self) -> Self {
        let (mut scope, env) = self.mutate_scope();
        let forwarded_vars = std::mem::take(&mut scope.forwarded_vars);
        forwarded_vars.into_iter().for_each(|(k, v)| {
            scope.local_vars.entry(k).or_insert(v);
        });
        let forwarded_mixins = std::mem::take(&mut scope.forwarded_mixins);
        forwarded_mixins.into_iter().for_each(|(k, v)| {
            scope.local_mixins.entry(k).or_insert(v);
        });
        let forwarded_functions = std::mem::take(&mut scope.forwarded_functions);
        forwarded_functions.into_iter().for_each(|(k, v)| {
            scope.local_functions.entry(k).or_insert(v);
        });
        env.with_scope(scope)
    }

    /// 修改 namespace exports 中的变量。
    pub(crate) fn with_namespace_var(mut self, ns: &str, var_name: &str, val: Value) -> Self {
        if let Some(exports) = self.namespaces.get(ns) {
            let mut new_exports = (**exports).clone();
            match new_exports.forwarded_vars.contains_key(var_name) {
                true => {
                    new_exports.forwarded_vars.insert(var_name.to_string(), val);
                }
                false => {
                    new_exports.local_vars.insert(var_name.to_string(), val);
                }
            }
            self.namespaces.insert(ns.to_string(), Rc::new(new_exports));
        }
        self
    }

    /// 从当前 scope 提取字段到 ModuleExports（用于模块加载完成后构建导出）。
    pub(crate) fn take_scope_fields(
        &mut self,
    ) -> (
        HashMap<String, Value>,
        HashMap<String, MixinDef>,
        HashMap<String, FunctionDef>,
        HashMap<String, Value>,
        HashMap<String, MixinDef>,
        HashMap<String, FunctionDef>,
    ) {
        let scope = match Rc::try_unwrap(std::mem::take(&mut self.current)) {
            Ok(s) => s,
            Err(rc) => (*rc).clone(),
        };
        self.current = Rc::new(Scope::new());
        (
            scope.local_vars,
            scope.local_mixins,
            scope.local_functions,
            scope.forwarded_vars,
            scope.forwarded_mixins,
            scope.forwarded_functions,
        )
    }

    /// 获取 `forwarded_vars` 中的值引用。
    pub(crate) fn get_forwarded_var(&self, key: &str) -> Option<&Value> {
        self.current.forwarded_vars.get(key)
    }

    /// 获取 `forwarded_mixins` 中的值引用。
    pub(crate) fn get_forwarded_mixin(&self, key: &str) -> Option<&MixinDef> {
        self.current.forwarded_mixins.get(key)
    }

    /// 获取 `forwarded_functions` 中的值引用。
    pub(crate) fn get_forwarded_function(&self, key: &str) -> Option<&FunctionDef> {
        self.current.forwarded_functions.get(key)
    }
}
