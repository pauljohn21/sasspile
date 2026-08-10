use super::*;
use crate::css::node::CssNode;
use tracing::instrument;

impl Evaluator {
    pub(crate) fn apply_extends(nodes: &mut [CssNode], extends: &[(String, String)]) {
        let span = tracing::info_span!("apply_extends", n_extends = extends.len());
        let _enter = span.enter();
        for node in nodes.iter_mut() {
            match node {
                CssNode::Rule {
                    selector, children, ..
                } => {
                    tracing::debug!(
                        target: "sasspile::extend",
                        selector = %selector,
                        "processing rule for extends"
                    );
                    // 应用 extend
                    for (extender, target) in extends {
                        let target_trimmed = target.trim();
                        if selector.contains(target_trimmed) {
                            tracing::info!(
                                target: "sasspile::extend",
                                extender = %extender,
                                target = %target_trimmed,
                                selector = %selector,
                                "extend matched"
                            );
                            if target_trimmed.starts_with('%') {
                                // 占位符：直接替换为目标
                                *selector = selector.replace(target_trimmed, extender);
                                tracing::debug!(
                                    target: "sasspile::extend",
                                    new_selector = %selector,
                                    "placeholder replaced"
                                );
                            } else {
                                // 普通选择器：添加继承者作为额外选择器
                                let new_sel = selector.replace(target_trimmed, extender);
                                if !new_sel.is_empty() && new_sel != *selector {
                                    if !selector.contains(&new_sel) {
                                        selector.push_str(", ");
                                        selector.push_str(&new_sel);
                                        tracing::debug!(
                                            target: "sasspile::extend",
                                            final_selector = %selector,
                                            "extender appended"
                                        );
                                    }
                                }
                            }
                        }
                    }
                    // 递归处理子规则
                    Self::apply_extends(children, extends);
                    // 移除未被继承的占位符选择器部分
                    let parts: Vec<&str> = selector
                        .split(',')
                        .filter(|s| !s.trim().starts_with('%'))
                        .collect();
                    *selector = parts.join(",").trim().to_string();
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
