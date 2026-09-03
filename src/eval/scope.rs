//! 作用域链——Scope 结构体 + Rc<Scope> 父链管理嵌套作用域。

use super::env::{FunctionDef, MixinDef};
use crate::parse::ast::Value;
use std::collections::HashMap;
use std::rc::Rc;

/// 单层作用域——变量/mixin/function 表 + 父链。
///
/// 通过 `Rc<Scope>` 链接父作用域，实现零 clone 作用域进出。
/// `Env` 持有 `Rc<Scope>`（当前活跃作用域），进入子作用域时创建新 `Scope`，
/// 退出时恢复父 `Scope`。
#[derive(Debug, Clone, Default)]
pub(crate) struct Scope {
    pub(crate) local_vars: HashMap<String, Value>,
    pub(crate) local_mixins: HashMap<String, MixinDef>,
    pub(crate) local_functions: HashMap<String, FunctionDef>,
    pub(crate) forwarded_vars: HashMap<String, Value>,
    pub(crate) forwarded_mixins: HashMap<String, MixinDef>,
    pub(crate) forwarded_functions: HashMap<String, FunctionDef>,
    pub(crate) global_writes: HashMap<String, Value>,
    pub(crate) parent: Option<Rc<Scope>>,
}

impl Scope {
    /// 创建 root scope（无父作用域）。
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// 创建子 scope，设置 `parent` 指向当前 scope。
    pub(crate) fn new_child(parent: Rc<Scope>) -> Self {
        Self {
            parent: Some(parent),
            ..Self::default()
        }
    }

    /// 沿 parent 链向上查找变量。
    pub(crate) fn lookup(&self, name: &str) -> Option<&Value> {
        let mut scope: &Scope = self;
        loop {
            if let Some(v) = scope.local_vars.get(name) {
                return Some(v);
            }
            scope = scope.parent.as_deref()?;
        }
    }

    /// 沿 parent 链向上查找 mixin。
    pub(crate) fn get_mixin(&self, name: &str) -> Option<&MixinDef> {
        let mut scope: &Scope = self;
        loop {
            if let Some(m) = scope.local_mixins.get(name) {
                return Some(m);
            }
            scope = scope.parent.as_deref()?;
        }
    }

    /// 沿 parent 链向上查找 function。
    pub(crate) fn get_function(&self, name: &str) -> Option<&FunctionDef> {
        let mut scope: &Scope = self;
        loop {
            if let Some(f) = scope.local_functions.get(name) {
                return Some(f);
            }
            scope = scope.parent.as_deref()?;
        }
    }

    /// 检查变量是否在当前或父作用域中定义。
    pub(crate) fn has_var(&self, name: &str) -> bool {
        self.lookup(name).is_some()
    }
}
