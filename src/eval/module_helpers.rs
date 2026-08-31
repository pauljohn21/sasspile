//! 模块辅助函数——绑定、配置、合并逻辑。

use super::*;
use crate::error::{Result, SassError};
use crate::eval::value::values_eq;
use std::path::Path;

/// 返回内建模块的导出变量。
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
pub(crate) fn merge_with_local_precedence<'a>(
    local: &'a HashMap<String, Value>,
    forwarded: &'a HashMap<String, Value>,
) -> impl Iterator<Item = (&'a String, &'a Value)> {
    local.iter().chain(
        forwarded.iter().filter(|(k, _)| !local.contains_key(*k))
    )
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
    let fmt_key = |k: &str| -> String {
        prefix.map_or_else(|| k.to_string(), |p| format!("{p}{k}"))
    };
    match mode {
        BindMode::Use => {
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
                // Sass 私有成员约定：以 - 或 _ 开头的名称不通过 @forward 转发
                if k.starts_with('-') || k.starts_with('_') { continue; }
                let key = fmt_key(k);
                let var_key = format!("${key}");
                if !filter.show.is_empty() && !filter.show.contains(&var_key) { continue; }
                if filter.hide.contains(&var_key) { continue; }
                // 冲突检测：forwarded_vars 已存在同名时报错
                // 但如果值相同（来自同一底层模块）则跳过不报错
                if let Some(existing) = new_env.forwarded_vars.get(&key) {
                    // 冲突检测：值相同则跳过（values_eq + Display 字符串后备）
                    if !values_eq(existing, v) && format!("{existing}") != format!("{v}") {
                        return Err(SassError::Eval(format!(
                            "Two forwarded modules both define a variable named ${key}."
                        )));
                    }
                } else {
                    new_env.forwarded_vars.insert(key, v.clone());
                }
            }
            for (k, v) in &merged_mixins {
                // Sass 私有成员约定：以 - 或 _ 开头的名称不通过 @forward 转发
                if k.starts_with('-') || k.starts_with('_') { continue; }
                let key = fmt_key(k);
                if !filter.show.is_empty() && !filter.show.contains(&key) { continue; }
                if filter.hide.contains(&key) { continue; }
                if let Some(existing) = new_env.forwarded_mixins.get(&key) {
                    // 用 body Debug 比较相同则跳过
                    let existing_str = format!("{:?}", existing.body);
                    let new_str = format!("{:?}", v.body);
                    if existing_str != new_str {
                        return Err(SassError::Eval(format!(
                            "Two forwarded modules both define a mixin named {key}."
                        )));
                    }
                }
                new_env = new_env.define_forwarded_mixin(key, v.clone());
            }
            for (k, v) in &merged_functions {
                // Sass 私有成员约定：以 - 或 _ 开头的名称不通过 @forward 转发
                if k.starts_with('-') || k.starts_with('_') { continue; }
                let key = fmt_key(k);
                if !filter.show.is_empty() && !filter.show.contains(&key) { continue; }
                if filter.hide.contains(&key) { continue; }
                if let Some(existing) = new_env.forwarded_functions.get(&key) {
                    // 用 body Debug 比较相同则跳过
                    let existing_str = format!("{:?}", existing.body);
                    let new_str = format!("{:?}", v.body);
                    if existing_str != new_str {
                        return Err(SassError::Eval(format!(
                            "Two forwarded modules both define a function named {key}."
                        )));
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
