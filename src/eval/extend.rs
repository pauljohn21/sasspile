use super::*;
use crate::css::node::CssNode;
use crate::eval::selector::parse::parse_selector_list;

impl Evaluator {
    pub(crate) fn apply_extends(nodes: &mut [CssNode], extends: &[(String, String)]) {
        let span = crate::__tracing::info_span!("apply_extends", n_extends = extends.len());
        let _enter = span.enter();
        for node in nodes.iter_mut() {
            match node {
                CssNode::Rule {
                    selector, children, ..
                } => {
                    let selector_str = selector.to_string();
                    crate::__tracing::debug!(
                        target: "sasspile::extend",
                        selector = %selector_str,
                        "processing rule for extends"
                    );
                    // 应用 extend
                    for (extender, target) in extends {
                        let target_trimmed = target.trim();
                        if selector_str.contains(target_trimmed) {
                            crate::__tracing::info!(
                                target: "sasspile::extend",
                                extender = %extender,
                                target = %target_trimmed,
                                selector = %selector_str,
                                "extend matched"
                            );
                            if target_trimmed.starts_with('%') {
                                // 占位符：直接替换为目标
                                let new_selector_str = selector_str.replace(target_trimmed, extender);
                                *selector = parse_selector_list(&new_selector_str).unwrap_or_default();
                                crate::__tracing::debug!(
                                    target: "sasspile::extend",
                                    new_selector = %new_selector_str,
                                    "placeholder replaced"
                                );
                            } else {
                                // 普通选择器：添加继承者作为额外选择器
                                let new_sel = selector_str.replace(target_trimmed, extender);
                                if !new_sel.is_empty()
                                    && new_sel != selector_str
                                    && !selector_str.contains(&new_sel)
                                {
                                    let final_selector = format!("{selector_str}, {new_sel}");
                                    *selector = parse_selector_list(&final_selector).unwrap_or_default();
                                    crate::__tracing::debug!(
                                        target: "sasspile::extend",
                                        final_selector = %final_selector,
                                        "extender appended"
                                    );
                                }
                            }
                        }
                    }
                    // 递归处理子规则
                    Self::apply_extends(children, extends);
                    // 移除未被继承的占位符选择器部分
                    let sel_str = selector.to_string();
                    let parts: Vec<&str> = sel_str
                        .split(',')
                        .filter(|s| !s.trim().starts_with('%'))
                        .collect();
                    let cleaned = parts.join(",").trim().to_string();
                    *selector = parse_selector_list(&cleaned).unwrap_or_default();
                }
                CssNode::AtRule {
                    has_body: true,
                    children,
                    ..
                } => {
                    Self::apply_extends(children, extends);
                }
                CssNode::AtRoot(kids) => {
                    Self::apply_extends(kids, extends);
                }
                _ => {}
            }
        }
    }
}
