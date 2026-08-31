//! 求值器——纯函数式管线 + move 语义（零 clone）。

use crate::css::node::CssNode;
use crate::error::{Result, SassError};
use crate::lex::Lexer;
use crate::lex::token::Token;
use crate::parse::ast::*;
use crate::__tracing::warn;

use std::collections::HashMap;
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
    pub(crate) loaded_modules: Rc<std::collections::HashSet<PathBuf>>,
    pub(crate) extends: Rc<Vec<(String, String, bool)>>,
    pub(crate) module_cache: Rc<HashMap<PathBuf, ModuleExports>>,
}

impl ModuleExports {
    pub(crate) fn all_functions(&self) -> impl Iterator<Item = (&String, &FunctionDef)> {
        self.local_functions.iter().chain(
            self.forwarded_functions.iter().filter(|(k, _)| !self.local_functions.contains_key(*k))
        )
    }
    pub(crate) fn all_mixins(&self) -> impl Iterator<Item = (&String, &MixinDef)> {
        self.local_mixins.iter().chain(
            self.forwarded_mixins.iter().filter(|(k, _)| !self.local_mixins.contains_key(*k))
        )
    }
    pub(crate) fn all_vars(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.local_vars.iter().chain(
            self.forwarded_vars.iter().filter(|(k, _)| !self.local_vars.contains_key(*k))
        )
    }
}

/// 不可变求值环境（move 语义——零 clone）。
#[derive(Debug, Clone, Default)]
pub struct Env {
    local_vars: HashMap<String, Value>,
    local_mixins: HashMap<String, MixinDef>,
    local_functions: HashMap<String, FunctionDef>,
    forwarded_vars: HashMap<String, Value>,
    forwarded_mixins: HashMap<String, MixinDef>,
    forwarded_functions: HashMap<String, FunctionDef>,
    global_writes: HashMap<String, Value>,
    content: Option<Rc<Vec<Node>>>,
    content_env: Option<Rc<Env>>,
    builtin_modules: Vec<String>,
    pub(crate) namespaces: HashMap<String, Rc<ModuleExports>>,
    base_path: Option<PathBuf>,
    depth: usize,
    extends: Rc<Vec<(String, String, bool)>>,
    current_selector: Option<String>,
    load_paths: Vec<PathBuf>,
    plain_css: bool,
    loaded_modules: Rc<std::collections::HashSet<PathBuf>>,
    module_cache: Rc<HashMap<PathBuf, ModuleExports>>,
    pending_config: HashMap<String, Value>,
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
    pub fn new_env() -> Self { Self::default() }

    pub fn incr_depth(mut self) -> Self { self.depth += 1; self }
    pub fn bind(mut self, name: String, value: Value) -> Self { self.local_vars.insert(name, value); self }
    pub fn lookup(&self, name: &str) -> Option<&Value> { self.local_vars.get(name) }
    pub fn has_var(&self, name: &str) -> bool { self.local_vars.contains_key(name) }

    pub(crate) fn define_mixin(self, name: String, def: MixinDef) -> Self { self.define_local_mixin(name, def) }
    pub(crate) fn define_local_mixin(mut self, name: String, def: MixinDef) -> Self { self.local_mixins.insert(name, def); self }
    pub(crate) fn define_forwarded_mixin(mut self, name: String, def: MixinDef) -> Self { self.forwarded_mixins.insert(name, def); self }
    pub(crate) fn get_mixin(&self, name: &str) -> Option<&MixinDef> { self.local_mixins.get(name) }

    pub(crate) fn get_mixin_ref_data(&self, name: &str) -> Option<(Vec<Param>, Vec<Node>, Vec<String>)> {
        self.local_mixins.get(name).map(|m| {
            let ns_keys: Vec<String> = m.captured_namespaces.keys().cloned().collect();
            (m.params.clone(), m.body.clone(), ns_keys)
        })
    }

    pub(crate) fn define_function(self, name: String, def: FunctionDef) -> Self { self.define_local_function(name, def) }
    pub(crate) fn define_local_function(mut self, name: String, def: FunctionDef) -> Self { self.local_functions.insert(name, def); self }
    pub(crate) fn define_forwarded_function(mut self, name: String, def: FunctionDef) -> Self { self.forwarded_functions.insert(name, def); self }
    pub(crate) fn get_function(&self, name: &str) -> Option<&FunctionDef> { self.local_functions.get(name) }

    pub fn set_content(mut self, content: Vec<Node>, content_env: Env) -> Self {
        self.content = Some(Rc::new(content));
        self.content_env = Some(Rc::new(content_env));
        self
    }
    pub fn get_content(&self) -> Option<(&[Node], &Env)> {
        self.content.as_ref().map(|c| c.as_slice()).zip(self.content_env.as_ref().map(|e| e.as_ref()))
    }

    pub fn add_module(mut self, name: String) -> Self {
        if !self.builtin_modules.contains(&name) { self.builtin_modules.push(name.clone()); }
        let ns_name = name.strip_prefix("sass:").unwrap_or(&name).to_string();
        if let Some(exports) = module_helpers::builtin_module_exports(&name) {
            self.namespaces.insert(ns_name, Rc::new(exports));
        }
        self
    }
    pub fn has_module(&self, name: &str) -> bool { self.builtin_modules.iter().any(|m| m == name) }

    pub(crate) fn add_namespace(mut self, ns: String, exports: ModuleExports) -> Self { self.namespaces.insert(ns, Rc::new(exports)); self }
    pub(crate) fn get_namespace(&self, ns: &str) -> Option<&ModuleExports> { self.namespaces.get(ns).map(|rc| rc.as_ref()) }

    pub fn with_base_path(mut self, path: PathBuf) -> Self { self.base_path = Some(path); self }
    pub fn add_extend(mut self, extender: String, target: String, optional: bool) -> Self { Rc::make_mut(&mut self.extends).push((extender, target, optional)); self }
    pub fn get_extends(&self) -> &[(String, String, bool)] { &self.extends }
    pub fn with_selector(mut self, sel: String) -> Self { self.current_selector = Some(sel); self }
    pub fn get_selector(&self) -> Option<&str> { self.current_selector.as_deref() }
    pub fn with_load_paths(mut self, paths: Vec<PathBuf>) -> Self { self.load_paths = paths; self }
    pub(crate) fn get_load_paths(&self) -> &[PathBuf] { &self.load_paths }
    pub(crate) fn get_module_cache(&self) -> &HashMap<PathBuf, ModuleExports> { &self.module_cache }
    pub(crate) fn with_module_cache(mut self, cache: HashMap<PathBuf, ModuleExports>) -> Self { self.module_cache = Rc::new(cache); self }
    pub fn with_plain_css(mut self, plain_css: bool) -> Self { self.plain_css = plain_css; self }
    pub(crate) fn with_depth(mut self, depth: usize) -> Self { self.depth = depth; self }
    pub(crate) fn with_loaded_modules(mut self, loaded: std::collections::HashSet<PathBuf>) -> Self { self.loaded_modules = Rc::new(loaded); self }
    pub(crate) fn with_extends(mut self, extends: Vec<(String, String, bool)>) -> Self { self.extends = Rc::new(extends); self }
    pub(crate) fn with_namespaces(mut self, ns: HashMap<String, Rc<ModuleExports>>) -> Self { self.namespaces = ns; self }
    pub(crate) fn with_pending_config(mut self, config: HashMap<String, Value>) -> Self { self.pending_config = config; self }
    pub(crate) fn add_pending_config(mut self, key: String, val: Value) -> Self { self.pending_config.insert(key, val); self }
    pub(crate) fn get_pending_config(&self) -> &HashMap<String, Value> { &self.pending_config }
    pub(crate) fn add_global_write(mut self, name: String, val: Value) -> Self { self.global_writes.insert(name, val); self }
    pub(crate) fn get_global_writes(&self) -> &HashMap<String, Value> { &self.global_writes }
    pub(crate) fn take_global_writes(&mut self) -> HashMap<String, Value> { std::mem::take(&mut self.global_writes) }
    pub(crate) fn get_base_path(&self) -> Option<&PathBuf> { self.base_path.as_ref() }
    pub(crate) fn get_depth(&self) -> usize { self.depth }
    pub(crate) fn is_plain_css(&self) -> bool { self.plain_css }
    pub(crate) fn get_local_vars(&self) -> &HashMap<String, Value> { &self.local_vars }
    pub(crate) fn get_local_mixins(&self) -> &HashMap<String, MixinDef> { &self.local_mixins }
    pub(crate) fn get_local_functions(&self) -> &HashMap<String, FunctionDef> { &self.local_functions }
    pub(crate) fn get_forwarded_vars(&self) -> &HashMap<String, Value> { &self.forwarded_vars }
    pub(crate) fn get_forwarded_mixins(&self) -> &HashMap<String, MixinDef> { &self.forwarded_mixins }
    pub(crate) fn get_forwarded_functions(&self) -> &HashMap<String, FunctionDef> { &self.forwarded_functions }
    pub(crate) fn get_namespaces(&self) -> &HashMap<String, Rc<ModuleExports>> { &self.namespaces }
    pub(crate) fn get_loaded_modules(&self) -> &std::collections::HashSet<PathBuf> { &self.loaded_modules }
    pub(crate) fn get_module_cache_rc(&self) -> Rc<HashMap<PathBuf, ModuleExports>> { self.module_cache.clone() }
    pub(crate) fn get_loaded_modules_rc(&self) -> Rc<std::collections::HashSet<PathBuf>> { self.loaded_modules.clone() }
    pub(crate) fn get_extends_rc(&self) -> Rc<Vec<(String, String, bool)>> { self.extends.clone() }
    pub(crate) fn merge_forwarded_to_local(mut self) -> Self {
        for (k, v) in self.forwarded_vars.iter().map(|(k, v)| (k.clone(), v.clone())) {
            self.local_vars.entry(k).or_insert(v);
        }
        for (k, v) in self.forwarded_mixins.iter().map(|(k, v)| (k.clone(), v.clone())) {
            self.local_mixins.entry(k).or_insert(v);
        }
        for (k, v) in self.forwarded_functions.iter().map(|(k, v)| (k.clone(), v.clone())) {
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

        // 传播命名空间变量赋值（名字含 . 的）
        for (name, val) in &rule_local_vars {
            if name.contains('.') {
                self.local_vars.insert(name.clone(), val.clone());
            }
        }
        // 传播 !global 变量赋值
        for (name, val) in &rule_global_writes {
            self.local_vars.insert(name.clone(), val.clone());
        }
        // 传播新增 mixin/function（规则体内定义的）
        for (name, def) in &rule_local_mixins {
            self.local_mixins.entry(name.clone()).or_insert_with(|| def.clone());
        }
        for (name, def) in &rule_local_functions {
            self.local_functions.entry(name.clone()).or_insert_with(|| def.clone());
        }
        // 传播新增 forwarded 成员
        for (name, def) in &rule_forwarded_mixins {
            self.forwarded_mixins.entry(name.clone()).or_insert_with(|| def.clone());
        }
        for (name, def) in &rule_forwarded_functions {
            self.forwarded_functions.entry(name.clone()).or_insert_with(|| def.clone());
        }
        for (name, val) in &rule_forwarded_vars {
            self.forwarded_vars.entry(name.clone()).or_insert_with(|| val.clone());
        }
        self
    }
}

/// 求值器。
pub struct Evaluator;
const MAX_DEPTH: usize = 100000;

impl Evaluator {
    pub fn evaluate(ast: &Ast) -> Result<Vec<CssNode>> {
        let (css, final_env) = Self::eval_nodes(&ast.nodes, Env::default())?;
        let extends = final_env.get_extends().to_vec();
        let css = if !extends.is_empty() {
            let css = Self::apply_extends(css, &extends);
            Self::check_extend_targets(&css, &extends)?;
            css
        } else {
            css
        };
        Ok(hoist_css_imports(css))
    }

    pub(crate) fn evaluate_with_env(ast: &Ast, env: Env) -> Result<Vec<CssNode>> {
        let (css, final_env) = Self::eval_nodes(&ast.nodes, env)?;
        let extends = final_env.get_extends().to_vec();
        let css = if !extends.is_empty() {
            let css = Self::apply_extends(css, &extends);
            Self::check_extend_targets(&css, &extends)?;
            css
        } else {
            css
        };
        Ok(hoist_css_imports(css))
    }

    /// CSS @import 提升策略——将所有 `@import` AtRule 提升到输出顶部。
    ///
    /// Sass 规范要求 CSS `@import`（`@import "file.css"`）出现在输出顶部，
    /// 保持源码中的相对顺序。此函数递归扫描 CSS 树，提取 @import 节点。
    /// 已改为自由函数 `hoist_css_imports`（消费 Vec 返回新 Vec）。


    #[cfg_attr(feature = "tracing", tracing::instrument(skip(nodes, env), fields(depth = env.get_depth(), n = nodes.len())))]
    fn eval_nodes(nodes: &[Node], env: Env) -> Result<(Vec<CssNode>, Env)> {
        if env.get_depth() > MAX_DEPTH {
            warn!(depth = env.get_depth(), "recursion limit exceeded");
            return Err(SassError::Eval("Recursion depth limit exceeded (possible infinite loop)".into()));
        }
        let mut css = Vec::new();
        let mut env = env;
        for node in nodes {
            let (mut out, new_env) = Self::eval_node(node, env).map_err(|e| {
                crate::__tracing::error!(error = %e, node_type = ?std::mem::discriminant(node), "eval_node failed");
                e
            })?;
            css.append(&mut out);
            env = new_env;
        }
        Ok((css, env))
    }

    /// 求值单个节点——纯函数分发，每个 arm 委托独立函数。
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(node, env), fields(depth = env.get_depth())))]
    fn eval_node(node: &Node, env: Env) -> Result<(Vec<CssNode>, Env)> {
        if env.is_plain_css() && !matches!(node, Node::Use { .. } | Node::Forward { .. } | Node::Import { .. }) {
            Self::check_plain_css_node(node)?;
        }
        match node {
            Node::Rule { selector, body } => {
                if env.is_plain_css() { Self::check_plain_css_selector(selector)?; }
                Self::eval_rule(selector, body, env)
            },
            Node::Decl { property, value, important } => eval_decl(property, value, *important, env),
            Node::Variable { name, value, flags } => Self::eval_variable(name, value, flags, env),
            Node::Comment(text, silent) => eval_comment(text, *silent, env),
            Node::If { branches, else_body } => Self::eval_if(branches, else_body, env),
            Node::For { var, from, to, inclusive, body } => Self::eval_for(var, from, to, *inclusive, body, env),
            Node::Each { vars, list, body } => Self::eval_each(vars, list, body, env),
            Node::While { cond, body } => Self::eval_while(cond, body, env),
            Node::MixinDef { name, params, body } => eval_mixin_def(name, params, body, env),
            Node::Include { name, args, content } => Self::eval_include(name, args, content, env),
            Node::Content => eval_content(env),
            Node::FunctionDef { name, params, body } => eval_func_def(name, params, body, env),
            Node::Return(v) => eval_return(v, env),
            Node::Use { url, namespace, star, config } => Self::eval_use(url, namespace, *star, config, env),
            Node::Forward { url, show, hide, prefix, config } => Self::eval_forward(url, prefix, config, env, show, hide),
            Node::Import { url, modifier } => Self::eval_import(url, modifier, env),
            Node::Extend { selector, optional } => eval_extend_node(selector, *optional, env),
            Node::AtRoot { query, body } => Self::eval_at_root(query, body, env),
            Node::AtRule { name, params, body } => Self::eval_at_rule(name, params, body, env),
            Node::Warn(v) => eval_warn(v, env),
            Node::Debug(v) => eval_debug(v, env),
            Node::Error(v) => eval_error_node(v, env),
        }
    }
}

/// 求值声明节点。
fn eval_decl(property: &str, value: &Value, important: bool, env: Env) -> Result<(Vec<CssNode>, Env)> {
    use crate::eval::Evaluator;
    if env.is_plain_css() {
        Evaluator::check_plain_css_value(value)?;
        if property.contains("#{") { return Err(SassError::Eval("Interpolation isn't allowed in plain CSS.".into())); }
    }
    // 顶层声明检测：不在样式规则内的裸声明是非法的
    if env.get_selector().is_none() {
        return Err(SassError::Eval("Declarations may only be used within style rules.".into()));
    }
    let val = Evaluator::eval_value(value, &env)?;
    if matches!(val, Value::Null) { return Ok((vec![], env)); }
    let property = crate::eval::value::eval_property_name(property, &env);
    Ok((vec![CssNode::Declaration { property, value: val.to_string(), important }], env))
}

/// 求值注释节点。
fn eval_comment(text: &str, silent: bool, env: Env) -> Result<(Vec<CssNode>, Env)> {
    if silent { Ok((vec![], env)) } else { Ok((vec![CssNode::Comment(text.to_string())], env)) }
}

/// 求值 mixin 定义。
fn eval_mixin_def(name: &str, params: &[Param], body: &[Node], env: Env) -> Result<(Vec<CssNode>, Env)> {
    let captured = env.get_namespaces().clone();
    Ok((vec![], env.define_mixin(name.to_string(), MixinDef { params: params.to_vec(), body: body.to_vec(), captured_namespaces: captured })))
}

/// 求值 @content 节点。
fn eval_content(env: Env) -> Result<(Vec<CssNode>, Env)> {
    use crate::eval::Evaluator;
    if let Some((content_nodes, content_env)) = env.get_content() {
        // @content 在 mixin body 内执行，继承当前 current_selector
        // content_env 是 Rc<Env> 引用，需要 clone 创建副本（@content 上下文快照例外）
        let content_env = content_env.clone().with_selector(
            env.get_selector().map(|s| s.to_string()).unwrap_or_default()
        );
        let content_nodes = content_nodes.to_vec();
        Evaluator::eval_nodes(&content_nodes, content_env)
    } else { Ok((vec![], env)) }
}

/// 求值函数定义。
fn eval_func_def(name: &str, params: &[Param], body: &[Node], env: Env) -> Result<(Vec<CssNode>, Env)> {
    let captured = env.get_namespaces().clone();
    Ok((vec![], env.define_function(name.to_string(), FunctionDef { params: params.to_vec(), body: body.to_vec(), captured_namespaces: captured })))
}

/// 求值 @return 节点。
fn eval_return(v: &Value, env: Env) -> Result<(Vec<CssNode>, Env)> {
    use crate::eval::Evaluator;
    let val = Evaluator::eval_value(v, &env)?;
    Ok((vec![CssNode::Return(val)], env))
}

/// 求值 @extend 节点。
fn eval_extend_node(selector: &str, optional: bool, env: Env) -> Result<(Vec<CssNode>, Env)> {
    if let Some(extender) = env.get_selector().map(|s| s.to_string()) {
        Ok((vec![], env.add_extend(extender, selector.to_string(), optional)))
    } else { Ok((vec![], env)) }
}

/// 求值 @warn 节点。
fn eval_warn(_v: &Value, env: Env) -> Result<(Vec<CssNode>, Env)> {
    Ok((vec![], env))
}

/// 求值 @debug 节点。
fn eval_debug(_v: &Value, env: Env) -> Result<(Vec<CssNode>, Env)> {
    Ok((vec![], env))
}

/// 求值 @error 节点。
fn eval_error_node(v: &Value, env: Env) -> Result<(Vec<CssNode>, Env)> {
    use crate::eval::Evaluator;
    let msg = Evaluator::eval_value(v, &env)?;
    Err(SassError::Eval(msg.to_string()))
}

/// CSS @import 提升——纯函数版（消费 Vec 返回新 Vec）。
fn hoist_css_imports(nodes: Vec<CssNode>) -> Vec<CssNode> {
    let span = crate::__tracing::debug_span!("hoist_css_imports", n = nodes.len());
    let _enter = span.enter();
    let mut imports = Vec::new();
    let mut rest = Vec::new();
    for node in nodes {
        // 先递归处理嵌套节点，再判断是否为 @import
        let node = match node {
            CssNode::AtRule { name, params, children, has_body: true } => {
                let children = hoist_css_imports(children);
                CssNode::AtRule { name, params, children, has_body: true }
            }
            CssNode::AtRoot(kids) => {
                CssNode::AtRoot(hoist_css_imports(kids))
            }
            other => other,
        };
        // 判断是否为 CSS @import（无 body 的 @import AtRule）
        let is_css_import = matches!(
            &node,
            CssNode::AtRule { name, has_body: false, .. } if name == "import"
        );
        if is_css_import {
            imports.push(node);
        } else {
            rest.push(node);
        }
    }
    if !imports.is_empty() {
        crate::__tracing::debug!(n_imports = imports.len(), "hoisted css imports");
    }
    let mut result = imports;
    result.extend(rest);
    result
}

mod at_params;
mod builtin;
mod color;
mod control_flow;
mod extend;
mod file_resolver;
mod import;
mod meta_ops;
mod mixin;
mod module;
// module_dispatch 已被 builtin::dispatch 替代
// mod module_dispatch;
mod module_helpers;
mod module_validation;
mod plain_css;
mod rule;
mod value;
