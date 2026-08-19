//! 求值器——纯函数式 try_fold + 不可变环境。

use crate::css::node::CssNode;
use crate::error::{Result, SassError};
use crate::lex::Lexer;
use crate::lex::token::Token;
use crate::parse::ast::*;
use crate::__tracing::warn;

use im::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

/// 模块导出——加载的文件模块的成员。
#[derive(Debug, Clone, Default)]
pub(crate) struct ModuleExports {
    vars: HashMap<String, Value>,
    mixins: HashMap<String, MixinDef>,
    functions: HashMap<String, FunctionDef>,
    css: Vec<CssNode>,
    /// 模块加载过程中发现的已加载路径（用于缓存传播）。
    loaded_modules: Rc<std::collections::HashSet<PathBuf>>,
    /// 模块中收集的 @extend 关系——需要传播到顶层 CSS。
    extends: Rc<Vec<(String, String)>>,
}

/// 不可变求值环境。
#[derive(Debug, Clone, Default)]
pub struct Env {
    /// 变量绑定（扁平，用作用域前缀模拟）。
    vars: HashMap<String, Value>,
    /// !global 变量写入——规则体内 !global 赋值需要传播到外层。
    global_writes: HashMap<String, Value>,
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
    /// 加载路径——`@use`/`@import` 无法从当前文件解析时回退搜索。
    load_paths: Vec<PathBuf>,
    /// plain CSS 模式——`.css` 文件加载时设为 true，不展开选择器。
    plain_css: bool,
    /// 已加载模块的路径集合——同一模块的 CSS 只输出一次。
    loaded_modules: Rc<std::collections::HashSet<PathBuf>>,
}

/// mixin 定义存储。
#[derive(Debug, Clone)]
pub(crate) struct MixinDef {
    params: Vec<Param>,
    body: Vec<Node>,
    /// mixin 定义时捕获的命名空间（使 mixin 体可访问定义模块的 @use 命名空间）。
    captured_namespaces: HashMap<String, Rc<ModuleExports>>,
}

/// 函数定义存储。
#[derive(Debug, Clone)]
pub(crate) struct FunctionDef {
    params: Vec<Param>,
    body: Vec<Node>,
    /// 函数定义时捕获的命名空间（使函数体可访问定义模块的 @use 命名空间）。
    captured_namespaces: HashMap<String, Rc<ModuleExports>>,
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
        new.vars.insert(name, value);
        new
    }
    /// 按名查找变量引用。
    pub fn lookup(&self, name: &str) -> Option<&Value> {
        self.vars.get(name)
    }
    /// 判断变量是否已定义。
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
        let env = Env::default().with_base_path(base_path);
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
        match node {
            Node::Rule { selector, body } => Self::eval_rule(selector, body, env),
            Node::Decl {
                property,
                value,
                important,
            } => {
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
            } => {
                // 内建模块 sass:math/string/list/map/color/meta/selector
                if url.starts_with("sass:") {
                    return Ok((vec![], env.add_module(url.clone())));
                }
                // 文件模块——解析路径并加载
                let base = env.base_path.as_ref();
                let load_paths = env.get_load_paths();
                if let Some(path) = Self::resolve_file(base, url, load_paths) {
                    // 模块缓存：同一路径只加载一次，CSS 只输出一次
                    let already_loaded = env.loaded_modules.contains(&path);
                    // 已加载过的模块直接返回空 exports（不重新加载）
                    let exports = if already_loaded {
                        ModuleExports::default()
                    } else {
                        // 将 ConfigVar 转换为 (String, Value) 列表
                        let config_pairs: Vec<(String, Value)> = config
                            .iter()
                            .map(|c| {
                                let val = Self::eval_value(&c.value, env)
                                    .unwrap_or(Value::Null);
                                (c.name.clone(), val)
                            })
                            .collect();
                        Self::load_module(&path, &config_pairs, env)?
                    };
                    // 更新 loaded_modules 缓存：合并子模块发现的路径
                    let mut new_loaded = (*env.loaded_modules).clone();
                    new_loaded.insert(path.clone());
                    new_loaded.extend((*exports.loaded_modules).clone().iter().cloned());
                    // 合并模块的 @extend 关系到当前 env
                    let mut new_extends = (*env.extends).clone();
                    new_extends.extend((*exports.extends).clone().iter().cloned());
                    let env_with_cache = Env {
                        loaded_modules: Rc::new(new_loaded),
                        extends: Rc::new(new_extends),
                        ..env.clone()
                    };
                    // CSS 只在首次加载时输出
                    let css = if already_loaded { vec![] } else { exports.css.clone() };
                    if *star {
                        let mut new_env = env_with_cache;
                        for (k, v) in &exports.vars {
                            new_env = new_env.bind(k.clone(), v.clone());
                        }
                        for (k, v) in &exports.mixins {
                            new_env = new_env.define_mixin(k.clone(), v.clone());
                        }
                        for (k, v) in &exports.functions {
                            new_env = new_env.define_function(k.clone(), v.clone());
                        }
                        return Ok((css, new_env));
                    }
                    let ns = namespace.clone().unwrap_or_else(|| {
                        // 命名空间从 URL 的 basename 计算，去掉所有扩展名和前导下划线
                        let url_stem = std::path::Path::new(url)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or(url);
                        // 去掉所有扩展名（如 other.foo.bar.baz → other）
                        let base = url_stem.split('.').next().unwrap_or(url_stem);
                        base.trim_start_matches('_').to_string()
                    });
                    return Ok((css, env_with_cache.add_namespace(ns, exports)));
                }
                // 找不到文件——静默跳过
                Ok((vec![], env.clone()))
            }
            Node::Forward {
                url,
                show: _,
                hide: _,
                prefix,
                config,
            } => {
                // @forward 'url' —— 转发模块成员到当前作用域
                // as prefix-* 时，成员名加前缀（如 c → d-c）
                let base = env.base_path.as_ref();
                let load_paths = env.get_load_paths();
                if let Some(path) = Self::resolve_file(base, url, load_paths) {
                    // 构建配置变量列表：@forward with() 配置 + caller_env 的变量
                    // 使被加载模块中的 !default 变量能看到当前作用域已定义的值
                    let mut inherited_vars: Vec<(String, Value)> = env
                        .vars
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();
                    // with() 配置覆盖 caller_env 的同名变量
                    for cfg in config {
                        let val = Self::eval_value(&cfg.value, env)?;
                        // with() 配置：!default 时仅未定义才赋值，否则覆盖
                        if cfg.is_default {
                            if !inherited_vars.iter().any(|(n, _)| n.as_str() == cfg.name.as_str()) {
                                inherited_vars.push((cfg.name.clone(), val));
                            }
                        } else if let Some(idx) = inherited_vars.iter().position(|(n, _)| n.as_str() == cfg.name.as_str()) {
                            inherited_vars[idx].1 = val;
                        } else {
                            inherited_vars.push((cfg.name.clone(), val));
                        }
                    }
                    // 模块缓存：同一路径只输出一次 CSS
                    let already_loaded = env.loaded_modules.contains(&path);
                    // 已加载过的模块直接返回空 exports（不重新加载）
                    let exports = if already_loaded {
                        ModuleExports::default()
                    } else {
                        Self::load_module(&path, &inherited_vars, env)?
                    };
                    let css = if already_loaded { vec![] } else { exports.css.clone() };
                    // 更新 loaded_modules 缓存：合并子模块发现的路径
                    let mut new_loaded = (*env.loaded_modules).clone();
                    new_loaded.insert(path.clone());
                    new_loaded.extend((*exports.loaded_modules).clone().iter().cloned());
                    // 合并模块的 @extend 关系到当前 env
                    let mut new_extends = (*env.extends).clone();
                    new_extends.extend((*exports.extends).clone().iter().cloned());
                    let mut new_env = Env {
                        loaded_modules: Rc::new(new_loaded),
                        extends: Rc::new(new_extends),
                        ..env.clone()
                    };
                    if let Some(prefix) = prefix {
                        // 带前缀重映射：c → prefix-c
                        for (k, v) in &exports.vars {
                            new_env = new_env.bind(format!("{prefix}{k}"), v.clone());
                        }
                        for (k, v) in &exports.mixins {
                            new_env = new_env.define_mixin(format!("{prefix}{k}"), v.clone());
                        }
                        for (k, v) in &exports.functions {
                            new_env = new_env.define_function(format!("{prefix}{k}"), v.clone());
                        }
                    } else {
                        // 无前缀：原样绑定
                        for (k, v) in &exports.vars {
                            new_env = new_env.bind(k.clone(), v.clone());
                        }
                        for (k, v) in &exports.mixins {
                            new_env = new_env.define_mixin(k.clone(), v.clone());
                        }
                        for (k, v) in &exports.functions {
                            new_env = new_env.define_function(k.clone(), v.clone());
                        }
                    }
                    return Ok((css, new_env));
                }
                Ok((vec![], env.clone()))
            }
            Node::Import { url, modifier } => {
                // @import 'url' —— 旧版内联：加载文件内容注入当前作用域
                if url.starts_with("sass:") {
                    return Ok((vec![], env.add_module(url.clone())));
                }
                // CSS @import 透传：以 .css 结尾或 url() 包裹，或带修饰符，或多值 CSS @import
                let is_css_import = url.ends_with(".css")
                    || url.starts_with("http://")
                    || url.starts_with("https://")
                    || url.starts_with("url(")
                    || !modifier.is_empty()
                    || url.split("\", \"").any(|u| u.trim_matches('"').ends_with(".css"));
                if is_css_import {
                    // 多值 @import "a", "b" → 输出多行
                    let urls: Vec<&str> = url.split("\", \"").collect();
                    let mut nodes = Vec::new();
                    for u in &urls {
                        let u = u.trim_matches('"');
                        let params = if modifier.is_empty() {
                            format!("\"{u}\"")
                        } else {
                            format!("\"{u}\" {modifier}")
                        };
                        nodes.push(CssNode::AtRule {
                            name: "import".to_string(),
                            params: Some(params),
                            children: vec![],
                            has_body: false,
                        });
                    }
                    return Ok((nodes, env.clone()));
                }
                let base = env.base_path.as_ref();
                let load_paths = env.get_load_paths();
                if let Some(path) = Self::resolve_file(base, url, load_paths) {
                    // @import 内联：继承当前环境（变量/mixin/函数），使被导入文件能看到之前定义的成员
                    return Self::load_import(&path, env);
                }
                // 文件未找到：如果不是 CSS URL（.css / http / url()），报错
                if !url.ends_with(".css") && !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("url(") && modifier.is_empty() {
                    return Err(SassError::Module(format!("Can't find stylesheet to import: {url}")));
                }
                // CSS @import 透传（带修饰符或 CSS URL）
                let params = if modifier.is_empty() {
                    format!("\"{url}\"")
                } else {
                    format!("\"{url}\" {modifier}")
                };
                Ok((
                    vec![CssNode::AtRule {
                        name: "import".to_string(),
                        params: Some(params),
                        children: vec![],
                        has_body: false,
                    }],
                    env.clone(),
                ))
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

mod at_params;
mod builtin;
mod color;
mod control_flow;
mod extend;
mod mixin;
mod module;
mod rule;
mod value;
