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
                        if selector.contains(target_trimmed) {
                            crate::__tracing::info!(
                                target: "sasspile::extend",
                                extender = %extender,
                                target = %target_trimmed,
                                selector = %selector,
                                "extend matched"
                            );
                            // bogus 选择器检测：extender 末尾为组合器（+ > ~）时跳过
                            let extender_trimmed = extender.trim();
                            if extender_trimmed.ends_with('+')
                                || extender_trimmed.ends_with('>')
                                || extender_trimmed.ends_with('~')
                            {
                                crate::__tracing::debug!(
                                    target: "sasspile::extend",
                                    extender = %extender_trimmed,
                                    "bogus combinator extender skipped"
                                );
                                continue;
                            }
                            if target_trimmed.starts_with('%') {
                                // 占位符：直接替换为目标
                                selector = selector.replace(target_trimmed, extender);
                                crate::__tracing::debug!(
                                    target: "sasspile::extend",
                                    new_selector = %selector,
                                    "placeholder replaced"
                                );
                            } else {
                                // 普通选择器：添加继承者作为额外选择器
                                let new_sel = selector.replace(target_trimmed, extender);
                                if !new_sel.is_empty() && new_sel != selector
                                    && !selector.contains(&new_sel) {
                                    selector.push_str(", ");
                                    selector.push_str(&new_sel);
                                    crate::__tracing::debug!(
                                        target: "sasspile::extend",
                                        final_selector = %selector,
                                        "extender appended"
                                    );
                                }
                            }
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
