//! 求值器——纯函数式 try_fold + 不可变环境。

use crate::css::node::CssNode;
use crate::error::{Result, SassError};
use crate::lex::Lexer;
use crate::lex::token::Token;
use crate::parse::ast::*;
use tracing::{instrument, warn};

use im::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

/// 模块导出——加载的文件模块的成员。
#[derive(Debug, Clone, Default)]
pub(crate) struct ModuleExports {
    vars: HashMap<String, Value>,
    mixins: HashMap<String, MixinDef>,
    functions: HashMap<String, FunctionDef>,
    #[allow(dead_code)]
    css: Vec<CssNode>,
}

/// 不可变求值环境。
#[derive(Debug, Clone, Default)]
pub struct Env {
    /// 变量绑定（扁平，用作用域前缀模拟）。
    vars: HashMap<String, Value>,
    /// mixin 定义。
    mixins: HashMap<String, MixinDef>,
    /// 用户函数定义。
    functions: HashMap<String, FunctionDef>,
    /// @content 内容块（Rc 共享，避免深拷贝）。
    content: Option<Rc<Vec<Node>>>,
    /// @content 的环境（Rc 共享，避免深拷贝）。
    content_env: Option<Rc<Env>>,
    /// 已加载的内建模块名集合。
    builtin_modules: Vec<String>,
    /// 命名空间模块（文件加载的模块）。
    namespaces: HashMap<String, Rc<ModuleExports>>,
    /// 当前文件路径（用于解析相对 @use/@import）。
    base_path: Option<PathBuf>,
    /// 递归深度计数器。
    depth: usize,
    /// @extend 收集的继承关系 (extender, target)——Rc 共享避免深拷贝。
    extends: Rc<Vec<(String, String)>>,
    /// 当前选择器上下文（进入规则体时设置）。
    current_selector: Option<String>,
}

/// mixin 定义存储。
#[derive(Debug, Clone)]
pub(crate) struct MixinDef {
    params: Vec<Param>,
    body: Vec<Node>,
}

/// 函数定义存储。
#[derive(Debug, Clone)]
pub(crate) struct FunctionDef {
    params: Vec<Param>,
    body: Vec<Node>,
}

impl Env {
    pub fn new_env() -> Self {
        Self::default()
    }
    /// 递增深度。
    pub fn incr_depth(&self) -> Self {
        let mut new = self.clone();
        new.depth += 1;
        new
    }
    pub fn bind(&self, name: String, value: Value) -> Self {
        let mut new = self.clone();
        new.vars.insert(name, value);
        new
    }
    pub fn lookup(&self, name: &str) -> Option<&Value> {
        self.vars.get(name)
    }
    pub fn has_var(&self, name: &str) -> bool {
        self.vars.contains_key(name)
    }
    pub(crate) fn define_mixin(&self, name: String, def: MixinDef) -> Self {
        let mut new = self.clone();
        new.mixins.insert(name, def);
        new
    }
    pub(crate) fn get_mixin(&self, name: &str) -> Option<&MixinDef> {
        self.mixins.get(name)
    }
    pub(crate) fn define_function(&self, name: String, def: FunctionDef) -> Self {
        let mut new = self.clone();
        new.functions.insert(name, def);
        new
    }
    pub(crate) fn get_function(&self, name: &str) -> Option<&FunctionDef> {
        self.functions.get(name)
    }
    /// 设置 @content 内容块。
    pub fn set_content(&self, content: Vec<Node>, content_env: Env) -> Self {
        let mut new = self.clone();
        new.content = Some(Rc::new(content));
        new.content_env = Some(Rc::new(content_env));
        new
    }
    /// 获取 @content 内容块。
    pub fn get_content(&self) -> Option<(&[Node], &Env)> {
        self.content
            .as_ref()
            .map(|c| c.as_slice())
            .zip(self.content_env.as_ref().map(|e| e.as_ref()))
    }
    /// 注册已加载内建模块。
    pub fn add_module(&self, name: String) -> Self {
        let mut new = self.clone();
        if !new.builtin_modules.contains(&name) {
            new.builtin_modules.push(name);
        }
        new
    }
    /// 检查内建模块是否已加载。
    pub fn has_module(&self, name: &str) -> bool {
        self.builtin_modules.iter().any(|m| m == name)
    }
    /// 添加命名空间模块。
    pub(crate) fn add_namespace(&self, ns: String, exports: ModuleExports) -> Self {
        let mut new = self.clone();
        new.namespaces.insert(ns, Rc::new(exports));
        new
    }
    /// 获取命名空间模块。
    pub(crate) fn get_namespace(&self, ns: &str) -> Option<&ModuleExports> {
        self.namespaces.get(ns).map(|rc| rc.as_ref())
    }
    /// 设置基础路径。
    pub fn with_base_path(&self, path: PathBuf) -> Self {
        let mut new = self.clone();
        new.base_path = Some(path);
        new
    }
    /// 添加 @extend 关系。
    pub fn add_extend(&self, extender: String, target: String) -> Self {
        let mut new = self.clone();
        let mut extends = (*self.extends).clone();
        extends.push((extender, target));
        new.extends = Rc::new(extends);
        new
    }
    /// 获取所有 @extend 关系。
    pub fn get_extends(&self) -> &[(String, String)] {
        &self.extends
    }
    /// 设置当前选择器。
    pub fn with_selector(&self, sel: String) -> Self {
        let mut new = self.clone();
        new.current_selector = Some(sel);
        new
    }
    /// 获取当前选择器。
    pub fn get_selector(&self) -> Option<&str> {
        self.current_selector.as_deref()
    }
}

/// 求值器。
pub struct Evaluator;

/// 最大递归深度——防止无限递归导致内存爆炸。
const MAX_DEPTH: usize = 200;

impl Evaluator {
    /// 求值 AST 入口。
    pub fn evaluate(ast: &Ast) -> Result<Vec<CssNode>> {
        let (mut css, final_env) = Self::eval_nodes(&ast.nodes, &Env::default())?;
        let extends = final_env.get_extends().to_vec();
        if !extends.is_empty() {
            Self::apply_extends(&mut css, &extends);
        }
        Ok(css)
    }

    /// 求值 AST 入口（带基础路径，支持文件加载）。
    pub fn evaluate_with_path(ast: &Ast, base_path: PathBuf) -> Result<Vec<CssNode>> {
        let env = Env::default().with_base_path(base_path);
        let (mut css, final_env) = Self::eval_nodes(&ast.nodes, &env)?;
        let extends = final_env.get_extends().to_vec();
        if !extends.is_empty() {
            Self::apply_extends(&mut css, &extends);
        }
        Ok(css)
    }

    /// 求值节点列表——try_fold。
    #[instrument(skip(nodes, env), fields(depth = env.depth, n = nodes.len()))]
    fn eval_nodes(nodes: &[Node], env: &Env) -> Result<(Vec<CssNode>, Env)> {
        if env.depth > MAX_DEPTH {
            warn!(depth = env.depth, "recursion limit exceeded");
            return Err(SassError::Eval("递归深度超过限制（可能是无限循环）".into()));
        }
        nodes.iter().try_fold((Vec::new(), env.clone()), |(mut css, env), node| {
            let node_span = tracing::debug_span!("eval_node_item", node = ?std::mem::discriminant(node));
            let _enter = node_span.enter();
            let (mut out, new_env) = Self::eval_node(node, &env).map_err(|e| {
                tracing::error!(error = %e, node_type = ?std::mem::discriminant(node), "eval_node failed");
                e
            })?;
            css.append(&mut out);
            Ok((css, new_env))
        })
    }

    /// 求值单个节点。
    #[instrument(skip(node, env), fields(depth = env.depth))]
    fn eval_node(node: &Node, env: &Env) -> Result<(Vec<CssNode>, Env)> {
        match node {
            Node::Rule { selector, body } => Self::eval_rule(selector, body, env),
            Node::Decl {
                property,
                value,
                important,
            } => {
                let val = Self::eval_value(value, env)?;
                Ok((
                    vec![CssNode::Declaration {
                        property: property.clone(),
                        value: val.to_string(),
                        important: *important,
                    }],
                    env.clone(),
                ))
            }
            Node::Variable { name, value, flags } => Self::eval_variable(name, value, flags, env),
            Node::Comment(text, silent) => {
                if *silent {
                    Ok((vec![], env.clone()))
                } else {
                    Ok((vec![CssNode::Comment(text.clone())], env.clone()))
                }
            }
            Node::If {
                branches,
                else_body,
            } => Self::eval_if(branches, else_body, env),
            Node::For {
                var,
                from,
                to,
                inclusive,
                body,
            } => Self::eval_for(var, from, to, *inclusive, body, env),
            Node::Each { vars, list, body } => Self::eval_each(vars, list, body, env),
            Node::While { cond, body } => Self::eval_while(cond, body, env),
            Node::MixinDef { name, params, body } => {
                let new_env = env.define_mixin(
                    name.clone(),
                    MixinDef {
                        params: params.clone(),
                        body: body.clone(),
                    },
                );
                Ok((vec![], new_env))
            }
            Node::Include {
                name,
                args,
                content,
            } => Self::eval_include(name, args, content, env),
            Node::Content => {
                if let Some((content_nodes, content_env)) = env.get_content() {
                    Self::eval_nodes(content_nodes, content_env)
                } else {
                    Ok((vec![], env.clone()))
                }
            }
            Node::FunctionDef { name, params, body } => {
                let new_env = env.define_function(
                    name.clone(),
                    FunctionDef {
                        params: params.clone(),
                        body: body.clone(),
                    },
                );
                Ok((vec![], new_env))
            }
            Node::Return(_) => Ok((vec![], env.clone())), // @return 由函数调用处理
            Node::Use {
                url,
                namespace,
                star,
                config,
            } => {
                // 内建模块 sass:math/string/list/map/color/meta/selector
                if url.starts_with("sass:") {
                    return Ok((vec![], env.add_module(url.clone())));
                }
                // 文件模块——解析路径并加载
                let base = env.base_path.as_ref();
                if let Some(path) = Self::resolve_file(base, url) {
                    let exports = Self::load_module(&path, config, env)?;
                    if *star {
                        let mut new_env = env.clone();
                        for (k, v) in &exports.vars {
                            new_env = new_env.bind(k.clone(), v.clone());
                        }
                        for (k, v) in &exports.mixins {
                            new_env = new_env.define_mixin(k.clone(), v.clone());
                        }
                        for (k, v) in &exports.functions {
                            new_env = new_env.define_function(k.clone(), v.clone());
                        }
                        return Ok((vec![], new_env));
                    }
                    let ns = namespace.clone().unwrap_or_else(|| {
                        // 默认命名空间 = 文件名（不含扩展名和前缀 _）
                        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(url);
                        stem.trim_start_matches('_').to_string()
                    });
                    return Ok((vec![], env.add_namespace(ns, exports)));
                }
                // 找不到文件——静默跳过
                Ok((vec![], env.clone()))
            }
            Node::Forward {
                url,
                show: _,
                hide: _,
                prefix: _,
            } => {
                // @forward 'url' —— 转发模块成员到当前作用域（简化版：同 @import）
                let base = env.base_path.as_ref();
                if let Some(path) = Self::resolve_file(base, url) {
                    let exports = Self::load_module(&path, &[], env)?;
                    let mut new_env = env.clone();
                    for (k, v) in &exports.vars {
                        new_env = new_env.bind(k.clone(), v.clone());
                    }
                    for (k, v) in &exports.mixins {
                        new_env = new_env.define_mixin(k.clone(), v.clone());
                    }
                    for (k, v) in &exports.functions {
                        new_env = new_env.define_function(k.clone(), v.clone());
                    }
                    return Ok((exports.css, new_env));
                }
                Ok((vec![], env.clone()))
            }
            Node::Import { url } => {
                // @import 'url' —— 旧版内联：加载文件内容注入当前作用域
                if url.starts_with("sass:") {
                    return Ok((vec![], env.add_module(url.clone())));
                }
                let base = env.base_path.as_ref();
                if let Some(path) = Self::resolve_file(base, url) {
                    let exports = Self::load_module(&path, &[], env)?;
                    let mut new_env = env.clone();
                    for (k, v) in &exports.vars {
                        new_env = new_env.bind(k.clone(), v.clone());
                    }
                    for (k, v) in &exports.mixins {
                        new_env = new_env.define_mixin(k.clone(), v.clone());
                    }
                    for (k, v) in &exports.functions {
                        new_env = new_env.define_function(k.clone(), v.clone());
                    }
                    return Ok((exports.css, new_env));
                }
                Ok((vec![], env.clone()))
            }
            Node::Extend {
                selector,
                optional: _,
            } => {
                // @extend selector —— 收集继承关系
                if let Some(extender) = env.get_selector() {
                    let new_env = env.add_extend(extender.to_string(), selector.clone());
                    Ok((vec![], new_env))
                } else {
                    Ok((vec![], env.clone()))
                }
            }
            Node::AtRoot { query, body } => Self::eval_at_root(query, body, env),
            Node::AtRule { name, params, body } => Self::eval_at_rule(name, params, body, env),
            Node::Warn(_) | Node::Debug(_) => Ok((vec![], env.clone())),
            Node::Error(v) => {
                let msg = Self::eval_value(v, env)?;
                Err(SassError::Eval(msg.to_string()))
            }
        }
    }
}

mod builtin;
mod color;
mod control_flow;
mod extend;
mod mixin;
mod module;
mod rule;
mod value;
