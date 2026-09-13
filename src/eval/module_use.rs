//! @use 指令处理——模块加载、命名空间管理、配置验证。
//!
//! 从 `module.rs` 拆分出来，包含 `eval_use` 的完整逻辑。

use super::*;
use crate::error::{Result, SassError};

use super::module_helpers::{BindMode, FilterConfig, bind_exports, is_css_url, merge_module_cache};

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
        // @use 不能加载 CSS 文件
        match !url.starts_with("sass:") && is_css_url(url) {
            true => return Err(SassError::Eval("CSS files can't be @used.".into())),
            false => {}
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
