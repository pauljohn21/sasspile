//! 规则求值。

use crate::error::Result;
use crate::parse::Node;
use crate::css::CssNode;
use super::env::Env;
use super::eval_nodes;

/// 求值规则——selector + body。
pub fn eval_rule(selector: &str, body: &[Node], env: Env) -> Result<(Option<Vec<CssNode>>, Env)> {
    // 解析选择器中的插值/变量
    let resolved_selector = resolve_selector(selector, &env);

    let child_env = env.enter_scope().with_selector(resolved_selector.clone());
    let children = eval_nodes(body, child_env)?.0;

    // 分离 declarations 和子规则
    let mut declarations = Vec::new();
    let mut nested = Vec::new();
    let mut root_nodes = Vec::new();
    for child in children {
        match child {
            CssNode::Declaration { .. } | CssNode::Comment(_) => declarations.push(child),
            CssNode::AtRoot(nodes) => root_nodes.extend(nodes),
            _ => nested.push(child),
        }
    }

    // 嵌套规则选择器合并：把 parent selector 传播到子规则
    let nested = nest_rule_in_children(nested, &resolved_selector);

    let rule = CssNode::Rule {
        selector: resolved_selector,
        declarations,
        children: nested,
    };

    // AtRoot 节点提升到顶层
    let mut result = vec![rule];
    result.extend(root_nodes);

    Ok((Some(result), env))
}

/// 把父选择器传播到子规则——实现 SCSS 嵌套。
fn nest_rule_in_children(children: Vec<CssNode>, parent: &str) -> Vec<CssNode> {
    let mut result = Vec::new();
    for child in children {
        match child {
            CssNode::Rule { selector, declarations, children } => {
                // 处理选择器列表：每个逗号分隔的选择器分别嵌套
                let merged = if selector.contains('&') {
                    selector.replace('&', parent)
                } else {
                    // 处理逗号分隔的父选择器和子选择器
                    let parent_parts: Vec<&str> = parent.split(',').map(|s| s.trim()).collect();
                    let child_parts: Vec<&str> = selector.split(',').map(|s| s.trim()).collect();
                    if parent_parts.len() == 1 {
                        format!("{parent} {selector}")
                    } else {
                        // 笛卡尔积嵌套
                        let combinations: Vec<String> = child_parts.iter().flat_map(|child| {
                            parent_parts.iter().map(move |p| format!("{p} {child}"))
                        }).collect();
                        combinations.join(", ")
                    }
                };
                let children = nest_rule_in_children(children, &merged);
                result.push(CssNode::Rule {
                    selector: merged,
                    declarations,
                    children,
                });
            }
            CssNode::AtRule { name, params, children, has_body } => {
                // @keyframes 不传播父选择器
                let processed_children = if matches!(name.as_str(), "keyframes" | "-webkit-keyframes" | "-moz-keyframes") {
                    children
                } else {
                    nest_rule_in_children(children, parent)
                };
                result.push(CssNode::AtRule { name, params, children: processed_children, has_body });
            }
            other => result.push(other),
        }
    }
    result
}

/// 解析选择器——替换 $var 和 & 父选择器。
fn resolve_selector(selector: &str, env: &Env) -> String {
    let mut result = selector.to_string();
    // 替换 $var
    for (name, value) in &env.local_vars {
        let placeholder = format!("${name}");
        result = result.replace(&placeholder, &value.to_css_string());
    }
    result = result.trim().to_string();
    result
}
