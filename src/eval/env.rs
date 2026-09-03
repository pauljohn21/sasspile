//! 求值环境——Env + ModuleExports 类型定义和方法。

use crate::css::node::CssNode;
use crate::parse::ast::*;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::Rc;

/// 模块导出——加载的文件模块的成员。
#[derive(Debug, Clone, Default)]
pub(crate) struct ModuleExports {
    pub(crate) local_vars: HashMap<String, Value>,
    pub(crate) local_mixins: HashMap<String, MixinDef>,
    pub(crate) local_functions: HashMap<String, FunctionDef>,
    pub(crate) forwarded_vars: HashMap<String, Value>,
    pub(crate) forwarded_mixins: HashMap<String, MixinDef>,
    pub(crate) forwarded_functions: HashMap<String, FunctionDef>,
    pub(crate) css: Vec<CssNode>,
    pub(crate) loaded_modules: Rc<HashSet<PathBuf>>,
    pub(crate) extends: Rc<Vec<(String, String, bool, Option<PathBuf>)>>,
    pub(crate) module_cache: Rc<HashMap<PathBuf, ModuleExports>>,
    pub(crate) consumed_config: HashSet<String>,
    /// 该模块产生的所有选择器（用于 extend scope 检查）
    pub(crate) selectors: HashSet<String>,
    /// 通过 `@use ... as *` 引入的成员名集合（不应传递到下一个 `@use ... as *`）。
    pub(crate) star_imported: HashSet<String>,
}

impl ModuleExports {
    pub(crate) fn all_functions(&self) -> impl Iterator<Item = (&String, &FunctionDef)> {
        self.local_functions.iter().chain(
            self.forwarded_functions
                .iter()
                .filter(|(k, _)| !self.local_functions.contains_key(*k)),
        )
    }
    pub(crate) fn all_mixins(&self) -> impl Iterator<Item = (&String, &MixinDef)> {
        self.local_mixins.iter().chain(
            self.forwarded_mixins
                .iter()
                .filter(|(k, _)| !self.local_mixins.contains_key(*k)),
        )
    }
    pub(crate) fn all_vars(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.local_vars.iter().chain(
            self.forwarded_vars
                .iter()
                .filter(|(k, _)| !self.local_vars.contains_key(*k)),
        )
    }
}

/// 不可变求值环境（move 语义——零 clone）。
#[derive(Debug, Clone, Default)]
pub struct Env {
    pub(crate) local_vars: HashMap<String, Value>,
    pub(crate) local_mixins: HashMap<String, MixinDef>,
    pub(crate) local_functions: HashMap<String, FunctionDef>,
    pub(crate) forwarded_vars: HashMap<String, Value>,
    pub(crate) forwarded_mixins: HashMap<String, MixinDef>,
    pub(crate) forwarded_functions: HashMap<String, FunctionDef>,
    pub(crate) global_writes: HashMap<String, Value>,
    pub(crate) content: Option<Rc<Vec<Node>>>,
    pub(crate) content_env: Option<Rc<Env>>,
    pub(crate) builtin_modules: Vec<String>,
    pub(crate) namespaces: HashMap<String, Rc<ModuleExports>>,
    pub(crate) base_path: Option<PathBuf>,
    pub(crate) depth: usize,
    pub(crate) extends: Rc<Vec<(String, String, bool, Option<PathBuf>)>>,
    pub(crate) current_selector: Option<String>,
    pub(crate) load_paths: Vec<PathBuf>,
    pub(crate) plain_css: bool,
    pub(crate) loaded_modules: Rc<HashSet<PathBuf>>,
    pub(crate) module_cache: Rc<HashMap<PathBuf, ModuleExports>>,
    pub(crate) pending_config: HashMap<String, Value>,
    /// 已被 !default 变量消费的 `pending_config` key 集合。
    pub(crate) consumed_config: HashSet<String>,
    /// `@use ... as *` 模块的成员名→模块名列表映射（用于冲突检测）。
    pub(crate) star_members: HashMap<String, Vec<String>>,
    /// 通过 `@use ... as *` 引入到当前作用域的成员名集合。
    pub(crate) star_imported: HashSet<String>,
}

/// mixin 定义存储。
#[derive(Debug, Clone)]
pub(crate) struct MixinDef {
    pub(crate) params: Vec<Param>,
    pub(crate) body: Vec<Node>,
    pub(crate) captured_namespaces: HashMap<String, Rc<ModuleExports>>,
}

/// 函数定义存储。
#[derive(Debug, Clone)]
pub(crate) struct FunctionDef {
    pub(crate) params: Vec<Param>,
    pub(crate) body: Vec<Node>,
    pub(crate) captured_namespaces: HashMap<String, Rc<ModuleExports>>,
}

impl Env {
    pub fn new_env() -> Self {
        Self::default()
    }

    pub fn incr_depth(mut self) -> Self {
        self.depth += 1;
        self
    }
    pub fn bind(mut self, name: String, value: Value) -> Self {
        self.local_vars.insert(name, value);
        self
    }
    pub fn lookup(&self, name: &str) -> Option<&Value> {
        self.local_vars.get(name)
    }
    pub fn has_var(&self, name: &str) -> bool {
        self.local_vars.contains_key(name)
    }

    pub(crate) fn define_mixin(self, name: String, def: MixinDef) -> Self {
        self.define_local_mixin(name, def)
    }
    pub(crate) fn define_local_mixin(mut self, name: String, def: MixinDef) -> Self {
        self.local_mixins.insert(name, def);
        self
    }
    pub(crate) fn define_forwarded_mixin(mut self, name: String, def: MixinDef) -> Self {
        self.forwarded_mixins.insert(name, def);
        self
    }
    pub(crate) fn get_mixin(&self, name: &str) -> Option<&MixinDef> {
        self.local_mixins.get(name)
    }

    pub(crate) fn get_mixin_ref_data(
        &self,
        name: &str,
    ) -> Option<(Vec<Param>, Vec<Node>, Vec<String>)> {
        self.local_mixins.get(name).map(|m| {
            let ns_keys: Vec<String> = m.captured_namespaces.keys().cloned().collect();
            (m.params.clone(), m.body.clone(), ns_keys)
        })
    }

    pub(crate) fn define_function(self, name: String, def: FunctionDef) -> Self {
        self.define_local_function(name, def)
    }
    pub(crate) fn define_local_function(mut self, name: String, def: FunctionDef) -> Self {
        self.local_functions.insert(name, def);
        self
    }
    pub(crate) fn define_forwarded_function(mut self, name: String, def: FunctionDef) -> Self {
        self.forwarded_functions.insert(name, def);
        self
    }
    pub(crate) fn get_function(&self, name: &str) -> Option<&FunctionDef> {
        self.local_functions.get(name)
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
    pub fn has_module(&self, name: &str) -> bool {
        self.builtin_modules.iter().any(|m| m == name)
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
    pub fn add_extend(self, extender: String, target: String, optional: bool, module: Option<PathBuf>) -> Self {
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
    pub(crate) fn with_loaded_modules(
        mut self,
        loaded: HashSet<PathBuf>,
    ) -> Self {
        self.loaded_modules = Rc::new(loaded);
        self
    }
    pub(crate) fn with_extends(mut self, extends: Vec<(String, String, bool, Option<PathBuf>)>) -> Self {
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
    pub(crate) fn add_global_write(mut self, name: String, val: Value) -> Self {
        self.global_writes.insert(name, val);
        self
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
    pub(crate) fn get_local_vars(&self) -> &HashMap<String, Value> {
        &self.local_vars
    }
    pub(crate) fn get_local_mixins(&self) -> &HashMap<String, MixinDef> {
        &self.local_mixins
    }
    pub(crate) fn get_local_functions(&self) -> &HashMap<String, FunctionDef> {
        &self.local_functions
    }
    pub(crate) fn get_forwarded_vars(&self) -> &HashMap<String, Value> {
        &self.forwarded_vars
    }
    pub(crate) fn get_forwarded_mixins(&self) -> &HashMap<String, MixinDef> {
        &self.forwarded_mixins
    }
    pub(crate) fn get_forwarded_functions(&self) -> &HashMap<String, FunctionDef> {
        &self.forwarded_functions
    }
    pub(crate) fn get_namespaces(&self) -> &HashMap<String, Rc<ModuleExports>> {
        &self.namespaces
    }
    /// 添加 `as *` 模块的成员到冲突追踪映射。
    pub(crate) fn add_star_members(mut self, module_name: &str, names: &[&str]) -> Self {
        for name in names {
            self.star_members
                .entry((*name).to_string())
                .or_default()
                .push(module_name.to_string());
        }
        self
    }
    /// 检查成员名是否被多个 `as *` 模块定义（冲突）。
    pub(crate) fn star_conflict(&self, name: &str) -> Option<&[String]> {
        self.star_members
            .get(name)
            .filter(|v| v.len() > 1)
            .map(Vec::as_slice)
    }
    /// 检查 `as *` 模块是否已记录到 `star_members`。
    pub(crate) fn star_module_loaded(&self, module_name: &str) -> bool {
        self.star_members
            .values()
            .any(|mods| mods.iter().any(|m| m == module_name))
    }
    /// 获取 `star_imported` 成员名集合。
    pub(crate) fn get_star_imported(&self) -> &HashSet<String> {
        &self.star_imported
    }
    /// 添加通过 `@use ... as *` 引入的成员名。
    pub(crate) fn add_star_imported(mut self, name: String) -> Self {
        self.star_imported.insert(name);
        self
    }
    /// 移除通过 `@use ... as *` 引入的传递性成员（@import 内联后清理）。
    pub(crate) fn remove_star_imported(mut self) -> Self {
        for name in self.star_imported.drain() {
            self.local_vars.remove(&name);
            self.local_mixins.remove(&name);
            self.local_functions.remove(&name);
        }
        self
    }
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
    pub(crate) fn merge_forwarded_to_local(mut self) -> Self {
        for (k, v) in self
            .forwarded_vars
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            self.local_vars.entry(k).or_insert(v);
        }
        for (k, v) in self
            .forwarded_mixins
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            self.local_mixins.entry(k).or_insert(v);
        }
        for (k, v) in self
            .forwarded_functions
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            self.local_functions.entry(k).or_insert(v);
        }
        self.forwarded_vars.clear();
        self.forwarded_mixins.clear();
        self.forwarded_functions.clear();
        self
    }
    pub(crate) fn with_namespace_var(mut self, ns: &str, var_name: &str, val: Value) -> Self {
        if let Some(exports) = self.namespaces.get(ns) {
            let mut new_exports = (**exports).clone();
            if new_exports.forwarded_vars.contains_key(var_name) {
                new_exports.forwarded_vars.insert(var_name.to_string(), val);
            } else {
                new_exports.local_vars.insert(var_name.to_string(), val);
            }
            self.namespaces.insert(ns.to_string(), Rc::new(new_exports));
        }
        self
    }
    /// 从子作用域提取传播字段，合并回 saved 状态。
    /// 规则体内的局部变量/mixin/function 不传播到外层，
    /// 但命名空间变量（含 .）、!global 变量、新增 mixin/function 传播。
    pub(crate) fn exit_scope(
        mut self,
        saved_local_vars: HashMap<String, Value>,
        saved_local_mixins: HashMap<String, MixinDef>,
        saved_local_functions: HashMap<String, FunctionDef>,
        saved_forwarded_vars: HashMap<String, Value>,
        saved_forwarded_mixins: HashMap<String, MixinDef>,
        saved_forwarded_functions: HashMap<String, FunctionDef>,
    ) -> Self {
        // 提取规则体内产生的变更
        let rule_local_vars = std::mem::take(&mut self.local_vars);
        let rule_global_writes = std::mem::take(&mut self.global_writes);
        let rule_local_mixins = std::mem::take(&mut self.local_mixins);
        let rule_local_functions = std::mem::take(&mut self.local_functions);
        let rule_forwarded_mixins = std::mem::take(&mut self.forwarded_mixins);
        let rule_forwarded_functions = std::mem::take(&mut self.forwarded_functions);
        let rule_forwarded_vars = std::mem::take(&mut self.forwarded_vars);

        // 恢复 saved 的 local 表
        self.local_vars = saved_local_vars;
        self.local_mixins = saved_local_mixins;
        self.local_functions = saved_local_functions;
        self.forwarded_vars = saved_forwarded_vars;
        self.forwarded_mixins = saved_forwarded_mixins;
        self.forwarded_functions = saved_forwarded_functions;

        // 传播命名空间变量赋值（名字含 . 的）+ !global 变量赋值
        self.local_vars.extend(
            rule_local_vars.into_iter()
                .filter(|(name, _)| name.contains('.')),
        );
        self.local_vars.extend(rule_global_writes);
        // 传播新增 mixin/function（规则体内定义的）
        #[allow(clippy::needless_for_each)]
        rule_local_mixins.into_iter()
            .for_each(|(name, def)| { self.local_mixins.entry(name).or_insert(def); });
        #[allow(clippy::needless_for_each)]
        rule_local_functions.into_iter()
            .for_each(|(name, def)| { self.local_functions.entry(name).or_insert(def); });
        // 传播新增 forwarded 成员
        #[allow(clippy::needless_for_each)]
        rule_forwarded_mixins.into_iter()
            .for_each(|(name, def)| { self.forwarded_mixins.entry(name).or_insert(def); });
        #[allow(clippy::needless_for_each)]
        rule_forwarded_functions.into_iter()
            .for_each(|(name, def)| { self.forwarded_functions.entry(name).or_insert(def); });
        self.forwarded_vars.extend(rule_forwarded_vars);
        self
    }
}
