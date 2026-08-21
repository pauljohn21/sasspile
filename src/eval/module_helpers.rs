//! 模块辅助函数——绑定、配置、合并逻辑。

use super::*;
use crate::error::{Result, SassError};
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

/// 使用 ConfigVar 列表更新 inherited_vars。
pub(crate) fn apply_config(
    inherited_vars: &mut Vec<(String, Value)>,
    config: &[crate::parse::ast::ConfigVar],
    env: &Env,
) -> Result<()> {
    for cfg in config {
        let val = Evaluator::eval_value(&cfg.value, env)?;
        // null 配置值不注入——让上游模块的 !default 生效
        if matches!(val, Value::Null) && !cfg.is_default {
            // 移除已有的同名变量（如果有），让 !default 重新生效
            inherited_vars.retain(|(n, _)| n != cfg.name);
            continue;
        }
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
    Ok(())
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
                if !filter.show.is_empty() && !filter.show.iter().any(|s| *s == var_key) { continue; }
                if filter.hide.iter().any(|s| *s == var_key) { continue; }
                // 冲突检测：forwarded_vars 已存在同名且来源不同时报错
                if new_env.forwarded_vars.contains_key(&key) {
                    return Err(SassError::Eval(format!(
                        "Two forwarded modules both define a variable named ${key}."
                    )));
                }
                new_env.forwarded_vars.insert(key, v.clone());
            }
            for (k, v) in &merged_mixins {
                // Sass 私有成员约定：以 - 或 _ 开头的名称不通过 @forward 转发
                if k.starts_with('-') || k.starts_with('_') { continue; }
                let key = fmt_key(k);
                if !filter.show.is_empty() && !filter.show.iter().any(|s| *s == key) { continue; }
                if filter.hide.iter().any(|s| *s == key) { continue; }
                if new_env.forwarded_mixins.contains_key(&key) {
                    return Err(SassError::Eval(format!(
                        "Two forwarded modules both define a mixin named {key}."
                    )));
                }
                new_env = new_env.define_forwarded_mixin(key, v.clone());
            }
            for (k, v) in &merged_functions {
                // Sass 私有成员约定：以 - 或 _ 开头的名称不通过 @forward 转发
                if k.starts_with('-') || k.starts_with('_') { continue; }
                let key = fmt_key(k);
                if !filter.show.is_empty() && !filter.show.iter().any(|s| *s == key) { continue; }
                if filter.hide.iter().any(|s| *s == key) { continue; }
                if new_env.forwarded_functions.contains_key(&key) {
                    return Err(SassError::Eval(format!(
                        "Two forwarded modules both define a function named {key}."
                    )));
                }
                new_env = new_env.define_forwarded_function(key, v.clone());
            }
        }
    }
    Ok(new_env)
}

/// 合并模块缓存和 @extend 关系。
pub(crate) fn merge_module_cache(env: &Env, path: &Path, exports: &ModuleExports) -> Env {
    let mut new_loaded = (*env.loaded_modules).clone();
    new_loaded.insert(path.to_path_buf());
    new_loaded.extend((*exports.loaded_modules).clone().iter().cloned());
    let mut new_extends = (*env.extends).clone();
    new_extends.extend((*exports.extends).clone().iter().cloned());
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
