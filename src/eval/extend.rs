use super::*;
use crate::css::node::CssNode;

impl Evaluator {
    pub(crate) fn apply_extends(nodes: &mut [CssNode], extends: &[(String, String)]) -> Result<()> {
        let span = crate::__tracing::info_span!("apply_extends", n_extends = extends.len());
        let _enter = span.enter();
        // 内存检查 —— 链式反应：超限返 Err，上层 evaluate() 感知并释放
        memory_limit::check_memory_limit()?;
        for node in nodes.iter_mut() {
            match node {
                CssNode::Rule {
                    selector, children, ..
                } => {
                    crate::__tracing::debug!(
                        target: "sasspile::extend",
                        selector = %selector,
                        "processing rule for extends"
                    );
                    // 应用 extend
                    for (extender, target) in extends {
                        let target_trimmed = target.trim();
                        if selector.contains(target_trimmed) {
                            crate::__tracing::info!(
                                target: "sasspile::extend",
                                extender = %extender,
                                target = %target_trimmed,
                                selector = %selector,
                                "extend matched"
                            );
                            if target_trimmed.starts_with('%') {
                                // 占位符：直接替换为目标
                                *selector = selector.replace(target_trimmed, extender);
                                crate::__tracing::debug!(
                                    target: "sasspile::extend",
                                    new_selector = %selector,
                                    "placeholder replaced"
                                );
                            } else {
                                // 普通选择器：添加继承者作为额外选择器
                                // **防爆炸**：防止选择器嵌套膨胀
                                if extender.split(',').any(|e| e.trim() == selector.trim()) {
                                    crate::__tracing::warn!(
                                        target: "sasspile::extend",
                                        extender = %extender,
                                        selector = %selector,
                                        "跳过：extender 已包含选择器，避免嵌套膨胀"
                                    );
                                    continue;
                                }
                                let new_sel = selector.replace(target_trimmed, extender);
                                if !new_sel.is_empty() && new_sel != *selector
                                    && !selector.contains(&new_sel) {
                                        // **防爆炸**：选择器超 256 字符拒绝追加
                                        if selector.len() + new_sel.len() + 2 > 256 {
                                            crate::__tracing::warn!(
                                                target: "sasspile::extend",
                                                selector_len = selector.len(),
                                                "跳过：选择器过长，防止指数膨胀"
                                            );
                                            continue;
                                        }
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
                    Self::apply_extends(children, extends)?;
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
                    Self::apply_extends(children, extends)?;
                }
                CssNode::AtRoot(kids) => {
                    Self::apply_extends(kids, extends)?;
                }
                _ => {}
            }
        }
        Ok(())
    }
}
