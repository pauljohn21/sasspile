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
    pub(crate) extends: Rc<Vec<(String, String)>>,
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
    extends: Rc<Vec<(String, String)>>,
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
    pub fn add_extend(mut self, extender: String, target: String) -> Self { Rc::make_mut(&mut self.extends).push((extender, target)); self }
    pub fn get_extends(&self) -> &[(String, String)] { &self.extends }
    pub fn with_selector(mut self, sel: String) -> Self { self.current_selector = Some(sel); self }
    pub fn get_selector(&self) -> Option<&str> { self.current_selector.as_deref() }
    pub fn with_load_paths(mut self, paths: Vec<PathBuf>) -> Self { self.load_paths = paths; self }
    pub(crate) fn get_load_paths(&self) -> &[PathBuf] { &self.load_paths }
    pub(crate) fn get_module_cache(&self) -> &HashMap<PathBuf, ModuleExports> { &self.module_cache }
    pub(crate) fn with_module_cache(mut self, cache: HashMap<PathBuf, ModuleExports>) -> Self { self.module_cache = Rc::new(cache); self }
    pub fn with_plain_css(mut self, plain_css: bool) -> Self { self.plain_css = plain_css; self }
}

/// 求值器。
pub struct Evaluator;
const MAX_DEPTH: usize = 100000;

impl Evaluator {
    pub fn evaluate(ast: &Ast) -> Result<Vec<CssNode>> {
        let (mut css, final_env) = Self::eval_nodes(&ast.nodes, Env::default())?;
        let extends = final_env.get_extends().to_vec();
        if !extends.is_empty() { Self::apply_extends(&mut css, &extends); }
        Ok(css)
    }

    pub(crate) fn evaluate_with_env(ast: &Ast, env: Env) -> Result<Vec<CssNode>> {
        let (mut css, final_env) = Self::eval_nodes(&ast.nodes, env)?;
        let extends = final_env.get_extends().to_vec();
        if !extends.is_empty() { Self::apply_extends(&mut css, &extends); }
        Ok(css)
    }

    /// 求值节点列表——for 循环 + move（零 clone）。
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(nodes, env), fields(depth = env.depth, n = nodes.len())))]
    fn eval_nodes(nodes: &[Node], env: Env) -> Result<(Vec<CssNode>, Env)> {
        if env.depth > MAX_DEPTH {
            warn!(depth = env.depth, "recursion limit exceeded");
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

    /// 求值单个节点。
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(node, env), fields(depth = env.depth)))]
    fn eval_node(node: &Node, env: Env) -> Result<(Vec<CssNode>, Env)> {
        if env.plain_css && !matches!(node, Node::Use { .. } | Node::Forward { .. } | Node::Import { .. }) {
            Self::check_plain_css_node(node)?;
        }
        match node {
            Node::Rule { selector, body } => {
                if env.plain_css { Self::check_plain_css_selector(selector)?; }
                Self::eval_rule(selector, body, env)
            },
            Node::Decl { property, value, important } => {
                if env.plain_css {
                    Self::check_plain_css_value(value)?;
                    if property.contains("#{") { return Err(SassError::Eval("Interpolation isn't allowed in plain CSS.".into())); }
                }
                let val = Self::eval_value(value, &env)?;
                if matches!(val, Value::Null) { return Ok((vec![], env)); }
                let property = crate::eval::value::eval_property_name(property, &env);
                Ok((vec![CssNode::Declaration { property, value: val.to_string(), important: *important }], env))
            }
            Node::Variable { name, value, flags } => Self::eval_variable(name, value, flags, env),
            Node::Comment(text, silent) => {
                if *silent { Ok((vec![], env)) } else { Ok((vec![CssNode::Comment(text.clone())], env)) }
            }
            Node::If { branches, else_body } => Self::eval_if(branches, else_body, env),
            Node::For { var, from, to, inclusive, body } => Self::eval_for(var, from, to, *inclusive, body, env),
            Node::Each { vars, list, body } => Self::eval_each(vars, list, body, env),
            Node::While { cond, body } => Self::eval_while(cond, body, env),
            Node::MixinDef { name, params, body } => {
                let captured = env.namespaces.clone();
                Ok((vec![], env.define_mixin(name.clone(), MixinDef { params: params.clone(), body: body.clone(), captured_namespaces: captured })))
            }
            Node::Include { name, args, content } => Self::eval_include(name, args, content, env),
            Node::Content => {
                if let Some((content_nodes, content_env)) = env.get_content() {
                    let content_env = content_env.clone();
                    let content_nodes = content_nodes.to_vec();
                    Self::eval_nodes(&content_nodes, content_env)
                } else { Ok((vec![], env)) }
            }
            Node::FunctionDef { name, params, body } => {
                let captured = env.namespaces.clone();
                Ok((vec![], env.define_function(name.clone(), FunctionDef { params: params.clone(), body: body.clone(), captured_namespaces: captured })))
            }
            Node::Return(v) => {
                let val = Self::eval_value(v, &env)?;
                Ok((vec![CssNode::Return(val)], env))
            }
            Node::Use { url, namespace, star, config } => Self::eval_use(url, namespace, *star, config, env),
            Node::Forward { url, show, hide, prefix, config } => Self::eval_forward(url, prefix, config, env, show, hide),
            Node::Import { url, modifier } => Self::eval_import(url, modifier, env),
            Node::Extend { selector, optional: _ } => {
                if let Some(extender) = env.get_selector().map(|s| s.to_string()) {
                    Ok((vec![], env.add_extend(extender, selector.clone())))
                } else { Ok((vec![], env)) }
            }
            Node::AtRoot { query, body } => Self::eval_at_root(query, body, env),
            Node::AtRule { name, params, body } => Self::eval_at_rule(name, params, body, env),
            Node::Warn(_) | Node::Debug(_) => Ok((vec![], env)),
            Node::Error(v) => {
                let msg = Self::eval_value(v, &env)?;
                Err(SassError::Eval(msg.to_string()))
            }
        }
    }
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
mod module_dispatch;
mod module_helpers;
mod module_validation;
mod plain_css;
mod rule;
mod value;
