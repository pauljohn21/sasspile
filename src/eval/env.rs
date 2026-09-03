//! 求值环境——Env + `ModuleExports` 类型定义和方法。

use crate::css::node::CssNode;
use crate::eval::scope::Scope;
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

/// 不可变求值环境（move 语义——零 clone 作用域进出）。
///
/// `current` 持有 `Rc<Scope>`（当前活跃作用域），通过 parent 链管理嵌套作用域。
/// 写操作通过 `Rc::try_unwrap` 获取 scope 所有权（引用计数为 1 时零 clone）。
#[derive(Debug, Default)]
#[allow(clippy::struct_field_names)]
pub struct Env {
    /// 当前活跃作用域——通过 Rc<Scope> 父链管理嵌套。
    pub(crate) current: Rc<Scope>,
    // 全局字段——不参与作用域链
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

impl Clone for Env {
    fn clone(&self) -> Self {
        Self {
            current: Rc::clone(&self.current),
            content: self.content.clone(),
            content_env: self.content_env.clone(),
            builtin_modules: self.builtin_modules.clone(),
            namespaces: self.namespaces.clone(),
            base_path: self.base_path.clone(),
            depth: self.depth,
            extends: self.extends.clone(),
            current_selector: self.current_selector.clone(),
            load_paths: self.load_paths.clone(),
            plain_css: self.plain_css,
            loaded_modules: self.loaded_modules.clone(),
            module_cache: self.module_cache.clone(),
            pending_config: self.pending_config.clone(),
            consumed_config: self.consumed_config.clone(),
            star_members: self.star_members.clone(),
            star_imported: self.star_imported.clone(),
        }
    }
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
