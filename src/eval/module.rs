use super::*;
use crate::error::{Result, SassError};
use std::path::Path;

use super::module_helpers::{bind_exports, merge_module_cache, BindMode, FilterConfig};

impl Evaluator {
    /// 加载文件模块——读取、词法分析、语法分析、求值，返回导出。
    pub(crate) fn load_module(
        path: &Path,
        config: &[(String, Value)],
        caller_env: &Env,
    ) -> Result<ModuleExports> {
        let span = crate::__tracing::info_span!("load_module", path = %path.display(), depth = caller_env.depth, n_config = config.len());
        let _enter = span.enter();
        // 防止循环导入导致栈溢出
        if caller_env.depth > 50 {
            return Ok(ModuleExports::default());
        }
        // 模块缓存：如果路径已加载过，从缓存返回 exports（CSS 为空，不重复输出）。
        if caller_env.loaded_modules.contains(path) {
            let span = crate::__tracing::debug_span!("load_module_cached", path = %path.display());
            let _enter = span.enter();
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
            .with_module_cache((*caller_env.module_cache).clone());
        env.depth = caller_env.depth + 1;
        env.plain_css = is_plain_css;
        let mut loaded = (*caller_env.loaded_modules).clone();
        loaded.insert(path.to_path_buf());
        env.loaded_modules = Rc::new(loaded);
        // 注入 with() 配置变量到 pending_config
        for (name, value) in config {
            let val = Self::eval_value(value, caller_env)?;
            if !matches!(val, Value::Null) {
                let key = name.replace('-', "_");
                env.pending_config.insert(key, val);
            }
        }
        // 验证配置变量在上游模块中必须带 !default 声明
        if !env.pending_config.is_empty() {
            let default_vars = crate::eval::module_validation::collect_default_vars(&ast.nodes);
            for (name, _) in env.pending_config.iter() {
                if !default_vars.iter().any(|d| d.replace('-', "_") == *name) {
                    return Err(SassError::Eval(
                        "This variable was not declared with !default in the @used module.".into(),
                    ));
                }
            }
        }
        let (module_css, final_env) = Self::eval_nodes(&ast.nodes, env)?;
        let css = if is_plain_css {
            vec![crate::css::node::CssNode::AtRoot(module_css)]
        } else {
            module_css
        };
        let exports = ModuleExports {
            local_vars: final_env.local_vars.clone(),
            local_mixins: final_env.local_mixins.clone(),
            local_functions: final_env.local_functions.clone(),
            forwarded_vars: final_env.forwarded_vars.clone(),
            forwarded_mixins: final_env.forwarded_mixins.clone(),
            forwarded_functions: final_env.forwarded_functions.clone(),
            css,
            loaded_modules: final_env.loaded_modules.clone(),
            extends: final_env.extends.clone(),
            module_cache: final_env.module_cache.clone(),
        };
        let mut updated_cache = (*exports.module_cache).clone();
        updated_cache.insert(path.to_path_buf(), ModuleExports { css: vec![], module_cache: exports.module_cache.clone(), ..exports.clone() });
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
    pub(crate) fn load_import(path: &Path, caller_env: Env) -> Result<(Vec<CssNode>, Env)> {
        let span =
            crate::__tracing::info_span!("load_import", path = %path.display(), depth = caller_env.depth);
        let _enter = span.enter();
        // 循环加载检测
        if caller_env.loaded_modules.contains(path) && !caller_env.get_module_cache().contains_key(path) {
            return Err(SassError::Module("This file is already being loaded.".into()));
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
        let mut env = caller_env;
        let saved_base_path = env.base_path.clone();
        let saved_depth = env.depth;
        env.base_path = Some(path.to_path_buf());
        env.depth += 1;
        env.plain_css = is_plain_css;
        let (css, mut final_env) = Self::eval_nodes(&ast.nodes, env)?;
        // 恢复调用者的 base_path 和 depth
        final_env.base_path = saved_base_path;
        final_env.depth = saved_depth;
        // @import 内联语义：forwarded 成员合并到 local
        for (k, v) in final_env.forwarded_vars.iter().map(|(k, v)| (k.clone(), v.clone())) {
            final_env.local_vars.entry(k).or_insert(v);
        }
        for (k, v) in final_env.forwarded_mixins.iter().map(|(k, v)| (k.clone(), v.clone())) {
            final_env.local_mixins.entry(k).or_insert(v);
        }
        for (k, v) in final_env.forwarded_functions.iter().map(|(k, v)| (k.clone(), v.clone())) {
            final_env.local_functions.entry(k).or_insert(v);
        }
        final_env.forwarded_vars.clear();
        final_env.forwarded_mixins.clear();
        final_env.forwarded_functions.clear();
        let css = if is_plain_css {
            vec![crate::css::node::CssNode::AtRoot(css)]
        } else {
            css
        };
        Ok((css, final_env))
    }

    /// 模块限定函数调用。
    pub(crate) fn call_module_function(name: &str, pos_args: &[Value], kw_args: &HashMap<String, Value>, env: &Env) -> Result<Value> {
        let span = crate::__tracing::info_span!("call_module_function", name = name);
        let _enter = span.enter();
        // 先检查文件加载的命名空间
        if let Some(dot) = name.find('.') {
            let ns = &name[..dot];
            let func_name = &name[dot + 1..];
            if let Some(module) = env.get_namespace(ns)
                && let Some(func) = module.all_functions().find(|(k, _)| *k == func_name).map(|(_, f)| f) {
                    // 注入模块的 vars 到函数环境，使函数体可访问模块变量
                    let mut func_env = env.clone();
                    for (k, v) in module.all_vars() {
                        if !func_env.local_vars.contains_key(k) {
                            func_env = func_env.bind(k.clone(), v.clone());
                        }
                    }
                    return Self::call_user_function(func, pos_args, kw_args, &func_env);
                }
        }
        // 将模块限定名映射到内建函数
        let builtin_name = super::module_dispatch::module_builtin_name(name);
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
            return Ok((vec![], env.add_module(url.to_string())));
        }
        let base = env.base_path.clone();
        let load_paths = env.get_load_paths().to_vec();
        if let Some(path) = Self::resolve_file(base.as_ref(), url, &load_paths) {
            let already_loaded = env.loaded_modules.contains(&path);
            if already_loaded && !env.get_module_cache().contains_key(&path) {
                return Err(SassError::Module("Module loop: this module is already being loaded.".into()));
            }
            if already_loaded && !config.is_empty() {
                return Err(SassError::Eval(
                    "This module was already loaded, so it can't be configured using \"with\".".into(),
                ));
            }
            let exports = if already_loaded {
                env.get_module_cache().get(&path).cloned().unwrap_or_default()
            } else {
                let config_pairs: Vec<(String, Value)> = config
                    .iter()
                    .map(|c| {
                        let val = Self::eval_value(&c.value, &env)?;
                        Ok::<(String, Value), SassError>((c.name.clone(), val))
                    })
                    .collect::<Result<Vec<_>>>()?;
                Self::load_module(&path, &config_pairs, &env)?
            };
            let env_with_cache = merge_module_cache(env, &path, &exports);
            let css = if already_loaded { vec![] } else { exports.css.clone() };
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
            return Ok((css, env_with_cache.add_namespace(ns, exports)));
        }
        Ok((vec![], env))
    }

    /// @forward 指令处理。
    pub(crate) fn eval_forward(
        url: &str,
        prefix: &Option<String>,
        config: &[crate::parse::ast::ConfigVar],
        env: Env,
        show: &[String],
        hide: &[String],
    ) -> Result<(Vec<CssNode>, Env)> {
        let span = crate::__tracing::info_span!("eval_forward", url = url, has_prefix = prefix.is_some());
        let _enter = span.enter();
        // 内建模块（sass:xxx）不能用 with 配置
        if url.starts_with("sass:") && !config.is_empty() {
            return Err(SassError::Eval("Built-in modules can't be configured.".into()));
        }
        let base = env.base_path.clone();
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
                let mut pairs: Vec<(String, Value)> = env
                    .pending_config
                    .iter()
                    .map(|(k, v)| (strip_prefix(k), v.clone()))
                    .collect();
                for cfg in config {
                    let val = Evaluator::eval_value(&cfg.value, &env)?;
                    let name = strip_prefix(&cfg.name);
                    if cfg.is_default {
                        if !pairs.iter().any(|(n, _)| n == &name) && !matches!(val, Value::Null) {
                            pairs.push((name, val));
                        }
                    } else if !matches!(val, Value::Null) {
                        if let Some(idx) = pairs.iter().position(|(n, _)| n == &name) {
                            pairs[idx].1 = val;
                        } else {
                            pairs.push((name, val));
                        }
                    }
                }
                pairs
            };
            let already_loaded = env.loaded_modules.contains(&path);
            if already_loaded && !env.get_module_cache().contains_key(&path) {
                return Err(SassError::Module("Module loop: this module is already being loaded.".into()));
            }
            if already_loaded && !config.is_empty() {
                return Err(SassError::Eval(
                    "This module was already loaded, so it can't be configured using \"with\".".into(),
                ));
            }
            let exports = if already_loaded {
                env.get_module_cache().get(&path).cloned().unwrap_or_default()
            } else {
                Self::load_module(&path, &config_pairs, &env)?
            };
            let css = if already_loaded { vec![] } else { exports.css.clone() };
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
            return Ok((css, new_env));
        }
        Err(SassError::Eval("Can't find stylesheet to import.".into()))
    }
}
