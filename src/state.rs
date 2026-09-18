use std::collections::HashMap;

/// 单个 CSS 声明: `property: value`
#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub property: String,
    pub value: String,
}

/// CSS 规则: `selector { decls }
#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub selector: String,
    pub declarations: Vec<Declaration>,
}

/// 求值阶段共享状态 — 在 scan_map 链中移动,不 clone 分发
#[derive(Debug, Clone, Default)]
pub struct EvalState {
    pub variables: HashMap<String, String>,
    pub mixins: HashMap<String, Vec<String>>,
}

impl EvalState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_var(&mut self, key: String, value: String) {
        self.variables.insert(key, value);
    }

    pub fn get_var(&self, key: &str) -> Option<&str> {
        self.variables.get(key).map(|s| s.as_str())
    }

    pub fn register_mixin(&mut self, name: String, body: Vec<String>) {
        self.mixins.insert(name, body);
    }

    pub fn get_mixin(&self, name: &str) -> Option<&[String]> {
        self.mixins.get(name).map(|v| v.as_slice())
    }
}
