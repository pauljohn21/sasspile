//! Environment — mutable variable/mixin/function scope management.

use crate::value::Value;
use std::collections::HashMap;
use std::path::PathBuf;

/// Function pointer type for builtin functions.
pub type BuiltinFn = fn(&[crate::ast::Arg], &mut Env) -> Result<Value, crate::error::SassError>;

/// Mixin definition.
#[derive(Debug, Clone)]
pub struct Mixin {
    pub params: Vec<crate::ast::Param>,
    pub body: Vec<crate::ast::Stmt>,
}

/// User-defined function.
#[derive(Debug, Clone)]
pub struct UserFunction {
    pub params: Vec<crate::ast::Param>,
    pub body: Vec<crate::ast::Stmt>,
}

/// The evaluation environment.
pub struct Env {
    pub variables: HashMap<String, Value>,
    pub mixins: HashMap<String, Mixin>,
    pub functions: HashMap<String, UserFunction>,
    pub builtins: HashMap<String, BuiltinFn>,
    pub modules: HashMap<String, ModuleEnv>,
    pub parent: Option<Box<Env>>,
    pub is_global: bool,
    /// Content block passed to a mixin via @include ... { @content }
    pub content: Option<Vec<crate::ast::Stmt>>,
    /// Base directory for resolving @use/@import from filesystem.
    pub base_dir: Option<PathBuf>,
}

/// A loaded module's environment (separate namespace).
pub struct ModuleEnv {
    pub variables: HashMap<String, Value>,
    pub mixins: HashMap<String, Mixin>,
    pub functions: HashMap<String, UserFunction>,
}

impl ModuleEnv {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            mixins: HashMap::new(),
            functions: HashMap::new(),
        }
    }
}

impl Env {
    pub fn new_global() -> Self {
        Self {
            variables: HashMap::new(),
            mixins: HashMap::new(),
            functions: HashMap::new(),
            builtins: HashMap::new(),
            modules: HashMap::new(),
            parent: None,
            is_global: true,
            content: None,
            base_dir: None,
        }
    }

    pub fn new_child(parent: Env) -> Self {
        Self {
            variables: HashMap::new(),
            mixins: HashMap::new(),
            functions: HashMap::new(),
            builtins: HashMap::new(),
            modules: HashMap::new(),
            parent: Some(Box::new(parent)),
            is_global: false,
            content: None,
            base_dir: None,
        }
    }

    /// Look up a variable in the scope chain (local → parent → global).
    pub fn get_var(&self, name: &str) -> Option<&Value> {
        if let Some(v) = self.variables.get(name) {
            Some(v)
        } else if let Some(ref p) = self.parent {
            p.get_var(name)
        } else {
            None
        }
    }

    /// Check if a variable is defined in the scope chain.
    pub fn has_var(&self, name: &str) -> bool {
        self.get_var(name).is_some()
    }

    /// Set a variable with !default and !global handling.
    pub fn set_var(&mut self, name: String, value: Value, global: bool, default: bool) {
        if default {
            // !default: only set if not already defined or is null
            let existing = self.get_var(&name);
            if existing.map_or(false, |v| !matches!(v, Value::Null)) {
                return;
            }
        }

        if global {
            // !global: set in global scope
            self.set_global_var(&name, value);
        } else if self.is_global {
            self.variables.insert(name, value);
        } else {
            // Check if variable exists in parent scope — update there
            if self.has_var_in_parent(&name) {
                self.set_existing_var(&name, value);
            } else {
                self.variables.insert(name, value);
            }
        }
    }

    fn has_var_in_parent(&self, name: &str) -> bool {
        if let Some(ref p) = self.parent {
            p.has_var(name)
        } else {
            false
        }
    }

    fn set_existing_var(&mut self, name: &str, value: Value) {
        if self.variables.contains_key(name) {
            self.variables.insert(name.to_string(), value);
            return;
        }
        if let Some(ref mut p) = self.parent {
            p.set_existing_var(name, value);
        }
    }

    fn set_global_var(&mut self, name: &str, value: Value) {
        if self.is_global {
            self.variables.insert(name.to_string(), value);
            return;
        }
        if let Some(ref mut p) = self.parent {
            p.set_global_var(name, value);
        }
    }

    /// Get the content block (for @content in mixins).
    pub fn get_content(&self) -> Option<&[crate::ast::Stmt]> {
        if let Some(ref c) = self.content {
            return Some(c);
        }
        if let Some(ref p) = self.parent {
            return p.get_content();
        }
        None
    }

    /// Set the content block.
    pub fn set_content(&mut self, content: Vec<crate::ast::Stmt>) {
        self.content = Some(content);
    }

    /// Look up a mixin.
    pub fn get_mixin(&self, name: &str) -> Option<&Mixin> {
        if let Some(m) = self.mixins.get(name) {
            Some(m)
        } else if let Some(ref p) = self.parent {
            p.get_mixin(name)
        } else {
            None
        }
    }

    /// Register a mixin.
    pub fn set_mixin(&mut self, name: String, mixin: Mixin) {
        self.mixins.insert(name, mixin);
    }

    /// Look up a user function.
    pub fn get_function(&self, name: &str) -> Option<&UserFunction> {
        if let Some(f) = self.functions.get(name) {
            Some(f)
        } else if let Some(ref p) = self.parent {
            p.get_function(name)
        } else {
            None
        }
    }

    /// Register a user function.
    pub fn set_function(&mut self, name: String, func: UserFunction) {
        self.functions.insert(name, func);
    }

    /// Look up a builtin function (global scope only).
    pub fn get_builtin(&self, name: &str) -> Option<&BuiltinFn> {
        if let Some(f) = self.builtins.get(name) {
            Some(f)
        } else if let Some(ref p) = self.parent {
            p.get_builtin(name)
        } else {
            None
        }
    }

    /// Register a builtin function.
    pub fn register_builtin(&mut self, name: String, func: BuiltinFn) {
        self.builtins.insert(name, func);
    }

    /// Get the base directory for @use resolution.
    pub fn get_base_dir(&self) -> Option<&PathBuf> {
        if self.base_dir.is_some() {
            self.base_dir.as_ref()
        } else if let Some(ref p) = self.parent {
            p.get_base_dir()
        } else {
            None
        }
    }

    /// Get a variable from a module namespace (searches scope chain).
    pub fn get_module_var(&self, ns: &str, name: &str) -> Option<&Value> {
        if let Some(m) = self.modules.get(ns) {
            if let Some(v) = m.variables.get(name) {
                return Some(v);
            }
        }
        if let Some(ref p) = self.parent {
            return p.get_module_var(ns, name);
        }
        None
    }

    /// Get a function from a module namespace (searches scope chain).
    pub fn get_module_function(&self, ns: &str, name: &str) -> Option<&UserFunction> {
        if let Some(m) = self.modules.get(ns) {
            if let Some(f) = m.functions.get(name) {
                return Some(f);
            }
        }
        if let Some(ref p) = self.parent {
            return p.get_module_function(ns, name);
        }
        None
    }

    /// Get a mixin from a module namespace (searches scope chain).
    pub fn get_module_mixin(&self, ns: &str, name: &str) -> Option<&Mixin> {
        if let Some(m) = self.modules.get(ns) {
            if let Some(mx) = m.mixins.get(name) {
                return Some(mx);
            }
        }
        if let Some(ref p) = self.parent {
            return p.get_module_mixin(ns, name);
        }
        None
    }

    /// Register a module on the global scope (modules are global in Sass).
    pub fn set_module(&mut self, ns: String, module: ModuleEnv) {
        if self.is_global {
            self.modules.insert(ns, module);
            return;
        }
        if let Some(ref mut p) = self.parent {
            p.set_module(ns, module);
        }
    }

    /// Get all registered module namespace names (for debugging).
    pub fn modules_keys(&self) -> Vec<String> {
        if self.is_global {
            self.modules.keys().cloned().collect()
        } else if let Some(ref p) = self.parent {
            p.modules_keys()
        } else {
            self.modules.keys().cloned().collect()
        }
    }

    /// Export all variables (for @use namespace collection).
    /// Only returns variables from the current scope chain, not builtins.
    pub fn export_vars(&self) -> Vec<(String, Value)> {
        self.variables.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// Export all user-defined functions (for @use namespace collection).
    pub fn export_functions(&self) -> Vec<(String, UserFunction)> {
        self.functions.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// Export all mixins (for @use namespace collection).
    pub fn export_mixins(&self) -> Vec<(String, Mixin)> {
        self.mixins.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// Check if a variable exists (including global scope).
    pub fn variable_exists(&self, name: &str) -> bool {
        self.has_var(name)
    }

    /// Check if a mixin exists.
    pub fn mixin_exists(&self, name: &str) -> bool {
        self.get_mixin(name).is_some()
    }

    /// Check if a function exists (user or builtin).
    pub fn function_exists(&self, name: &str) -> bool {
        self.get_function(name).is_some() || self.get_builtin(name).is_some()
    }

    /// Check if a global variable exists.
    pub fn global_variable_exists(&self, name: &str) -> bool {
        let mut env = self;
        while let Some(ref p) = env.parent {
            env = p;
        }
        env.variables.contains_key(name)
    }
}
