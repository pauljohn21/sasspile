use super::*;
use crate::error::{Result, SassError};
use std::path::{Path, PathBuf};

impl Evaluator {
    pub(crate) fn resolve_file(
        base: Option<&PathBuf>,
        url: &str,
        load_paths: &[PathBuf],
    ) -> Option<PathBuf> {
        let span = crate::__tracing::debug_span!("resolve_file", url = url);
        let _enter = span.enter();
        let base_dir = base
            .as_ref()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        // 先尝试相对于当前文件目录解析
        if let Some(path) = Self::try_resolve_dir(&base_dir, url) {
            return Some(path);
        }
        // 回退到 load paths
        for lp in load_paths {
            if let Some(path) = Self::try_resolve_dir(lp, url) {
                return Some(path);
            }
        }
        None
    }

    /// 在指定目录下尝试解析 url 对应的文件。
    fn try_resolve_dir(dir: &Path, url: &str) -> Option<PathBuf> {
        let url_path = std::path::Path::new(url);
        let parent = url_path.parent().unwrap_or(std::path::Path::new(""));
        let filename = url_path
            .file_stem() // file_stem 自动去除扩展名
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| url.to_string());
        let candidates = [
            dir.join(parent).join(format!("_{filename}.scss")),
            dir.join(parent).join(format!("{filename}.scss")),
            dir.join(parent).join(format!("_{filename}.sass")),
            dir.join(parent).join(format!("{filename}.sass")),
            dir.join(parent).join(format!("_{filename}.css")),
            dir.join(parent).join(format!("{filename}.css")),
            dir.join(parent).join(format!("_{filename}.import.scss")),
            dir.join(parent).join(format!("{filename}.import.scss")),
            dir.join(url).join("_index.scss"),
            dir.join(url).join("index.scss"),
            dir.join(url).join("_index.sass"),
            dir.join(url).join("index.sass"),
        ];
        for c in &candidates {
            if c.exists() {
                return Some(c.clone());
            }
        }
        None
    }

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
                // 返回缓存的 exports，但 CSS 为空（不重复输出）
                let cached_exports = ModuleExports {
                    css: vec![],
                    ..cached.clone()
                };
                return Ok(cached_exports);
            }
            // 缓存未命中（不应该发生），回退到空 exports
            return Ok(ModuleExports::default());
        }
        let source = std::fs::read_to_string(path)
            .map_err(|e| SassError::Module(format!("Cannot read {}: {e}", path.display())))?;

        // `.css` 文件——以 plain CSS 模式解析，保留嵌套不展开选择器
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
        // 传递已加载模块缓存（Rc 共享），并把当前路径加入缓存
        let mut loaded = (*caller_env.loaded_modules).clone();
        loaded.insert(path.to_path_buf());
        env.loaded_modules = Rc::new(loaded);
        // 注入 with() 配置变量到 pending_config——不进入 local_vars，
        // 使 meta.variable-exists() 在声明前返回 false，!default 赋值时消费
        for (name, value) in config {
            let val = Self::eval_value(value, caller_env)?;
            // null 配置值不注入——让上游 !default 生效
            if !matches!(val, Value::Null) {
                let key = name.replace('-', "_");
                env.pending_config.insert(key, val);
            }
        }
        let (module_css, final_env) = Self::eval_nodes(&ast.nodes, &env)?;
        // plain CSS 输出用 AtRoot 包裹，防止序列化器展平嵌套
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
        // 将 exports 存入缓存（CSS 为空，避免重复输出），供后续 @use/@forward 引用
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
    pub(crate) fn load_import(path: &Path, caller_env: &Env) -> Result<(Vec<CssNode>, Env)> {
        let span =
            crate::__tracing::info_span!("load_import", path = %path.display(), depth = caller_env.depth);
        let _enter = span.enter();
        // 循环加载检测：已加载但缓存中无模块 → 正在加载中
        if caller_env.loaded_modules.contains(path) && !caller_env.get_module_cache().contains_key(path) {
            return Err(SassError::Module("This file is already being loaded.".into()));
        }
        if caller_env.depth > 50 {
            return Ok((vec![], caller_env.clone()));
        }
        let source = std::fs::read_to_string(path)
            .map_err(|e| SassError::Module(format!("Cannot read {}: {e}", path.display())))?;

        // `.css` 文件——以 plain CSS 模式解析
        let is_plain_css = path.extension().and_then(|e| e.to_str()) == Some("css");

        let tokens: Vec<Token> = Lexer::new(&source)
            .filter(|t| !matches!(t.as_ref(), Ok(Token::Eof)))
            .collect::<Result<Vec<_>>>()?;
        let ast = crate::parse::Parser::parse(&tokens)?;
        // 继承当前环境的所有成员（变量、mixin、函数、命名空间）
        let mut env = caller_env.clone();
        env.base_path = Some(path.to_path_buf());
        env.depth = caller_env.depth + 1;
        env.plain_css = is_plain_css;
        let (css, mut final_env) = Self::eval_nodes(&ast.nodes, &env)?;
        // 恢复调用者的 base_path 和 depth，使父作用域后续 @import 使用正确的基准目录
        final_env.base_path = caller_env.base_path.clone();
        final_env.depth = caller_env.depth;
        // @import 内联语义：forwarded 成员应合并到 local（@import 把一切引入当前作用域）
        for (k, v) in final_env.forwarded_vars.clone() {
            final_env.local_vars.entry(k).or_insert(v);
        }
        for (k, v) in final_env.forwarded_mixins.clone() {
            final_env.local_mixins.entry(k).or_insert(v);
        }
        for (k, v) in final_env.forwarded_functions.clone() {
            final_env.local_functions.entry(k).or_insert(v);
        }
        // @import 内联的变量应传播到外层——把 partial 中新增的变量写入 global_writes
        for (name, val) in &final_env.local_vars {
            if !caller_env.local_vars.contains_key(name) {
                final_env.global_writes.insert(name.clone(), val.clone());
            }
        }
        // plain CSS 输出用 AtRoot 包裹
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

/// 返回内建模块的导出变量。
/// sass:math 模块导出 $pi, $e, $epsilon, $max-safe-integer, $min-safe-integer, $max-number, $min-number。
pub(crate) fn builtin_module_exports(module_name: &str) -> Option<ModuleExports> {
    match module_name {
        "sass:math" => {
            let mut vars = HashMap::new();
            vars.insert("pi".to_string(), Value::Number(std::f64::consts::PI, None));
            vars.insert("e".to_string(), Value::Number(std::f64::consts::E, None));
            vars.insert("epsilon".to_string(), Value::Number(f64::EPSILON, None));
            vars.insert("max-safe-integer".to_string(), Value::Number(9007199254740991.0, None));
            vars.insert("min-safe-integer".to_string(), Value::Number(-9007199254740991.0, None));
            vars.insert("max-number".to_string(), Value::Number(f64::MAX, None));
            vars.insert("min-number".to_string(), Value::Number(f64::MIN_POSITIVE, None));
            Some(ModuleExports {
                local_vars: vars,
                ..Default::default()
            })
        }
        _ => None,
    }
}

/// 绑定模式：Use 写入 local 表，Forward 写入 forwarded 表。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum BindMode {
    Use,
    Forward,
}

/// 前缀过滤配置：show/hide。
#[derive(Clone, Default)]
pub(crate) struct FilterConfig {
    pub(crate) show: Vec<String>,
    pub(crate) hide: Vec<String>,
}

/// 合并 local 和 forwarded（local 优先）。
fn merge_with_local_precedence<'a>(
    local: &'a HashMap<String, Value>,
    forwarded: &'a HashMap<String, Value>,
) -> impl Iterator<Item = (&'a String, &'a Value)> {
    local.iter().chain(
        forwarded.iter().filter(|(k, _)| !local.contains_key(*k))
    )
}

/// 将模块导出绑定到环境（支持前缀、模式和来源路径）。
/// Use 模式写入 local 表；Forward 模式写入 forwarded 表。
/// 冲突检测基于来源路径：同来源跳过，不同来源报错。
fn bind_exports(
    env: Env,
    exports: &ModuleExports,
    prefix: Option<&str>,
    mode: BindMode,
    source_path: &Path,
    filter: &FilterConfig,
) -> Result<Env> {
    let span = crate::__tracing::debug_span!("bind_exports", mode = ?mode, source = %source_path.display());
    let _enter = span.enter();
    let mut new_env = env;
    let fmt_key = |k: &str| -> String {
        prefix.map_or_else(|| k.to_string(), |p| format!("{p}{k}"))
    };
    // 根据模式选择源 exports 和目标表
    // Use: 从 exports.local_* 绑定到 env.local_*
    // Forward: 合并 exports.local_* + exports.forwarded_*（local 优先）后绑定到 env.forwarded_*
    match mode {
        BindMode::Use => {
            // Use 模式：从 local + forwarded 合并后绑定到 local 表（local 优先），后写覆盖先写
            for (k, v) in merge_with_local_precedence(&exports.local_vars, &exports.forwarded_vars) {
                let key = fmt_key(k);
                new_env = new_env.bind(key, v.clone());
            }
            for (k, v) in exports.all_mixins() {
                let key = fmt_key(k);
                new_env = new_env.define_local_mixin(key, v.clone());
            }
            for (k, v) in exports.all_functions() {
                let key = fmt_key(k);
                new_env = new_env.define_local_function(key, v.clone());
            }
        }
        BindMode::Forward => {
            // Forward 模式：合并 local + forwarded（local 优先）
            let merged_vars: Vec<(String, Value)> = merge_with_local_precedence(&exports.local_vars, &exports.forwarded_vars)
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            let merged_mixins: Vec<(String, MixinDef)> = exports.local_mixins.iter()
                .chain(exports.forwarded_mixins.iter().filter(|(k, _)| !exports.local_mixins.contains_key(*k)))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            let merged_functions: Vec<(String, FunctionDef)> = exports.local_functions.iter()
                .chain(exports.forwarded_functions.iter().filter(|(k, _)| !exports.local_functions.contains_key(*k)))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            for (k, v) in &merged_vars {
                let key = fmt_key(k);
                // 变量名带 $ 前缀比较：filter 中 $a → 比较 $key
                let var_key = format!("${key}");
                if !filter.show.is_empty() && !filter.show.iter().any(|s| *s == var_key) { continue; }
                if filter.hide.iter().any(|s| *s == var_key) { continue; }
                // Forward 模式：直接写入 forwarded 表，后写覆盖先写
                new_env.forwarded_vars.insert(key, v.clone());
            }
            for (k, v) in &merged_mixins {
                let key = fmt_key(k);
                if !filter.show.is_empty() && !filter.show.iter().any(|s| *s == key) { continue; }
                if filter.hide.iter().any(|s| *s == key) { continue; }
                new_env = new_env.define_forwarded_mixin(key, v.clone());
            }
            for (k, v) in &merged_functions {
                let key = fmt_key(k);
                if !filter.show.is_empty() && !filter.show.iter().any(|s| *s == key) { continue; }
                if filter.hide.iter().any(|s| *s == key) { continue; }
                new_env = new_env.define_forwarded_function(key, v.clone());
            }
        }
    }
    Ok(new_env)
}

/// 合并模块缓存和 @extend 关系。
fn merge_module_cache(env: &Env, path: &Path, exports: &ModuleExports) -> Env {
    let mut new_loaded = (*env.loaded_modules).clone();
    new_loaded.insert(path.to_path_buf());
    new_loaded.extend((*exports.loaded_modules).clone().iter().cloned());
    let mut new_extends = (*env.extends).clone();
    new_extends.extend((*exports.extends).clone().iter().cloned());
    // 合并 env 的 module_cache 和 exports 的 module_cache（exports 可能缺少 env 已有的缓存）
    let mut new_cache = (*env.module_cache).clone();
    for (k, v) in &*exports.module_cache {
        new_cache.insert(k.clone(), v.clone());
    }
    Env {
        loaded_modules: Rc::new(new_loaded),
        extends: Rc::new(new_extends),
        module_cache: Rc::new(new_cache),
        ..env.clone()
    }
}

impl Evaluator {
    /// @use 指令处理。
    pub(crate) fn eval_use(
        url: &str,
        namespace: &Option<String>,
        star: bool,
        config: &[crate::parse::ast::ConfigVar],
        env: &Env,
    ) -> Result<(Vec<CssNode>, Env)> {
        // 内建模块 sass:math/string/list/map/color/meta/selector
        if url.starts_with("sass:") {
            return Ok((vec![], env.add_module(url.to_string())));
        }
        let base = env.base_path.as_ref();
        let load_paths = env.get_load_paths();
        if let Some(path) = Self::resolve_file(base, url, load_paths) {
            let already_loaded = env.loaded_modules.contains(&path);
            // 循环加载检测：已加入 loaded 但缓存中无模块 → 正在加载中
            if already_loaded && !env.get_module_cache().contains_key(&path) {
                return Err(SassError::Module("Module loop: this module is already being loaded.".into()));
            }
            let exports = if already_loaded {
                // 从缓存获取 exports（CSS 为空，但 vars/mixins/functions 有效）
                env.get_module_cache().get(&path).cloned().unwrap_or_default()
            } else {
                let config_pairs: Vec<(String, Value)> = config
                    .iter()
                    .map(|c| {
                        let val = Self::eval_value(&c.value, env).unwrap_or(Value::Null);
                        (c.name.clone(), val)
                    })
                    .collect();
                Self::load_module(&path, &config_pairs, env)?
            };
            let env_with_cache = merge_module_cache(env, &path, &exports);
            let css = if already_loaded { vec![] } else { exports.css.clone() };
            if star {
                // @use as *：从模块的 local 成员绑定到当前文件的 local 表
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
        Ok((vec![], env.clone()))
    }

    /// @forward 指令处理。
    pub(crate) fn eval_forward(
        url: &str,
        prefix: &Option<String>,
        config: &[crate::parse::ast::ConfigVar],
        env: &Env,
        show: &[String],
        hide: &[String],
    ) -> Result<(Vec<CssNode>, Env)> {
        let span = crate::__tracing::info_span!("eval_forward", url = url, has_prefix = prefix.is_some());
        let _enter = span.enter();
        let base = env.base_path.as_ref();
        let load_paths = env.get_load_paths();
        if let Some(path) = Self::resolve_file(base, url, load_paths) {
            // @forward with 配置值 + 从下游继承的 pending_config 透传给上游
            // 处理 as 前缀：pending_config 中的 "b-a" → 去掉 "b-" 前缀 → "a"
            let config_pairs: Vec<(String, Value)> = {
                let prefix_str = prefix.as_deref();
                let strip_prefix = |k: &str| -> String {
                    if let Some(p) = prefix_str {
                        // 前缀已含 -（如 "b-"），normalize 为 "b_" 后匹配
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
                    let val = Evaluator::eval_value(&cfg.value, env)?;
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
            // 循环加载检测：已加入 loaded 但缓存中无模块 → 正在加载中
            if already_loaded && !env.get_module_cache().contains_key(&path) {
                return Err(SassError::Module("Module loop: this module is already being loaded.".into()));
            }
            let exports = if already_loaded {
                env.get_module_cache().get(&path).cloned().unwrap_or_default()
            } else {
                Self::load_module(&path, &config_pairs, env)?
            };
            let css = if already_loaded { vec![] } else { exports.css.clone() };
            let env_with_cache = merge_module_cache(env, &path, &exports);
            // Forward 模式：合并 local + forwarded（local 优先）后写入 forwarded 表
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
