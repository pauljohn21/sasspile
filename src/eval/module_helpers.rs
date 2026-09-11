//! —— 模块辅助函数 ——
//!
//! 概要：提供模块系统的辅助函数（绑定、配置、合并逻辑）。
//!
//! ## 核心概念
//! - `bind_exports`：将模块导出绑定到当前作用域
//! - `merge_module_cache`：合并全局模块缓存
//! - `BindMode`：绑定模式（import/use/forward）
//! - `FilterConfig`：成员过滤配置（show/hide）

use super::*;
use crate::error::{Result, SassError};
use crate::eval::value::values_eq;
use std::path::Path;

use crate::eval::env::FunctionDef;
use crate::parse::ast::Param;

/// Placeholder def for a builtin function (body empty — dispatched by Rust code).
fn builtin_fn_def(name: &str) -> FunctionDef {
    FunctionDef {
        params: vec![Param {
            name: format!("${name}"),
            default: None,
            rest: false,
        }],
        body: vec![],
        captured_namespaces: HashMap::new(),
    }
}

/// Collect the global-name entries for a given builtin module prefix.
fn module_fn_names<'a>(names: &[(&'a str, &'a str)], module: &str) -> Vec<&'a str> {
    let prefix = format!("{module}.");
    names
        .iter()
        .filter_map(|(qname, gname)| {
            if qname.starts_with(&prefix) || *qname == module {
                Some(*gname)
            } else {
                None
            }
        })
        .collect()
}

/// 返回内建模块的导出变量和函数。
pub(crate) fn builtin_module_exports(module_name: &str) -> Option<ModuleExports> {
    use super::builtin::dispatch::{
        COLOR_NAMES, LIST_NAMES, MAP_NAMES, MATH_NAMES, META_NAMES, SELECTOR_NAMES,
        STRING_NAMES,
    };

    let fn_names: Vec<&str> = match module_name {
        "sass:math" => module_fn_names(MATH_NAMES, "math"),
        "sass:string" => module_fn_names(STRING_NAMES, "string"),
        "sass:map" => module_fn_names(MAP_NAMES, "map"),
        "sass:list" => module_fn_names(LIST_NAMES, "list"),
        "sass:color" => module_fn_names(COLOR_NAMES, "color"),
        "sass:selector" => module_fn_names(SELECTOR_NAMES, "selector"),
        "sass:meta" => module_fn_names(META_NAMES, "meta"),
        _ => vec![],
    };
    let fn_map: HashMap<String, FunctionDef> = fn_names
        .into_iter()
        .map(|n| (n.to_string(), builtin_fn_def(n)))
        .collect();

    match module_name {
        "sass:math" => {
            let mut vars = HashMap::new();
            vars.insert("pi".to_string(), Value::Number(std::f64::consts::PI, None));
            vars.insert("e".to_string(), Value::Number(std::f64::consts::E, None));
            vars.insert("epsilon".to_string(), Value::Number(f64::EPSILON, None));
            vars.insert(
                "max-safe-integer".to_string(),
                Value::Number(9_007_199_254_740_991.0, None),
            );
            vars.insert(
                "min-safe-integer".to_string(),
                Value::Number(-9_007_199_254_740_991.0, None),
            );
            vars.insert("max-number".to_string(), Value::Number(f64::MAX, None));
            vars.insert(
                "min-number".to_string(),
                Value::Number(f64::MIN_POSITIVE, None),
            );
            Some(ModuleExports {
                local_vars: vars,
                local_functions: fn_map,
                is_builtin: true,
                ..Default::default()
            })
        }
        n if n.starts_with("sass:") => Some(ModuleExports {
            local_functions: fn_map,
            is_builtin: true,
            ..Default::default()
        }),
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
pub(crate) fn merge_with_local_precedence<'a>(
    local: &'a HashMap<String, Value>,
    forwarded: &'a HashMap<String, Value>,
) -> impl Iterator<Item = (&'a String, &'a Value)> {
    local
        .iter()
        .chain(forwarded.iter().filter(|(k, _)| !local.contains_key(*k)))
}

/// 将模块导出绑定到环境。
pub(crate) fn bind_exports(
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
let fmt_key =
|k: &str| -> String { prefix.map_or_else(|| k.to_string(), |p| format!("{p}{k}")) };
    match mode {
        BindMode::Use => {
            // `as *` 模式：追踪每个成员名到 star_members 以检测冲突
            // 只在第一次加载该模块时记录（避免同模块多次 @use 导致误报）
            match prefix.is_none() {
                true => {
                    let stem = source_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");
                    let is_new_module = !new_env.star_module_loaded(stem);
                    match is_new_module {
                        true => {
                    // star_members 只追踪"真正本地定义或 forwarded"的成员，
                    // 排除通过 @use ... as * 引入的成员（star_imported），
                    // 避免嵌套 @use 误触发 star 冲突检测。
                    let member_names: Vec<&str> = exports
                        .local_vars
                        .iter()
                        .map(|(k, _)| k.as_str())
                        .filter(|k| {
                            !(k.starts_with('-')
                                || k.starts_with('_')
                                || (exports.star_imported.contains(*k)
                                    && !exports.forwarded_vars.contains_key(*k)))
                        })
                        .chain(
                            exports
                                .local_mixins
                                .keys()
                                .map(|k| k.as_str())
                                .filter(|k| {
                                    !(k.starts_with('-')
                                        || k.starts_with('_')
                                        || (exports.star_imported.contains(*k)
                                            && !exports.forwarded_mixins.contains_key(*k)))
                                }),
                        )
                        .chain(
                            exports
                                .local_functions
                                .keys()
                                .map(|k| k.as_str())
                                .filter(|k| {
                                    !(k.starts_with('-')
                                        || k.starts_with('_')
                                        || (exports.star_imported.contains(*k)
                                            && !exports.forwarded_functions.contains_key(*k)))
                                }),
                        )
                        .collect();
                    new_env = new_env.add_star_members(stem, &member_names);
                        }
                        false => {}
                    }
                }
                false => {}
            }
            // `as *` 模式（prefix=None）：过滤私有成员和通过 @use ... as * 传递引入的成员
            let is_star = prefix.is_none();
            let star_imported = &exports.star_imported;
            // 检测 as * 引入的变量与当前作用域已有变量冲突
            match is_star {
                true => {
                    for (k, _) in
                        merge_with_local_precedence(&exports.local_vars, &exports.forwarded_vars)
                    {
                        match k.starts_with('-')
                            || k.starts_with('_')
                            || star_imported.contains(k.as_str()) {
                            true => continue,
                            false => match new_env.has_var(k) && !new_env.star_imported.contains(k.as_str()) {
                                true => return Err(SassError::Eval(format!(
                                    "This module and the new module both define a variable named \"${k}\"."
                                ))),
                                false => {}
                            },
                        }
                    }
                }
                false => {}
            }
            new_env = merge_with_local_precedence(&exports.local_vars, &exports.forwarded_vars)
                .filter(|(k, _)| {
                    !is_star
                        || !(k.starts_with('-')
                            || k.starts_with('_')
                            || (star_imported.contains(k.as_str()) && !exports.forwarded_vars.contains_key(k.as_str())))
                })
                .fold(new_env, |env, (k, v)| {
                    let env = env.bind(fmt_key(k), v.clone());
                    match is_star {
                        true => env.add_star_imported(fmt_key(k)),
                        false => env,
                    }
                });
            new_env = exports
                .all_mixins()
                .filter(|(k, _)| {
                    !is_star
                        || !(k.starts_with('-')
                            || k.starts_with('_')
                            || (star_imported.contains(k.as_str()) && !exports.forwarded_mixins.contains_key(k.as_str())))
                })
                .fold(new_env, |env, (k, v)| {
                    let env = env.define_local_mixin(fmt_key(k), v.clone());
                    match is_star {
                        true => env.add_star_imported(fmt_key(k)),
                        false => env,
                    }
                });
            new_env = exports
                .all_functions()
                .filter(|(k, _)| {
                    !is_star
                        || !(k.starts_with('-')
                            || k.starts_with('_')
                            || (star_imported.contains(k.as_str()) && !exports.forwarded_functions.contains_key(k.as_str())))
                })
                .fold(new_env, |env, (k, v)| {
                    let env = env.define_local_function(fmt_key(k), v.clone());
                    match is_star {
                        true => env.add_star_imported(fmt_key(k)),
                        false => env,
                    }
                });
        }
        BindMode::Forward => {
            let merged_vars: Vec<(String, Value)> =
                merge_with_local_precedence(&exports.local_vars, &exports.forwarded_vars)
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
            let merged_mixins: Vec<(String, MixinDef)> = exports
                .local_mixins
                .iter()
                .chain(
                    exports
                        .forwarded_mixins
                        .iter()
                        .filter(|(k, _)| !exports.local_mixins.contains_key(*k)),
                )
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            let merged_functions: Vec<(String, FunctionDef)> = exports
                .local_functions
                .iter()
                .chain(
                    exports
                        .forwarded_functions
                        .iter()
                        .filter(|(k, _)| !exports.local_functions.contains_key(*k)),
                )
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            for (k, v) in &merged_vars {
                // Sass 私有成员约定：以 - 或 _ 开头的名称不通过 @forward 转发
                match k.starts_with('-') || k.starts_with('_') {
                    true => continue,
                    false => {}
                }
                let key = fmt_key(k);
                let var_key = format!("${key}");
                match !filter.show.is_empty() && !filter.show.contains(&var_key) {
                    true => continue,
                    false => {}
                }
                match filter.hide.contains(&var_key) {
                    true => continue,
                    false => {}
                }
                // 冲突检测：forwarded_vars 已存在同名时报错
                // 但如果值相同（来自同一底层模块）则跳过不报错
                if let Some(existing) = new_env.get_forwarded_var(&key) {
                    // 冲突检测：值相同则跳过（values_eq + Display 字符串后备）
                match !values_eq(existing, v) && format!("{existing}") != format!("{v}") {
                    true => return Err(SassError::Eval(format!(
                        "Two forwarded modules both define a variable named ${key}."
                    ))),
                    false => {}
                }
                } else {
                    new_env = new_env.define_forwarded_var(key, v.clone());
                }
            }
            for (k, v) in &merged_mixins {
                // Sass 私有成员约定：以 - 或 _ 开头的名称不通过 @forward 转发
                match k.starts_with('-') || k.starts_with('_') {
                    true => continue,
                    false => {}
                }
                let key = fmt_key(k);
                match !filter.show.is_empty() && !filter.show.contains(&key) {
                    true => continue,
                    false => {}
                }
                match filter.hide.contains(&key) {
                    true => continue,
                    false => {}
                }
                if let Some(existing) = new_env.get_forwarded_mixin(&key) {
                    let existing_str = format!("{:?}", existing.body);
                    let new_str = format!("{:?}", v.body);
                    match existing_str != new_str {
                        true => return Err(SassError::Eval(format!(
                            "Two forwarded modules both define a mixin named {key}."
                        ))),
                        false => {}
                    }
                }
                new_env = new_env.define_forwarded_mixin(key, v.clone());
            }
            for (k, v) in &merged_functions {
                // Sass 私有成员约定：以 - 或 _ 开头的名称不通过 @forward 转发
                match k.starts_with('-') || k.starts_with('_') {
                    true => continue,
                    false => {}
                }
                let key = fmt_key(k);
                match !filter.show.is_empty() && !filter.show.contains(&key) {
                    true => continue,
                    false => {}
                }
                match filter.hide.contains(&key) {
                    true => continue,
                    false => {}
                }
                if let Some(existing) = new_env.get_forwarded_function(&key) {
                    let existing_str = format!("{:?}", existing.body);
                    let new_str = format!("{:?}", v.body);
                    match existing_str != new_str {
                        true => return Err(SassError::Eval(format!(
                            "Two forwarded modules both define a function named {key}."
                        ))),
                        false => {}
                    }
                }
                new_env = new_env.define_forwarded_function(key, v.clone());
            }
        }
    }
    Ok(new_env)
}

/// 合并模块缓存和 @extend 关系。
pub(crate) fn merge_module_cache(env: Env, path: &Path, exports: &ModuleExports) -> Env {
    let mut new_loaded = (*env.get_loaded_modules()).clone();
    new_loaded.insert(path.to_path_buf());
    new_loaded.extend((*exports.loaded_modules).clone().iter().cloned());
    // 合并模块的 extends 到调用者 env（保留模块路径标记）
    let mut new_extends = (*env.get_extends()).to_vec();
    new_extends.extend((*exports.extends).clone().iter().cloned());
    let mut new_cache = (*env.get_module_cache()).clone();
    for (k, v) in &*exports.module_cache {
        new_cache.insert(k.clone(), v.clone());
    }
    env.with_loaded_modules(new_loaded)
        .with_extends(new_extends)
        .with_module_cache(new_cache)
}
