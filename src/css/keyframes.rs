//! —— Keyframes 后处理 ——
//!
//! 概要：实现 CSS Animations 规范要求的首尾空步骤剥离。
//!
//! 当一个 keyframes 规则的首步是 `0%`/`from` 空步骤，或末步是 `100%`/`to` 空步骤时，
//! 依照 dart-sass 行为将其移除，以避免输出 `@keyframes X { 0% {} 50% { ... } }` 这类
//! 含无效空步骤的 CSS。
//!
//! 仅移除**边界**的空步骤——中间的空步骤保留不动。

use crate::css::node::CssNode;
use crate::parse::at_rule_kinds::CssAtRule;

/// 剥离 keyframes 首尾空步骤的主入口。
///
/// 对节点列表中每个 keyframes 规则，移除其首尾的 `0%`/`from` 和 `100%`/`to` 空步骤。
/// 空步骤定义为：`declarations` 为空且 `children` 为空。
///
/// 原地修改 `nodes`，不创建新容器。
pub fn strip_empty_keyframe_steps(nodes: &mut Vec<CssNode>) {
    nodes.iter_mut().for_each(|node| {
        if let CssNode::AtRule {
            name,
            children,
            has_body: true,
            ..
        } = node
        {
            if CssAtRule::is_keyframes(name) {
                strip_empty_boundary_steps(children);
            }
        }
    });
}

/// 从步骤列表中移除首尾空步骤。
/// 前向扫描移除开头的空 0%/from 步骤，后向扫描移除结尾的 100%/to 步骤。
fn strip_empty_boundary_steps(steps: &mut Vec<CssNode>) {
    while let Some(first) = steps.first() {
        if is_empty_start_step(first) {
            steps.remove(0);
        } else {
            break;
        }
    }
    while let Some(last) = steps.last() {
        if is_empty_end_step(last) {
            steps.pop();
        } else {
            break;
        }
    }
}

/// 判断一个节点是否为空的起始步骤（`0% {}` 或 `from {}`）。
/// 支持多选择器步骤（如 `0%, from {}`），任一选择器匹配即视为起始。
fn is_empty_start_step(node: &CssNode) -> bool {
    match node {
        CssNode::Rule {
            selector,
            declarations,
            children,
        } if declarations.is_empty() && children.is_empty() => {
            split_selectors(selector)
                .iter()
                .any(|s| *s == "0%" || *s == "from")
        }
        _ => false,
    }
}

/// 判断一个节点是否为空的末尾步骤（`100% {}` 或 `to {}`）。
/// 支持多选择器步骤（如 `100%, to {}`），任一选择器匹配即视为末尾。
fn is_empty_end_step(node: &CssNode) -> bool {
    match node {
        CssNode::Rule {
            selector,
            declarations,
            children,
        } if declarations.is_empty() && children.is_empty() => {
            split_selectors(selector)
                .iter()
                .any(|s| *s == "100%" || *s == "to")
        }
        _ => false,
    }
}

/// 将逗号分隔的选择器字符串拆分为独立选择器。
/// 去除空白并过滤空段。
fn split_selectors(selector: &str) -> Vec<&str> {
    selector
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule_step(selector: &str, decls: Vec<CssNode>) -> CssNode {
        CssNode::Rule {
            selector: selector.to_string(),
            declarations: decls,
            children: vec![],
        }
    }

    fn declaration() -> CssNode {
        CssNode::Declaration {
            property: "opacity".into(),
            value: "0".into(),
            important: false,
        }
    }

    #[test]
    fn empty_from_at_start() {
        let mut steps = vec![
            rule_step("from", vec![]),
            rule_step("$%", vec![declaration()]),
        ];
        strip_empty_boundary_steps(&mut steps);
        assert_eq!(steps.len(), 1);
    }

    #[test]
    fn empty_100pct_at_end() {
        let mut steps = vec![
            rule_step("0%", vec![declaration()]),
            rule_step("100%", vec![]),
        ];
        strip_empty_boundary_steps(&mut steps);
        assert_eq!(steps.len(), 1);
    }

    #[test]
    fn middle_empty_preserved() {
        let mut steps = vec![
            rule_step("0%", vec![declaration()]),
            rule_step("50%", vec![]),
            rule_step("100%", vec![declaration()]),
        ];
        strip_empty_boundary_steps(&mut steps);
        assert_eq!(steps.len(), 3);
    }
}
