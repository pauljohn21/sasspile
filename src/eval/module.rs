use super::*;
use crate::error::{Result, SassError};
use std::path::Path;

use super::module_helpers::{BindMode, FilterConfig, bind_exports, merge_module_cache};

impl Evaluator {
    /// 递归收集 AST 中所有 !global 变量名
    fn collect_global_vars(nodes: &[crate::parse::ast::Node]) -> Vec<String> {
        use crate::parse::ast::Node;
        let mut vars = Vec::new();
        for node in nodes {
            match node {
                Node::Variable {
                    name,
                    flags,
                    ..
                } => {
                    if flags.global && !name.contains('.') {
                        vars.push(name.clone());
                    }
                }
                Node::If {
                    branches,
                    else_body,
                } => {
                    for (_, branch_body) in branches {
                        vars.extend(Self::collect_global_vars(branch_body));
                    }
                    if let Some(else_nodes) = else_body {
                        vars.extend(Self::collect_global_vars(else_nodes));
                    }
                }
                Node::For { body, .. }
                | Node::Each { body, .. }
                | Node::While { body, .. }
                | Node::MixinDef { body, .. }
                | Node::FunctionDef { body, .. }
                | Node::AtRoot { body, .. } => {
                    vars.extend(Self::collect_global_vars(body));
                }
                Node::Include {
                    args,
                    content,
                    ..
                } => {
                    let _ = args;
                    if let Some(content_nodes) = content {
                        vars.extend(Self::collect_global_vars(content_nodes));
                    }
                }
                Node::Rule { body, .. } => {
                    vars.extend(Self::collect_global_vars(body));
                }
                Node::AtRule { body, .. } => {
                    if let Some(b) = body {
                        vars.extend(Self::collect_global_vars(b));
                    }
                }
                _ => {}
            }
        }
        vars
    }

    pub(crate) fn load_module(
        path: &Path,
        config: &[(String, Value)],
        caller_env: &Env,
        validate_config: bool,
    ) -> Result<ModuleExports> {
        // 防止循环导入导致栈溢出
        if caller_env.depth > 50 {
            return Ok(ModuleExports::default());
        }
        // 模块缓存：如果路径已加载过，从缓存返回 exports（CSS 为空，不重复输出）。
        if caller_env.loaded_modules.contains(path) {
            if let Some(cached) = caller_env.get_module_cache().get(path) {
                let cached_exports = ModuleExports {
                    css: vec![],
                    ..cached.clone()
                };
                return Ok(cached_exports);
            }
            return Ok(ModuleExports::default());
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
            if matches!(val, Value::Null) {
                // null 值配置不注入，但记录用于验证时跳过
                null_configs.push(name.replace('-', "_"));
            } else {
                let key = name.replace('-', "_");
                crate::__tracing::debug!(name = %key, "load_module: inject pending_config");
                env = env.add_pending_config(key, val);
            }
        }
        // 预扫描 AST 中所有 !global 变量声明，预先初始化为 null
        // SCSS 规范要求模块始终暴露这些变量，即使所在代码路径未执行
        for global_var in Self::collect_global_vars(&ast.nodes) {
            if !env.has_var(&global_var) {
                env = env.bind(global_var, crate::parse::ast::Value::Null);
            }
        }
        // 验证配置变量在上游模块中必须带 !default 声明
        // 验证在 eval_nodes 之后执行（运行时消费跟踪）
        let (module_css, final_env) = Self::eval_nodes(&ast.nodes, env)?;
        // 验证：config 中未被消费的 key 说明对应变量未声明 !default
        // 仅当 validate_config=true（@use with 调用）时验证
        if validate_config && !config.is_empty() {
            let consumed = final_env.get_consumed_config();
            crate::__tracing::debug!(
                consumed = ?consumed,
                pending = ?final_env.get_pending_config().keys().collect::<Vec<_>>(),
                "load_module: validation check"
            );
            for (name, _) in config {
                let normalized = name.replace('-', "_");
                // null 值配置跳过验证（不覆盖 !default）
                if null_configs.contains(&normalized) {
                    continue;
                }
                if !consumed.contains(&normalized) && !consumed.contains(name) {
                    crate::__tracing::warn!(
                        name = %name,
                        normalized = %normalized,
                        "load_module: config var not consumed — not !default"
                    );
                    return Err(SassError::Eval(
                        "This variable was not declared with !default in the @used module.".into(),
                    ));
                }
            }
        }
        // extends 在顶层 evaluate 中统一应用（带模块路径标记）
        let selectors = Self::collect_all_selectors(
            &final_env.get_module_cache(),
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
        let exports = ModuleExports {
            local_vars: final_env.get_local_vars().clone(),
            local_mixins: final_env.get_local_mixins().clone(),
            local_functions: final_env.get_local_functions().clone(),
            forwarded_vars: final_env.get_forwarded_vars().clone(),
            forwarded_mixins: final_env.get_forwarded_mixins().clone(),
            forwarded_functions: final_env.get_forwarded_functions().clone(),
            css,
            loaded_modules: final_env.get_loaded_modules_rc(),
            extends: final_env.get_extends_rc(),
            module_cache: final_env.get_module_cache_rc(),
            consumed_config: final_env.get_consumed_config().clone(),
            selectors,
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
        if caller_env.loaded_modules.contains(path)
            && !caller_env.get_module_cache().contains_key(path)
        {
            return Err(SassError::Module(
                "This file is already being loaded.".into(),
            ));
        }
        if caller_env.depth > 50 {
            return Ok((vec![], caller_env));
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
            if !env.has_var(&global_var) {
                env = env.bind(global_var, crate::parse::ast::Value::Null);
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
                let mut func_env = env.clone();
                for (k, v) in module.all_vars() {
                    if !func_env.local_vars.contains_key(k) {
                        func_env = func_env.bind(k.clone(), v.clone());
                    }
                }
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
        // 内建模块 sass:math/string/list/map/color/meta/selector
        if url.starts_with("sass:") {
            if !config.is_empty() {
                return Err(SassError::Eval(
                    "Built-in modules can't be configured.".into(),
                ));
            }
            // 检查内建模块命名空间冲突
            let ns = url.strip_prefix("sass:").unwrap_or(url);
            if env.get_namespace(ns).is_some() {
                return Err(SassError::Eval(format!(
                    "There's already a module with namespace \"{ns}\"."
                )));
            }
            return Ok((vec![], env.add_module(url.to_string())));
        }
        let base = env.get_base_path().cloned();
        let load_paths = env.get_load_paths().to_vec();
        if let Some(path) = Self::resolve_file(base.as_ref(), url, &load_paths) {
            let already_loaded = env.get_loaded_modules().contains(&path);
            if already_loaded && !env.get_module_cache().contains_key(&path) {
                return Err(SassError::Module(
                    "Module loop: this module is already being loaded.".into(),
                ));
            }
            if already_loaded && !config.is_empty() {
                return Err(SassError::Eval(
                    "This module was already loaded, so it can't be configured using \"with\"."
                        .into(),
                ));
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
                    if !seen.insert(normalized) {
                        return Err(SassError::Eval(
                            "The same variable may only be configured once.".into(),
                        ));
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
            let css = if already_loaded {
                vec![]
            } else {
                exports.css.clone()
            };
            if star {
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
            if !already_loaded && env_with_cache.get_namespace(&ns).is_some() {
                return Err(SassError::Eval(format!(
                    "There's already a module with namespace \"{ns}\"."
                )));
            }
            return Ok((css, env_with_cache.add_namespace(ns, exports)));
        }
        Ok((vec![], env))
    }

    /// @forward 指令处理。
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(env), fields(url = url, has_prefix = prefix.is_some())))]
    pub(crate) fn eval_forward(
        url: &str,
        prefix: &Option<String>,
        config: &[crate::parse::ast::ConfigVar],
        env: Env,
        show: &[String],
        hide: &[String],
    ) -> Result<(Vec<CssNode>, Env)> {
        // 内建模块（sass:xxx）不能用 with 配置
        if url.starts_with("sass:") && !config.is_empty() {
            return Err(SassError::Eval(
                "Built-in modules can't be configured.".into(),
            ));
        }
        // @forward 内建模块（sass:xxx）——注册内建模块命名空间
        if url.starts_with("sass:") {
            let exports = crate::eval::module_helpers::builtin_module_exports(url)
                .unwrap_or_default();
            let filter = FilterConfig {
                show: show.to_vec(),
                hide: hide.to_vec(),
            };
            let new_env = bind_exports(
                env,
                &exports,
                prefix.as_deref(),
                BindMode::Forward,
                &std::path::PathBuf::from(url),
                &filter,
            )?;
            // 标记内建模块已注册（使命名空间下的函数调用能正确分派）
            let new_env = new_env.add_module(url.to_string());
            return Ok((vec![], new_env));
        }
        let base = env.get_base_path().cloned();
        let load_paths = env.get_load_paths().to_vec();
        if let Some(path) = Self::resolve_file(base.as_ref(), url, &load_paths) {
            let config_pairs: Vec<(String, Value)> = {
                let prefix_str = prefix.as_deref();
                let strip_prefix = |k: &str| -> String {
                    if let Some(p) = prefix_str {
                        let pfx = p.replace('-', "_");
                        let k_norm = k.replace('-', "_");
                        if k_norm.starts_with(&pfx) {
                            return k_norm[pfx.len()..].to_string();
                        }
                    }
                    k.replace('-', "_")
                };
                // show/hide 过滤：检查变量名是否通过过滤
                // show 格式: ["$a", "mixin_name", ...]；hide 同理
                let passes_filter = |name: &str| -> bool {
                    let var_marker = format!("${name}");
                    if !show.is_empty() {
                        return show.iter().any(|s| s == &var_marker || s == name);
                    }
                    if !hide.is_empty() {
                        return !hide.iter().any(|s| s == &var_marker || s == name);
                    }
                    true
                };
                // 裸 forward：传递所有 pending_config（剥离前缀），受 show/hide 过滤
                // @forward with：传递 with 中声明的变量 + pending_config 中未被覆盖的变量，受 show/hide 过滤
                if config.is_empty() {
                    env.get_pending_config()
                        .iter()
                        .filter(|(k, _)| passes_filter(k))
                        .map(|(k, v)| (strip_prefix(k), v.clone()))
                        .collect()
                } else {
                    let mut configured_names: std::collections::HashSet<String> =
                        std::collections::HashSet::new();
                    let from_config: Vec<(String, Value)> =
                        config.iter().try_fold(Vec::new(), |mut acc, cfg| {
                            let name = strip_prefix(&cfg.name);
                            configured_names.insert(name.clone());
                            let val = Evaluator::eval_value(&cfg.value, &env)?;
                            let pending_val = env.get_pending_config().get(&name).or_else(|| {
                                env.get_pending_config().get(&cfg.name.replace('-', "_"))
                            });
                            // 选择值：!default 时 pending_config 优先，否则 with 的值优先
                            let chosen = if cfg.is_default {
                                pending_val
                                    .filter(|v| !matches!(v, Value::Null))
                                    .cloned()
                                    .or(if matches!(val, Value::Null) {
                                        None
                                    } else {
                                        Some(val)
                                    })
                            } else if matches!(val, Value::Null) {
                                pending_val.filter(|v| !matches!(v, Value::Null)).cloned()
                            } else {
                                Some(val)
                            };
                            if let Some(v) = chosen {
                                acc.push((name, v));
                            }
                            Ok::<_, SassError>(acc)
                        })?;
                    // 追加 pending_config 中未被 with 覆盖的变量（透传），受 show/hide 过滤
                    let mut result = from_config;
                    for (k, v) in env.get_pending_config() {
                        let stripped = strip_prefix(k);
                        if !configured_names.contains(&stripped)
                            && !matches!(v, Value::Null)
                            && passes_filter(k)
                        {
                            result.push((stripped, v.clone()));
                        }
                    }
                    result
                }
            };
            crate::__tracing::debug!(
                config_pairs = ?config_pairs.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>(),
                pending = ?env.get_pending_config().keys().collect::<Vec<_>>(),
                "eval_forward: built config_pairs"
            );
            let already_loaded = env.get_loaded_modules().contains(&path);
            if already_loaded && !env.get_module_cache().contains_key(&path) {
                return Err(SassError::Module(
                    "Module loop: this module is already being loaded.".into(),
                ));
            }
            if already_loaded && !config.is_empty() {
                return Err(SassError::Eval(
                    "This module was already loaded, so it can't be configured using \"with\"."
                        .into(),
                ));
            }
            let exports = if already_loaded {
                env.get_module_cache()
                    .get(&path)
                    .cloned()
                    .unwrap_or_default()
            } else {
                Self::load_module(&path, &config_pairs, &env, false)?
            };
            let css = if already_loaded {
                vec![]
            } else {
                exports.css.clone()
            };
            let env_with_cache = merge_module_cache(env, &path, &exports);
            let filter = FilterConfig {
                show: show.to_vec(),
                hide: hide.to_vec(),
            };
            let new_env = bind_exports(
                env_with_cache,
                &exports,
                prefix.as_deref(),
                BindMode::Forward,
                &path,
                &filter,
            )?;
            // 将子模块的 consumed_config 回传到当前 env
            // 如果有 as 前缀，子模块消费的 key 需要加上前缀后回传
            // （外层 load_module 用带前缀的 key 做验证）
            // 关键：如果 @forward ... with 显式传递了配置变量，这些变量不应算作父模块的 !default 消费
            // （它们是被 forward 的 with 消费，而非父模块自身声明 !default）
            let prefix_norm = prefix.as_deref().map(|p| p.replace('-', "_"));
            let add_prefix = |k: &str| -> String {
                if let Some(ref pfx) = prefix_norm {
                    format!("{pfx}{k}")
                } else {
                    k.to_string()
                }
            };
            // @forward with 显式传递且不带 !default 的变量名集合
            // 这些变量被 forward 的 with 消费，不算父模块的 !default 声明
            // 带 !default 的变量是透传 pending_config，仍然算 consumed
            let forward_with_names: std::collections::HashSet<String> = config
                .iter()
                .filter(|c| !c.is_default)
                .map(|c| {
                    if let Some(p) = prefix.as_deref() {
                        let pfx = p.replace('-', "_");
                        let k_norm = c.name.replace('-', "_");
                        if k_norm.starts_with(&pfx) {
                            k_norm[pfx.len()..].to_string()
                        } else {
                            k_norm
                        }
                    } else {
                        c.name.replace('-', "_")
                    }
                })
                .collect();
            let merged_consumed: std::collections::HashSet<String> = new_env
                .get_consumed_config()
                .iter()
                .cloned()
                .chain(
                    exports
                        .consumed_config
                        .iter()
                        .filter(|k| !forward_with_names.contains(k.as_str()))
                        .map(|k| add_prefix(k)),
                )
                .collect();
            crate::__tracing::debug!(
                child_consumed = ?exports.consumed_config,
                merged = ?merged_consumed,
                prefix = ?prefix,
                "eval_forward: consumed_config merge"
            );
            let new_env = new_env.with_consumed_config(merged_consumed);
            return Ok((css, new_env));
        }
        Err(SassError::Eval("Can't find stylesheet to import.".into()))
    }
}
