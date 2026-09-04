//! `@forward` 指令处理——模块成员转发。

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
                let new_env = new_env.with_consumed_config(merged_consumed);
                Ok((css, new_env))
            }
            None => Err(SassError::Eval("Can't find stylesheet to import.".into())),
        }
    }
}
