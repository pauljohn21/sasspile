//! 求值器——纯函数式 try_fold + 不可变环境。

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
    /// 模块文件内部定义 + @use as * 导入的成员（当前文件可见）。
    pub(crate) local_vars: HashMap<String, Value>,
    pub(crate) local_mixins: HashMap<String, MixinDef>,
    pub(crate) local_functions: HashMap<String, FunctionDef>,
    /// @forward 导出的成员（当前文件不可见，只传递给下游）。
    pub(crate) forwarded_vars: HashMap<String, Value>,
    pub(crate) forwarded_mixins: HashMap<String, MixinDef>,
    pub(crate) forwarded_functions: HashMap<String, FunctionDef>,
    pub(crate) css: Vec<CssNode>,
    /// 模块加载过程中发现的已加载路径（用于缓存传播）。
    pub(crate) loaded_modules: Rc<std::collections::HashSet<PathBuf>>,
    /// 模块中收集的 @extend 关系——需要传播到顶层 CSS。
    pub(crate) extends: Rc<Vec<(String, String)>>,
    /// 模块 exports 缓存——用于跨模块传播。
    pub(crate) module_cache: Rc<HashMap<PathBuf, ModuleExports>>,
}

impl ModuleExports {
    /// 合并迭代器：local 优先于 forwarded（供 meta 反射用）。
    pub(crate) fn all_functions(&self) -> impl Iterator<Item = (&String, &FunctionDef)> {
        self.local_functions.iter().chain(
            self.forwarded_functions.iter().filter(|(k, _)| !self.local_functions.contains_key(*k))
        )
    }
    /// 合并迭代器：local 优先于 forwarded（供 meta 反射用）。
    pub(crate) fn all_mixins(&self) -> impl Iterator<Item = (&String, &MixinDef)> {
        self.local_mixins.iter().chain(
            self.forwarded_mixins.iter().filter(|(k, _)| !self.local_mixins.contains_key(*k))
        )
    }
    /// 合并迭代器：local 优先于 forwarded（供 meta 反射用）。
    pub(crate) fn all_vars(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.local_vars.iter().chain(
            self.forwarded_vars.iter().filter(|(k, _)| !self.local_vars.contains_key(*k))
        )
    }
}

/// 不可变求值环境。
#[derive(Debug, Clone, Default)]
pub struct Env {
    /// —— local：当前文件定义 + @use as * 导入（当前文件可见）——
    local_vars: HashMap<String, Value>,
    local_mixins: HashMap<String, MixinDef>,
    local_functions: HashMap<String, FunctionDef>,
    /// —— forwarded：@forward 导出（当前文件不可见，只传递给下游）——
    forwarded_vars: HashMap<String, Value>,
    forwarded_mixins: HashMap<String, MixinDef>,
    forwarded_functions: HashMap<String, FunctionDef>,
    /// !global 变量写入——规则体内 !global 赋值需要传播到外层。
    global_writes: HashMap<String, Value>,
    /// @content 内容块（Rc 共享，避免深拷贝）。
    content: Option<Rc<Vec<Node>>>,
    /// @content 的环境（Rc 共享，避免深拷贝）。
    content_env: Option<Rc<Env>>,
    /// 已加载的内建模块名集合。
    builtin_modules: Vec<String>,
    /// 命名空间模块（文件加载的模块）。
    pub(crate) namespaces: HashMap<String, Rc<ModuleExports>>,
    /// 当前文件路径（用于解析相对 @use/@import）。
    base_path: Option<PathBuf>,
    /// 递归深度计数器。
    depth: usize,
    /// @extend 收集的继承关系 (extender, target)——Rc 共享避免深拷贝。
    extends: Rc<Vec<(String, String)>>,
    /// 当前选择器上下文（进入规则体时设置）。
    current_selector: Option<String>,
    /// 加载路径——`@use`/`@import` 无法从当前文件解析时回退搜索。
    load_paths: Vec<PathBuf>,
    /// plain CSS 模式——`.css` 文件加载时设为 true，不展开选择器。
    plain_css: bool,
    /// 已加载模块的路径集合——同一模块的 CSS 只输出一次。
    loaded_modules: Rc<std::collections::HashSet<PathBuf>>,
    /// 模块 exports 缓存——同一路径的模块只求值一次，后续 @use/@forward 从缓存取 vars/mixins/functions。
    module_cache: Rc<HashMap<PathBuf, ModuleExports>>,
    /// `with()` 配置变量——不进入 local_vars，在 !default 赋值时消费。
    /// key 不带 $ 前缀，value 是配置值。
    pending_config: HashMap<String, Value>,
}

/// mixin 定义存储。
#[derive(Debug, Clone)]
pub(crate) struct MixinDef {
    pub(crate) params: Vec<Param>,
    pub(crate) body: Vec<Node>,
    /// mixin 定义时捕获的命名空间（使 mixin 体可访问定义模块的 @use 命名空间）。
    pub(crate) captured_namespaces: HashMap<String, Rc<ModuleExports>>,
}

/// 函数定义存储。
#[derive(Debug, Clone)]
pub(crate) struct FunctionDef {
    pub(crate) params: Vec<Param>,
    pub(crate) body: Vec<Node>,
    /// 函数定义时捕获的命名空间（使函数体可访问定义模块的 @use 命名空间）。
    pub(crate) captured_namespaces: HashMap<String, Rc<ModuleExports>>,
}

impl Env {
    /// 创建空环境。
    pub fn new_env() -> Self {
        Self::default()
    }
    /// 递增深度。
    pub fn incr_depth(&self) -> Self {
        let mut new = self.clone();
        new.depth += 1;
        new
    }
    /// 不可变插入变量绑定，返回新环境。
    pub fn bind(&self, name: String, value: Value) -> Self {
        let mut new = self.clone();
        new.local_vars.insert(name, value);
        new
    }
    /// 按名查找变量引用（只查 local 表——forwarded 不可见）。
    pub fn lookup(&self, name: &str) -> Option<&Value> {
        self.local_vars.get(name)
    }
    /// 判断变量是否已定义（只查 local 表）。
    pub fn has_var(&self, name: &str) -> bool {
        self.local_vars.contains_key(name)
    }
    /// 定义 local mixin（@mixin 节点定义的是当前文件成员）。
    pub(crate) fn define_mixin(&self, name: String, def: MixinDef) -> Self {
        self.define_local_mixin(name, def)
    }
    /// 定义 local mixin。
    pub(crate) fn define_local_mixin(&self, name: String, def: MixinDef) -> Self {
        let mut new = self.clone();
        new.local_mixins.insert(name, def);
        new
    }
    /// 定义 forwarded mixin。
    pub(crate) fn define_forwarded_mixin(&self, name: String, def: MixinDef) -> Self {
        let mut new = self.clone();
        new.forwarded_mixins.insert(name, def);
        new
    }
    pub(crate) fn get_mixin(&self, name: &str) -> Option<&MixinDef> {
        self.local_mixins.get(name)
    }
    /// 获取 mixin 引用数据（用于 meta.get-mixin）。
    /// 返回 mixin 的参数、体和捕获的命名空间键列表。
    pub(crate) fn get_mixin_ref_data(&self, name: &str) -> Option<(Vec<Param>, Vec<Node>, Vec<String>)> {
        self.local_mixins.get(name).map(|m| {
            let ns_keys: Vec<String> = m.captured_namespaces.keys().cloned().collect();
            (m.params.clone(), m.body.clone(), ns_keys)
        })
    }
    /// 定义 local function（@function 节点定义的是当前文件成员）。
    pub(crate) fn define_function(&self, name: String, def: FunctionDef) -> Self {
        self.define_local_function(name, def)
    }
    /// 定义 local function。
    pub(crate) fn define_local_function(&self, name: String, def: FunctionDef) -> Self {
        let mut new = self.clone();
        new.local_functions.insert(name, def);
        new
    }
    /// 定义 forwarded function。
    pub(crate) fn define_forwarded_function(&self, name: String, def: FunctionDef) -> Self {
        let mut new = self.clone();
        new.forwarded_functions.insert(name, def);
        new
    }
    pub(crate) fn get_function(&self, name: &str) -> Option<&FunctionDef> {
        self.local_functions.get(name)
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
    /// 注册已加载内建模块，并注册模块变量到命名空间。
    pub fn add_module(&self, name: String) -> Self {
        let mut new = self.clone();
        if !new.builtin_modules.contains(&name) {
            new.builtin_modules.push(name.clone());
        }
        // 注册内建模块的变量到命名空间（如 math.$pi, math.$e）
        // 命名空间名是 url 中 "sass:" 后的部分（如 "sass:math" → "math"）
        let ns_name = name.strip_prefix("sass:").unwrap_or(&name).to_string();
        if let Some(exports) = module::builtin_module_exports(&name) {
            new.namespaces.insert(ns_name, Rc::new(exports));
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
    /// 设置加载路径。
    pub fn with_load_paths(&self, paths: Vec<PathBuf>) -> Self {
        let mut new = self.clone();
        new.load_paths = paths;
        new
    }
    /// 获取加载路径。
    pub(crate) fn get_load_paths(&self) -> &[PathBuf] {
        &self.load_paths
    }
    /// 获取模块 exports 缓存。
    pub(crate) fn get_module_cache(&self) -> &HashMap<PathBuf, ModuleExports> {
        &self.module_cache
    }
    /// 更新模块 exports 缓存。
    pub(crate) fn with_module_cache(&self, cache: HashMap<PathBuf, ModuleExports>) -> Self {
        let mut new = self.clone();
        new.module_cache = Rc::new(cache);
        new
    }

    /// 设置 plain CSS 模式标志（.css 文件加载时设为 true）。
    pub fn with_plain_css(&self, plain_css: bool) -> Self {
        let mut new = self.clone();
        new.plain_css = plain_css;
        new
    }
}

/// 求值器。
pub struct Evaluator;

/// 最大递归深度——防止无限递归导致内存爆炸。
const MAX_DEPTH: usize = 100000;

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
        let is_plain_css = base_path.extension().and_then(|e| e.to_str()) == Some("css");
        let env = Env::default().with_base_path(base_path).with_plain_css(is_plain_css);
        Self::evaluate_with_env(ast, env)
    }

    /// 求值 AST 入口（带基础路径和加载路径）。
    pub fn evaluate_with_path_and_load_paths(
        ast: &Ast,
        base_path: PathBuf,
        load_paths: Vec<PathBuf>,
    ) -> Result<Vec<CssNode>> {
        let env = Env::default()
            .with_base_path(base_path)
            .with_load_paths(load_paths);
        Self::evaluate_with_env(ast, env)
    }

    /// 求值 AST 入口（带环境）。
    fn evaluate_with_env(ast: &Ast, env: Env) -> Result<Vec<CssNode>> {
        let (mut css, final_env) = Self::eval_nodes(&ast.nodes, &env)?;
        let extends = final_env.get_extends().to_vec();
        if !extends.is_empty() {
            Self::apply_extends(&mut css, &extends);
        }
        Ok(css)
    }

    /// 求值节点列表——try_fold。
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(nodes, env), fields(depth = env.depth, n = nodes.len())))]
    fn eval_nodes(nodes: &[Node], env: &Env) -> Result<(Vec<CssNode>, Env)> {
        if env.depth > MAX_DEPTH {
            warn!(depth = env.depth, "recursion limit exceeded");
            return Err(SassError::Eval("Recursion depth limit exceeded (possible infinite loop)".into()));
        }
        nodes.iter().try_fold((Vec::new(), env.clone()), |(mut css, env), node| {
            let node_span = crate::__tracing::debug_span!("eval_node_item", node = ?std::mem::discriminant(node));
            let _enter = node_span.enter();
            let (mut out, new_env) = Self::eval_node(node, &env).map_err(|e| {
                crate::__tracing::error!(error = %e, node_type = ?std::mem::discriminant(node), "eval_node failed");
                e
            })?;
            css.append(&mut out);
            Ok((css, new_env))
        })
    }

    /// 求值单个节点。
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(node, env), fields(depth = env.depth)))]
    fn eval_node(node: &Node, env: &Env) -> Result<(Vec<CssNode>, Env)> {
        // plain CSS 模式——检查节点合法性（Use/Forward/Import 除外，由模块系统处理）
        if env.plain_css && !matches!(node, Node::Use { .. } | Node::Forward { .. } | Node::Import { .. }) {
            Self::check_plain_css_node(node)?;
        }
        match node {
            Node::Rule { selector, body } => {
                // plain CSS 模式——检查选择器合法性
                if env.plain_css {
                    Self::check_plain_css_selector(selector)?;
                }
                Self::eval_rule(selector, body, env)
            },
            Node::Decl {
                property,
                value,
                important,
            } => {
                // plain CSS 模式——检查值和属性名中的非法表达式
                if env.plain_css {
                    Self::check_plain_css_value(value)?;
                    if property.contains("#{") {
                        return Err(SassError::Eval(
                            "Interpolation isn't allowed in plain CSS.".into(),
                        ));
                    }
                }
                let val = Self::eval_value(value, env)?;
                // SCSS 中 null 值声明不输出
                if matches!(val, Value::Null) {
                    return Ok((vec![], env.clone()));
                }
                // 求值属性名（支持 $var 和 #{...} 插值）
                let property = crate::eval::value::eval_property_name(property, env);
                Ok((
                    vec![CssNode::Declaration {
                        property,
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
                        captured_namespaces: env.namespaces.clone(),
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
                        captured_namespaces: env.namespaces.clone(),
                    },
                );
                Ok((vec![], new_env))
            }
            Node::Return(v) => {
                let val = Self::eval_value(v, env)?;
                Ok((vec![CssNode::Return(val)], env.clone()))
            }
            Node::Use {
                url,
                namespace,
                star,
                config,
            } => Self::eval_use(url, namespace, *star, config, env),
            Node::Forward {
                url,
                show,
                hide,
                prefix,
                config,
            } => Self::eval_forward(url, prefix, config, env, show, hide),
            Node::Import { url, modifier } => Self::eval_import(url, modifier, env),
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

mod at_params;
mod builtin;
mod color;
mod control_flow;
mod extend;
mod import;
mod meta_ops;
mod mixin;
mod module;
mod module_dispatch;
mod plain_css;
mod rule;
mod value;
