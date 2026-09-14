//! —— @forward 指令处理 ——
//!
//! 概要：处理 `@forward` 指令，将上游模块的成员转发到当前模块。
//!
//! ## 核心概念
//! - 解析 `@forward "url" as prefix-*` 形式（前缀/后缀过滤）
//! - 支持 `show` / `hide` 成员过滤
//! - `forwarded_vars` / `forwarded_functions` / `forwarded_mixins` 跟踪
//! - 转发成员的 namespace 重写

use super::module_helpers::{BindMode, FilterConfig, bind_exports, merge_module_cache};
use super::*;
use crate::error::{Result, SassError};

impl Evaluator {
    pub(crate) fn eval_forward(
        url: &str,
        prefix: &Option<String>,
        config: &[crate::parse::ast::ConfigVar],
        env: Env,
        show: &[String],
        hide: &[String],
    ) -> Result<(Vec<CssNode>, Env)> {
        // 内建模块（sass:xxx）不能用 with 配置
        match url.starts_with("sass:") && !config.is_empty() {
            true => return Err(SassError::Eval("Built-in modules can't be configured.".into())),
            false => {}
        }
        // @forward 不能加载 CSS 文件
        match !url.starts_with("sass:") && super::module_helpers::is_css_url(url) {
            true => return Err(SassError::Eval("CSS files can't be @forwarded.".into())),
            false => {}
        }
        // @forward 内建模块（sass:xxx）——注册内建模块命名空间
        match url.starts_with("sass:") {
            true => {
                let exports =
                    crate::eval::module_helpers::builtin_module_exports(url).unwrap_or_default();
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
                let new_env = new_env.add_module(url.to_string());
                return Ok((vec![], new_env));
            }
            false => {}
        }
        let base = env.get_base_path().cloned();
        let load_paths = env.get_load_paths().to_vec();
        match Self::resolve_file(base.as_ref(), url, &load_paths) {
            Some(path) => {
                let config_pairs: Vec<(String, Value)> = {
                    let prefix_str = prefix.as_deref();
                    let strip_prefix = |k: &str| -> String {
                        match prefix_str {
                            Some(p) => {
                                let pfx = p.replace('-', "_");
                                let k_norm = k.replace('-', "_");
                                match k_norm.starts_with(&pfx) {
                                    true => k_norm[pfx.len()..].to_string(),
                                    false => k_norm,
                                }
                            }
                            None => k.replace('-', "_"),
                        }
                    };
                    let passes_filter = |name: &str| -> bool {
                        let var_marker = format!("${name}");
                        match !show.is_empty() {
                            true => return show.iter().any(|s| s == &var_marker || s == name),
                            false => {}
                        }
                        match !hide.is_empty() {
                            true => return !hide.iter().any(|s| s == &var_marker || s == name),
                            false => {}
                        }
                        true
                    };
                    match config.is_empty() {
                        true => env
                            .get_pending_config()
                            .iter()
                            .filter(|(k, _)| passes_filter(k))
                            .map(|(k, v)| (strip_prefix(k), v.clone()))
                            .collect(),
                        false => {
                            let configured_names: std::collections::HashSet<String> =
                                config.iter().map(|c| strip_prefix(&c.name)).collect();
                            let from_config: Vec<(String, Value)> =
                                config.iter().try_fold(Vec::new(), |mut acc, cfg| {
                                    let name = strip_prefix(&cfg.name);
                                    let val = Evaluator::eval_value(&cfg.value, &env)?;
                                    let pending_val = env.get_pending_config().get(&name).or_else(|| {
                                        env.get_pending_config().get(&cfg.name.replace('-', "_"))
                                    });
                                    let val_is_null = matches!(val, Value::Null);
                                    let chosen = match (cfg.is_default, val_is_null) {
                                        (true, false) => pending_val
                                            .filter(|v| !matches!(v, Value::Null))
                                            .cloned()
                                            .or(Some(val)),
                                        (true, true) => pending_val
                                            .filter(|v| !matches!(v, Value::Null))
                                            .cloned(),
                                        (false, true) => pending_val
                                            .filter(|v| !matches!(v, Value::Null))
                                            .cloned(),
                                        (false, false) => Some(val),
                                    };
                                    match chosen {
                                        Some(v) => acc.push((name, v)),
                                        None => {}
                                    }
                                    Ok::<_, SassError>(acc)
                                })?;
                            let mut result = from_config;
                            // 同时继承外层 pending_config 中未在当前 with 声明的配置（传播语义）
                            let extra: Vec<(String, Value)> = env
                                .get_pending_config()
                                .iter()
                                .filter(|(k, v)| {
                                    let stripped = strip_prefix(k);
                                    !configured_names.contains(&stripped)
                                        && !matches!(v, Value::Null)
                                        && passes_filter(k)
                                })
                                .map(|(k, v)| (strip_prefix(k), v.clone()))
                                .collect();
                            result.extend(extra);
                            result
                        }
                    }
                };
                crate::__tracing::debug!(
                    config_pairs = ?config_pairs.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>(),
                    pending = ?env.get_pending_config().keys().collect::<Vec<_>>(),
                    "eval_forward: built config_pairs"
                );
                let already_loaded = env.get_loaded_modules().contains(&path);
                match already_loaded && !env.get_module_cache().contains_key(&path) {
                    true => return Err(SassError::Module(
                        "Module loop: this module is already being loaded.".into(),
                    )),
                    false => {}
                }
                match already_loaded && !config.is_empty() {
                    true => return Err(SassError::Eval(
                        "This module was already loaded, so it can't be configured using \"with\"."
                            .into(),
                    )),
                    false => {}
                }
                let exports = match already_loaded {
                    true => env
                        .get_module_cache()
                        .get(&path)
                        .cloned()
                        .unwrap_or_default(),
                    false => Self::load_module(&path, &config_pairs, &env, false)?,
                };
                // @forward 自带 with(...) 时：仅验证实际有值的声明配置名
                // 通过检查 exports.consumed_config 判断目标模块是否声明了 !default
                match !config.is_empty() {
                    true => {
                        let prefix_str = prefix.as_deref();
                        let strip_p = |k: &str| -> String {
                            match prefix_str {
                                Some(p) => {
                                    let pfx = p.replace('-', "_");
                                    let k_norm = k.replace('-', "_");
                                    match k_norm.starts_with(&pfx) {
                                        true => k_norm[pfx.len()..].to_string(),
                                        false => k_norm,
                                    }
                                }
                                None => k.replace('-', "_"),
                            }
                        };
                        let is_null = |v: &Value| matches!(v, Value::Null);
                        let declared_names: std::collections::HashSet<String> = config
                            .iter()
                            .filter_map(|c| {
                                let name = strip_p(&c.name);
                                let val = Evaluator::eval_value(&c.value, &env).ok()?;
                                let has_value = match c.is_default {
                                    true => env.get_pending_config().get(&name).is_some()
                                        || !is_null(&val),
                                    false => !is_null(&val),
                                };
                                match has_value {
                                    true => Some(name),
                                    false => None,
                                }
                            })
                            .collect();
                        for name in &declared_names {
                            match !exports.consumed_config.contains(name) {
                                true => {
                                    return Err(SassError::Eval(
                                        "This variable was not declared with !default in the @used module.".into(),
                                    ));
                                }
                                false => {}
                            }
                        }
                    }
                    false => {}
                }
                let css = match already_loaded {
                    true => vec![],
                    false => {
                        let module_css = exports.css.clone();
                        match module_css.is_empty() {
                            true => vec![],
                            false => {
                                let marker = match config.is_empty() {
                                    true => None,
                                    false => Some("configured".to_string()),
                                };
                                vec![crate::css::node::CssNode::AtRoot(module_css, marker)]
                            }
                        }
                    }
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
                let prefix_norm = prefix.as_deref().map(|p| p.replace('-', "_"));
                let add_prefix = |k: &str| -> String {
                    match &prefix_norm {
                        Some(pfx) => format!("{pfx}{k}"),
                        None => k.to_string(),
                    }
                };
                let forward_with_names: std::collections::HashSet<String> = config
                    .iter()
                    .filter(|c| !c.is_default)
                    .map(|c| {
                        match prefix.as_deref() {
                            Some(p) => {
                                let pfx = p.replace('-', "_");
                                let k_norm = c.name.replace('-', "_");
                                match k_norm.starts_with(&pfx) {
                                    true => k_norm[pfx.len()..].to_string(),
                                    false => k_norm,
                                }
                            }
                            None => c.name.replace('-', "_"),
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
                let new_env = new_env.with_consumed_config(merged_consumed.into());
                Ok((css, new_env))
            }
            None => Err(SassError::Eval("Can't find stylesheet to import.".into())),
        }
    }
}
