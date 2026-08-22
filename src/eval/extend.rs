//! @extend 后处理——选择器匹配 + 替换。
//!
//! 遍历 CSS 树，将 extend target 选择器替换或追加 extender。

use crate::css::CssNode;

/// 收集 CSS 中所有选择器文本（用于 extend target 匹配检查）。
fn collect_selectors(nodes: &[CssNode], out: &mut Vec<String>) {
    for node in nodes {
        match node {
            CssNode::Rule { selector, children, .. } => {
                out.push(selector.clone());
                collect_selectors(children, out);
            }
            CssNode::AtRule { children, .. } => {
                collect_selectors(children, out);
            }
            CssNode::AtRoot(kids) => {
                collect_selectors(kids, out);
            }
            _ => {}
        }
    }
}

/// 应用 @extend——遍历 CSS 树，对每个规则的选择器执行 extend 匹配。
pub fn apply_extends(nodes: &mut [CssNode], extends: &[(String, String, bool)]) {
    if extends.is_empty() {
        return;
    }
    for node in nodes.iter_mut() {
        match node {
            CssNode::Rule { selector, children, .. } => {
                for (extender, target, _optional) in extends {
                    let target_trimmed = target.trim();
                    if selector.contains(target_trimmed) {
                        // bogus 选择器检测：extender 末尾为组合器时跳过
                        let extender_trimmed = extender.trim();
                        if extender_trimmed.ends_with('+')
                            || extender_trimmed.ends_with('>')
                            || extender_trimmed.ends_with('~')
                        {
                            continue;
                        }
                        if target_trimmed.starts_with('%') {
                            // 占位符：直接替换
                            *selector = selector.replace(target_trimmed, extender);
                        } else {
                            // 普通选择器：追加 extender 作为额外选择器
                            let new_sel = selector.replace(target_trimmed, extender);
                            if !new_sel.is_empty()
                                && new_sel != *selector
                                && !selector.contains(&new_sel)
                            {
                                selector.push_str(", ");
                                selector.push_str(&new_sel);
                            }
                        }
                    }
                }
                // 递归处理子规则
                apply_extends(children, extends);
                // 移除未被继承的占位符选择器部分
                let parts: Vec<&str> = selector
                    .split(',')
                    .filter(|s| !s.trim().starts_with('%'))
                    .collect();
                *selector = parts.join(",").trim().to_string();
            }
            CssNode::AtRule { has_body: true, children, .. } => {
                apply_extends(children, extends);
            }
            CssNode::AtRoot(kids) => {
                apply_extends(kids, extends);
            }
            _ => {}
        }
    }
}

/// 检查未匹配的 extend target——非 optional 的未匹配 target 报错。
pub fn check_extend_targets(
    css: &[CssNode],
    extends: &[(String, String, bool)],
) -> crate::error::Result<()> {
    let mut all_selectors = Vec::new();
    collect_selectors(css, &mut all_selectors);
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
            return Err(crate::error::SassError::eval(format!(
                "The target selector was not found.\nUse \"@extend {target_trimmed} !optional\" to avoid this error."
            )));
        }
    }
    Ok(())
}
