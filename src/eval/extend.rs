use super::*;
use crate::css::node::CssNode;

impl Evaluator {
    /// 收集 CSS 中所有选择器文本（用于 extend target 匹配检查）。
    fn collect_selectors(nodes: &[CssNode]) -> Vec<String> {
        nodes.iter().flat_map(|node| {
            let mut own = Vec::new();
            let mut nested: Vec<String> = match node {
                CssNode::Rule { selector, children, .. } => {
                    own.push(selector.clone());
                    Self::collect_selectors(children)
                }
                CssNode::AtRule { children, .. } => Self::collect_selectors(children),
                CssNode::AtRoot(kids) => Self::collect_selectors(kids),
                _ => Vec::new(),
            };
            own.append(&mut nested);
            own
        }).collect()
    }

    pub(crate) fn apply_extends(nodes: Vec<CssNode>, extends: &[(String, String, bool)]) -> Vec<CssNode> {
        let span = crate::__tracing::info_span!("apply_extends", n_extends = extends.len());
        let _enter = span.enter();
        nodes.into_iter().map(|node| {
            match node {
                CssNode::Rule {
                    mut selector, children, declarations
                } => {
                    crate::__tracing::debug!(
                        target: "sasspile::extend",
                        selector = %selector,
                        "processing rule for extends"
                    );
                    // 应用 extend
                    for (extender, target, _optional) in extends {
                        let target_trimmed = target.trim();
                        let extender_trimmed = extender.trim();
                        // bogus 选择器检测
                        if extender_trimmed.ends_with('+')
                            || extender_trimmed.ends_with('>')
                            || extender_trimmed.ends_with('~')
                        {
                            continue;
                        }
                        if target_trimmed.starts_with('%') {
                            // 占位符：替换每个包含 target 的选择器部分
                            let parts: Vec<&str> = selector.split(',').collect();
                            let new_parts: Vec<String> = parts.iter().map(|p| {
                                let trimmed = p.trim();
                                if trimmed == target_trimmed || trimmed.contains(target_trimmed) {
                                    p.replace(target_trimmed, extender_trimmed)
                                } else {
                                    p.to_string()
                                }
                            }).collect();
                            selector = new_parts.join(",");
                            crate::__tracing::debug!(
                                target: "sasspile::extend",
                                new_selector = %selector,
                                "placeholder replaced"
                            );
                        } else {
                            // 普通选择器：逐个逗号分隔部分检查匹配
                            let parts: Vec<String> = selector.split(',').map(|s| s.trim().to_string()).collect();
                            let mut new_selectors: Vec<String> = Vec::new();
                            for part in &parts {
                                if part == target_trimmed || part.contains(target_trimmed) {
                                    // 替换 target 为 extender，生成新选择器
                                    let replaced = part.replace(target_trimmed, extender_trimmed);
                                    if !replaced.is_empty() && !new_selectors.contains(&replaced) {
                                        new_selectors.push(replaced);
                                    }
                                }
                            }
                            // 追加新选择器（去重）
                            for ns in &new_selectors {
                                if !selector.contains(ns.as_str()) {
                                    selector.push_str(", ");
                                    selector.push_str(ns);
                                }
                            }
                            crate::__tracing::debug!(
                                target: "sasspile::extend",
                                final_selector = %selector,
                                "extender appended"
                            );
                        }
                    }
                    // 递归处理子规则
                    let children = Self::apply_extends(children, extends);
                    // 移除未被继承的占位符选择器部分
                    let parts: Vec<&str> = selector
                        .split(',')
                        .filter(|s| !s.trim().starts_with('%'))
                        .collect();
                    selector = parts.join(",").trim().to_string();
                    CssNode::Rule { selector, declarations, children }
                }
                CssNode::AtRule {
                    name, params, children, has_body: true
                } => {
                    let children = Self::apply_extends(children, extends);
                    CssNode::AtRule { name, params, children, has_body: true }
                }
                CssNode::AtRoot(kids) => {
                    CssNode::AtRoot(Self::apply_extends(kids, extends))
                }
                other => other,
            }
        }).collect()
    }

    /// 检查未匹配的 extend target——非 optional 的未匹配 target 报错。
    pub(crate) fn check_extend_targets(
        css: &[CssNode],
        extends: &[(String, String, bool)],
    ) -> Result<()> {
        let span = crate::__tracing::debug_span!("check_extend_targets", n_extends = extends.len());
        let _enter = span.enter();
    let all_selectors = Self::collect_selectors(css);
    for (_extender, target, optional) in extends {
            if *optional {
                continue;
            }
            let target_trimmed = target.trim();
            // 占位符选择器不需要在 CSS 中存在
            if target_trimmed.starts_with('%') {
                continue;
            }
            let found = all_selectors.iter().any(|s| s.contains(target_trimmed));
            if !found {
                return Err(SassError::Eval(format!(
                    "The target selector was not found.\nUse \"@extend {target_trimmed} !optional\" to avoid this error."
                )));
            }
        }
        Ok(())
    }
}
