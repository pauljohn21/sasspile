use super::*;
use crate::error::{Result, SassError};
use crate::lex::Lexer;
use crate::lex::token::Token;
use std::path::Path;

use super::module_helpers::{BindMode, FilterConfig, bind_exports, merge_module_cache};

impl Evaluator {
    /// 递归收集 AST 中所有 !global 变量名
    fn collect_global_vars(nodes: &[crate::parse::ast::Node]) -> Vec<String> {
        use crate::parse::ast::Node;
        nodes
            .iter()
            .flat_map(|node| {
                let vars: Vec<String> = match node {
                    Node::Variable { name, flags, .. } => {
                        match (flags.global, !name.contains('.')) {
                            (true, true) => vec![name.clone()],
                            _ => Vec::new(),
                        }
                    }
                    Node::If {
                        branches,
                        else_body,
                    } => {
                        let branch_vars: Vec<String> = branches
                            .iter()
                            .flat_map(|(_, branch_body)| Self::collect_global_vars(branch_body))
                            .collect();
                        let else_vars: Vec<String> = else_body
                            .as_ref()
                            .map(|e| Self::collect_global_vars(e))
                            .unwrap_or_default();
                        branch_vars.into_iter().chain(else_vars).collect()
                    }
                    Node::For { body, .. }
                    | Node::Each { body, .. }
                    | Node::While { body, .. }
                    | Node::MixinDef { body, .. }
                    | Node::FunctionDef { body, .. }
                    | Node::AtRoot { body, .. } => Self::collect_global_vars(body),
                    Node::Include { content, .. } => content
                        .as_ref()
                        .map(|c| Self::collect_global_vars(c))
                        .unwrap_or_default(),
                    Node::Rule { body, .. } => Self::collect_global_vars(body),
                    Node::AtRule { body, .. } => body
                        .as_ref()
                        .map(|b| Self::collect_global_vars(b))
                        .unwrap_or_default(),
                    _ => Vec::new(),
                };
                vars
            })
            .collect()
    }

    pub(crate) fn load_module(
        path: &Path,
        config: &[(String, Value)],
        caller_env: &Env,
        validate_config: bool,
    ) -> Result<ModuleExports> {
        // 防止循环导入导致栈溢出
        match caller_env.depth > 50 {
            true => return Ok(ModuleExports::default()),
            false => {}
        }
        // 模块缓存：如果路径已加载过，从缓存返回 exports（CSS 为空，不重复输出）。
        match caller_env.loaded_modules.contains(path) {
            true => match caller_env.get_module_cache().get(path) {
                Some(cached) => {
                    let cached_exports = ModuleExports {
                        css: vec![],
                        ..cached.clone()
                    };
                    return Ok(cached_exports);
                }
                None => return Ok(ModuleExports::default()),
            },
            false => {}
        }
        let source = std::fs::read_to_string(path)
            .map_err(|e| SassError::Module(format!("Cannot read {}: {e}", path.display())))?;

        let is_plain_css = path.extension().and_then(|e| e.to_str()) == Some("css");

        let tokens: Vec<Token> = Lexer::new(&source)
            .filter(|t| !matches!(t.as_ref(), Ok(Token::Eof)))
            .collect::<Result<Vec<_>>>()?;
        let ast = crate::parse::Parser::parse(&tokens)?;
        let mut env = Env::default()
            .with_base_path(path.to_path_buf())
            .with_load_paths(caller_env.get_load_paths().to_vec())
            .with_module_cache((*caller_env.module_cache).clone())
            .with_depth(caller_env.get_depth() + 1)
            .with_plain_css(is_plain_css);
        let mut loaded = (*caller_env.loaded_modules).clone();
        loaded.insert(path.to_path_buf());
        env = env.with_loaded_modules(loaded);
        // 注入 with() 配置变量到 pending_config
        let mut null_configs: Vec<String> = Vec::new();
        for (name, value) in config {
            let val = Self::eval_value(value, caller_env)?;
            match matches!(val, Value::Null) {
                true => null_configs.push(name.replace('-', "_")),
                false => {
                    let key = name.replace('-', "_");
                    crate::__tracing::debug!(name = %key, "load_module: inject pending_config");
                    env = env.add_pending_config(key, val);
                }
            }
        }
        // 预扫描 AST 中所有 !global 变量声明，预先初始化为 null
        // SCSS 规范要求模块始终暴露这些变量，即使所在代码路径未执行
        for global_var in Self::collect_global_vars(&ast.nodes) {
            match !env.has_var(&global_var) {
                true => env = env.bind(global_var, crate::parse::ast::Value::Null),
                false => {}
            }
        }
        // 验证配置变量在上游模块中必须带 !default 声明
        // 验证在 eval_nodes 之后执行（运行时消费跟踪）
        let (module_css, mut final_env) = Self::eval_nodes(&ast.nodes, env)?;
        // 验证：config 中未被消费的 key 说明对应变量未声明 !default
        // 仅当 validate_config=true（@use with 调用）时验证
        match (validate_config, !config.is_empty()) {
            (true, true) => {
                let consumed = final_env.get_consumed_config();
                crate::__tracing::debug!(
                    consumed = ?consumed,
                    pending = ?final_env.get_pending_config().keys().collect::<Vec<_>>(),
                    "load_module: validation check"
                );
                for (name, _) in config {
                    let normalized = name.replace('-', "_");
                    match null_configs.contains(&normalized) {
                        true => continue,
                        false => match !consumed.contains(&normalized) && !consumed.contains(name) {
                            true => {
                                crate::__tracing::warn!(
                                    name = %name,
                                    normalized = %normalized,
                                    "load_module: config var not consumed — not !default"
                                );
                                return Err(SassError::Eval(
                                    "This variable was not declared with !default in the @used module.".into(),
                                ));
                            }
                            false => {}
                        },
                    }
                }
            }
            _ => {}
        }
        // extends 在顶层 evaluate 中统一应用（带模块路径标记）
        let selectors = Self::collect_all_selectors(
            final_env.get_module_cache(),
            path,
            &module_css,
            &ast,
            final_env.get_load_paths(),
        );
        let css = if is_plain_css {
            vec![crate::css::node::CssNode::AtRoot(module_css, None)]
        } else {
            module_css
        };
        let (lv, lm, lf, fv, fm, ff) = final_env.take_scope_fields();
        let exports = ModuleExports {
            local_vars: lv,
            local_mixins: lm,
            local_functions: lf,
            forwarded_vars: fv,
            forwarded_mixins: fm,
            forwarded_functions: ff,
            css,
            loaded_modules: final_env.get_loaded_modules_rc(),
            extends: final_env.get_extends_rc(),
            module_cache: final_env.get_module_cache_rc(),
            consumed_config: final_env.get_consumed_config().clone(),
            selectors,
            star_imported: final_env.get_star_imported().clone(),
        };
        let exports_cache = exports.module_cache.clone();
        let mut updated_cache = (*exports_cache).clone();
        updated_cache.insert(
            path.to_path_buf(),
            ModuleExports {
                css: vec![],
                module_cache: exports.module_cache.clone(),
                ..exports.clone()
            },
        );
        let exports = ModuleExports {
            module_cache: Rc::new(updated_cache),
            ..exports
        };
        Ok(exports)
    }

    /// 加载 @import 文件——内联模式：继承当前环境的所有成员。
    ///
    /// SCSS @import 语义：被导入文件在当前作用域执行，
    /// 能看到之前定义的所有变量/mixin/函数，且定义的成员在导入后可见。
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(caller_env), fields(path = %path.display(), depth = caller_env.depth)))]
    pub(crate) fn load_import(path: &Path, caller_env: Env) -> Result<(Vec<CssNode>, Env)> {
        // 循环加载检测
        match (caller_env.loaded_modules.contains(path), !caller_env.get_module_cache().contains_key(path)) {
            (true, true) => return Err(SassError::Module(
                "This file is already being loaded.".into(),
            )),
            _ => {}
        }
        match caller_env.depth > 50 {
            true => return Ok((vec![], caller_env)),
            false => {}
        }
        let source = std::fs::read_to_string(path)
            .map_err(|e| SassError::Module(format!("Cannot read {}: {e}", path.display())))?;

        let is_plain_css = path.extension().and_then(|e| e.to_str()) == Some("css");

        let tokens: Vec<Token> = Lexer::new(&source)
            .filter(|t| !matches!(t.as_ref(), Ok(Token::Eof)))
            .collect::<Result<Vec<_>>>()?;
        let ast = crate::parse::Parser::parse(&tokens)?;
        // 继承当前环境的所有成员
        let saved_base_path = caller_env.get_base_path().cloned();
        let saved_depth = caller_env.get_depth();
        let env = caller_env
            .with_base_path(path.to_path_buf())
            .with_depth(saved_depth + 1)
            .with_plain_css(is_plain_css);
        // 预扫描导入文件中的 !global 变量（确保未执行路径的变量也可见）
        let mut env = env;
        for global_var in Self::collect_global_vars(&ast.nodes) {
            match !env.has_var(&global_var) {
                true => env = env.bind(global_var, crate::parse::ast::Value::Null),
                false => {}
            }
        }
        let (css, final_env) = Self::eval_nodes(&ast.nodes, env)?;
        // 恢复调用者的 base_path 和 depth
        let final_env = if let Some(bp) = saved_base_path {
            final_env.with_base_path(bp)
        } else {
            final_env
        }
        .with_depth(saved_depth);
        // @import 内联语义：forwarded 成员合并到 local
        let final_env = final_env.merge_forwarded_to_local();
        // @import 内联语义：移除通过 @use ... as * 引入的传递性成员
        // 这些成员只在被导入文件内部可见，不应传递到导入文件
        let final_env = final_env.remove_star_imported();
        let css = if is_plain_css {
            vec![crate::css::node::CssNode::AtRoot(css, None)]
        } else {
            css
        };
        Ok((css, final_env))
    }

    /// 模块限定函数调用。
    #[cfg_attr(feature = "tracing", tracing::instrument(skip_all, fields(name = name)))]
    pub(crate) fn call_module_function(
        name: &str,
        pos_args: &[Value],
        kw_args: &HashMap<String, Value>,
        env: &Env,
    ) -> Result<Value> {
        // 先检查文件加载的命名空间
        if let Some(dot) = name.find('.') {
            let ns = &name[..dot];
            let func_name = &name[dot + 1..];
            if let Some(module) = env.get_namespace(ns)
                && let Some(func) = module
                    .all_functions()
                    .find(|(k, _)| *k == func_name)
                    .map(|(_, f)| f)
            {
                // 注入模块的 vars 到函数环境，使函数体可访问模块变量
                let func_env = module.all_vars().fold(env.clone(), |acc, (k, v)| {
                    match acc.has_var(k) {
                        true => acc,
                        false => acc.bind(k.clone(), v.clone()),
                    }
                });
                return Self::call_user_function(func, pos_args, kw_args, func_env);
            }
        }
        // 将模块限定名映射到内建函数
        let builtin_name = super::builtin::dispatch::module_builtin_name(name);
        Self::call_builtin(builtin_name, pos_args, kw_args, env)
    }
}

impl Evaluator {
    /// @use 指令处理。
    pub(crate) fn eval_use(
        url: &str,
        namespace: &Option<String>,
        star: bool,
        config: &[crate::parse::ast::ConfigVar],
        env: Env,
    ) -> Result<(Vec<CssNode>, Env)> {
        // @use 只能在顶层使用——在 style rule 或 mixin 内报错
        match env.get_selector().is_some() || env.get_content().is_some() {
            true => return Err(SassError::Eval("This at-rule is not allowed here.".into())),
            false => {}
        }
        match url.is_empty() {
            true => return Err(SassError::Eval(
                "The default namespace \"\" is not a valid Sass identifier.".into(),
            )),
            false => {}
        }
        match (!url.starts_with("sass:"), url.contains(':')) {
            (true, true) => return Err(SassError::Module(format!(
                "Can't find stylesheet to import: {url}"
            ))),
            _ => {}
        }
        match (!url.starts_with("sass:"), namespace.is_none(), !star) {
            (true, true, true) => {
            let stem = std::path::Path::new(url)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(url);
            let base = stem.split('.').next().unwrap_or(stem);
            let ns = base.trim_start_matches('_');
            match (!ns.is_empty(), !ns.chars().next().is_some_and(|c| c.is_alphabetic() || c == '_')) {
                (true, true) => return Err(SassError::Eval(format!(
                    "The default namespace \"{ns}\" is not a valid Sass identifier."
                ))),
                _ => {}
            }
            }
            _ => {}
        }
        // 内建模块 sass:math/string/list/map/color/meta/selector
        match url.starts_with("sass:") {
            true => {
                match !config.is_empty() {
                    true => return Err(SassError::Eval(
                        "Built-in modules can't be configured.".into(),
                    )),
                    false => {}
                }
                let ns = url.strip_prefix("sass:").unwrap_or(url);
                match env.get_namespace(ns).is_some() {
                    true => return Err(SassError::Eval(format!(
                        "There's already a module with namespace \"{ns}\"."
                    ))),
                    false => {}
                }
                return Ok((vec![], env.add_module(url.to_string())));
            }
            false => {}
        }
        let base = env.get_base_path().cloned();
        let load_paths = env.get_load_paths().to_vec();
        // @use 文件歧义检测（与 @import 相同的四种冲突场景）
        Self::check_resolve_ambiguity(base.as_ref(), url, &load_paths)?;
        match Self::resolve_file(base.as_ref(), url, &load_paths) {
            Some(path) => {
            let already_loaded = env.get_loaded_modules().contains(&path);
            match (already_loaded, !env.get_module_cache().contains_key(&path)) {
                (true, true) => return Err(SassError::Module(
                    "Module loop: this module is already being loaded.".into(),
                )),
                _ => {}
            }
            match (already_loaded, !config.is_empty()) {
                (true, true) => return Err(SassError::Eval(
                    "This module was already loaded, so it can't be configured using \"with\"."
                        .into(),
                )),
                _ => {}
            }
            let exports = if already_loaded {
                env.get_module_cache()
                    .get(&path)
                    .cloned()
                    .unwrap_or_default()
            } else {
                // 检查重复配置变量
                let mut seen = std::collections::HashSet::new();
                for c in config {
                    let normalized = c.name.replace('-', "_");
                    match !seen.insert(normalized) {
                        true => return Err(SassError::Eval(
                            "The same variable may only be configured once.".into(),
                        )),
                        false => {}
                    }
                }
                let config_pairs: Vec<(String, Value)> = config
                    .iter()
                    .map(|c| {
                        let val = Self::eval_value(&c.value, &env)?;
                        Ok::<(String, Value), SassError>((c.name.clone(), val))
                    })
                    .collect::<Result<Vec<_>>>()?;
                Self::load_module(&path, &config_pairs, &env, true)?
            };
            let env_with_cache = merge_module_cache(env, &path, &exports);
            let mut exports = exports;
            let css = match already_loaded {
                true => Vec::new(),
                false => std::mem::take(&mut exports.css),
            };
            match star {
                true => {
                    let new_env = bind_exports(
                        env_with_cache,
                        &exports,
                        None,
                        BindMode::Use,
                        &path,
                        &FilterConfig::default(),
                    )?;
                    return Ok((css, new_env));
                }
                false => {}
            }
            let ns = namespace.clone().unwrap_or_else(|| {
                let url_stem = std::path::Path::new(url)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(url);
                let base = url_stem.split('.').next().unwrap_or(url_stem);
                base.trim_start_matches('_').to_string()
            });
            // 检查命名空间冲突
            // 如果模块已加载（already_loaded），命名空间可能来自 @import 继承的 env
            // 此时不应报冲突，而是从缓存返回
            match (!already_loaded, env_with_cache.get_namespace(&ns).is_some()) {
                (true, true) => return Err(SassError::Eval(format!(
                    "There's already a module with namespace \"{ns}\"."
                ))),
                _ => {}
            }
            return Ok((css, env_with_cache.add_namespace(ns, exports)));
            }
            _ => {
                // @use 找不到文件时必须报错（不像 @import 可以输出 CSS @import 语句）
                Err(SassError::Module(format!(
                    "Can't find stylesheet to import: {url}"
                )))
            }
        }
    }
}
