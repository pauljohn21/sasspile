//! CompilerContext — 编译期间的状态（路径栈 / 变量作用域）
//!
//! 查找用 borrow()，写操作通过 reducer 返回新状态实现（不在外部 borrow_mut）

use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CompilerContext {
    pub path_stack: Vec<PathBuf>,
    pub variables: HashMap<String, String>,
    pub mixins: HashMap<String, MixinDef>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MixinDef {
    pub params: Vec<String>,
    pub body: Vec<crate::ast::Node>,
}

#[allow(dead_code)]
impl CompilerContext {
    pub fn new() -> Self {
        Self {
            path_stack: Vec::new(),
            variables: HashMap::new(),
            mixins: HashMap::new(),
        }
    }

    pub fn push_path(&self, path: PathBuf) -> Self {
        let mut ctx = self.clone();
        ctx.path_stack.push(path);
        ctx
    }

    pub fn define_variable(&self, name: String, value: String) -> Self {
        let mut ctx = self.clone();
        ctx.variables.insert(name, value);
        ctx
    }

    pub fn define_mixin(&self, name: String, params: Vec<String>, body: Vec<crate::ast::Node>) -> Self {
        let mut ctx = self.clone();
        ctx.mixins.insert(name, MixinDef { params, body });
        ctx
    }

    pub fn resolve_variable(&self, name: &str) -> Option<&String> {
        self.variables.get(name)
    }

    pub fn resolve_mixin(&self, name: &str) -> Option<&MixinDef> {
        self.mixins.get(name)
    }
}

impl Default for CompilerContext {
    fn default() -> Self {
        Self::new()
    }
}
