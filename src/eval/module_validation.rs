//! 模块配置验证——@use with / @forward with 的参数验证逻辑。

use crate::error::{Result, SassError};
use crate::parse::ast::{ConfigVar, Node};
use std::collections::HashSet;

/// 从 AST 节点列表中收集所有带 `!default` 的顶层变量名。
pub(crate) fn collect_default_vars(nodes: &[Node]) -> HashSet<String> {
    let mut vars = HashSet::new();
    for node in nodes {
        if let Node::Variable { name, flags, .. } = node {
            if flags.default {
                vars.insert(name.clone());
            }
        }
    }
    vars
}

/// 验证 @forward/@use with 配置参数。
///
/// 检查内容：
/// 1. 内建模块（sass:*）不接受 with 配置
/// 2. 重复变量配置检测
/// 3. 已加载的模块不能再次 with 配置（多配置冲突）
/// 4. 配置变量在上游模块中必须带 !default
pub(crate) fn validate_config(
    url: &str,
    config: &[ConfigVar],
    already_loaded: bool,
    default_vars: &HashSet<String>,
) -> Result<()> {
    let span = crate::__tracing::debug_span!("validate_config", url = url, n_config = config.len());
    let _enter = span.enter();

    // 1. 内建模块不接受 with 配置
    if url.starts_with("sass:") && !config.is_empty() {
        return Err(SassError::Eval(
            "Built-in modules can't be configured.".into(),
        ));
    }

    // 2. 重复变量配置检测
    let mut seen: HashSet<&str> = HashSet::new();
    for cfg in config {
        if !seen.insert(cfg.name.as_str()) {
            return Err(SassError::Eval(
                "The same variable may only be configured once.".into(),
            ));
        }
    }

    // 3. 已加载的模块不能再次 with 配置（多配置冲突）
    if already_loaded && !config.is_empty() {
        return Err(SassError::Eval(
            "This module was already loaded, so it can't be configured using \"with\".".into(),
        ));
    }

    // 4. 配置变量在上游模块中必须带 !default
    for cfg in config {
        if !default_vars.contains(&cfg.name) {
            return Err(SassError::Eval(
                "This variable was not declared with !default in the @used module.".into(),
            ));
        }
    }

    Ok(())
}
