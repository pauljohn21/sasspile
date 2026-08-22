//! Env — 求值环境，move 语义。

use crate::eval::value::Value;
use crate::parse::{ast::{Param, Arg}, Node};
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

/// 混合体定义。
#[derive(Debug, Clone)]
pub struct MixinDef {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Node>,
}

/// 函数定义。
#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Node>,
}

/// 模块导出。
#[derive(Debug, Clone, Default)]
pub struct ModuleExports {
    pub variables: HashMap<String, Value>,
    pub mixins: HashMap<String, MixinDef>,
    pub functions: HashMap<String, FunctionDef>,
    pub css: Vec<crate::css::CssNode>,
}

/// 求值环境——move 语义。
pub struct Env {
    // 可变状态（enter_scope 时克隆）
    pub local_vars: HashMap<String, Value>,
    pub local_mixins: HashMap<String, MixinDef>,
    pub local_functions: HashMap<String, FunctionDef>,

    // 共享状态（Rc 引用计数）
    pub content: Option<Rc<Vec<Node>>>,
    pub namespaces: HashMap<String, Rc<ModuleExports>>,
    pub extends: Rc<Vec<(String, String, bool)>>,
    pub loaded_modules: Rc<Vec<PathBuf>>,
    pub module_cache: Rc<HashMap<PathBuf, ModuleExports>>,

    // 管线配置
    pub base_path: Option<PathBuf>,
    pub load_paths: Vec<PathBuf>,
    pub current_selector: Option<String>,
    pub depth: usize,
    pub plain_css: bool,
}

impl Env {
    /// 创建根环境。
    pub fn root(base_path: Option<PathBuf>, load_paths: Vec<PathBuf>) -> Self {
        Self {
            local_vars: HashMap::new(),
            local_mixins: HashMap::new(),
            local_functions: HashMap::new(),
            content: None,
            namespaces: HashMap::new(),
            extends: Rc::new(Vec::new()),
            loaded_modules: Rc::new(Vec::new()),
            module_cache: Rc::new(HashMap::new()),
            base_path,
            load_paths,
            current_selector: None,
            depth: 0,
            plain_css: false,
        }
    }

    /// 进入子作用域——克隆 local，共享 Rc。
    pub fn enter_scope(&self) -> Env {
        Env {
            local_vars: self.local_vars.clone(),
            local_mixins: self.local_mixins.clone(),
            local_functions: self.local_functions.clone(),
            content: self.content.clone(),
            namespaces: self.namespaces.clone(),
            extends: self.extends.clone(),
            loaded_modules: self.loaded_modules.clone(),
            module_cache: self.module_cache.clone(),
            base_path: self.base_path.clone(),
            load_paths: self.load_paths.clone(),
            current_selector: self.current_selector.clone(),
            depth: self.depth + 1,
            plain_css: self.plain_css,
        }
    }

    /// 退出子作用域——从 child 提取传播字段。
    pub fn exit_scope(self, child: &Env) -> Env {
        Env {
            local_vars: child.local_vars.clone(),
            local_mixins: child.local_mixins.clone(),
            local_functions: child.local_functions.clone(),
            content: self.content.clone(),
            namespaces: child.namespaces.clone(),
            extends: child.extends.clone(),
            loaded_modules: child.loaded_modules.clone(),
            module_cache: child.module_cache.clone(),
            base_path: self.base_path,
            load_paths: self.load_paths,
            current_selector: self.current_selector.clone(),
            depth: self.depth,
            plain_css: self.plain_css,
        }
    }

    // ─── Builder 方法 ───

    pub fn with_selector(mut self, sel: String) -> Self {
        self.current_selector = Some(sel);
        self
    }

    pub fn with_content(mut self, body: Vec<Node>) -> Self {
        self.content = Some(Rc::new(body));
        self
    }

    pub fn with_extends(mut self, ext: Vec<(String, String, bool)>) -> Self {
        self.extends = Rc::new(ext);
        self
    }

    pub fn add_extend(mut self, extender: String, target: String, optional: bool) -> Self {
        let mut extends = (*self.extends).clone();
        extends.push((extender, target, optional));
        self.extends = Rc::new(extends);
        self
    }

    pub fn define_var(mut self, name: &str, value: Value) -> Self {
        self.local_vars.insert(name.to_string(), value);
        self
    }

    pub fn define_mixin(mut self, mixin: MixinDef) -> Self {
        self.local_mixins.insert(mixin.name.clone(), mixin);
        self
    }

    pub fn define_function(mut self, func: FunctionDef) -> Self {
        self.local_functions.insert(func.name.clone(), func);
        self
    }

    pub fn define_namespace(mut self, name: String, exports: Rc<ModuleExports>) -> Self {
        self.namespaces.insert(name, exports);
        self
    }

    // ─── 只读访问 ───

    pub fn get_var(&self, name: &str) -> Option<&Value> {
        self.local_vars.get(name)
    }

    pub fn get_mixin(&self, name: &str) -> Option<&MixinDef> {
        self.local_mixins.get(name)
    }

    pub fn get_function(&self, name: &str) -> Option<&FunctionDef> {
        self.local_functions.get(name)
    }

    pub fn get_content(&self) -> Option<&Vec<Node>> {
        self.content.as_deref()
    }

    pub fn get_extends(&self) -> &[(String, String, bool)] {
        &self.extends
    }
}
