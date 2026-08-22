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
    let children = eval_nodes(body, child_env)?;

    // 分离 declarations 和子规则
    let mut declarations = Vec::new();
    let mut nested = Vec::new();
    for child in children {
        match child {
            CssNode::Declaration { .. } | CssNode::Comment(_) => declarations.push(child),
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

    Ok((Some(vec![rule]), env))
}

/// 把父选择器传播到子规则——实现 SCSS 嵌套。
fn nest_rule_in_children(children: Vec<CssNode>, parent: &str) -> Vec<CssNode> {
    let mut result = Vec::new();
    for child in children {
        match child {
            CssNode::Rule { selector, declarations, children } => {
                // 合并选择器：parent + " " + child（如果 child 没用 &）
                let merged = if selector.contains('&') {
                    selector.replace('&', parent)
                } else {
                    format!("{parent} {selector}")
                };
                let children = nest_rule_in_children(children, &merged);
                result.push(CssNode::Rule {
                    selector: merged,
                    declarations,
                    children,
                });
            }
            CssNode::AtRule { name, params, children, has_body } => {
                // @at-root 不传播父选择器
                result.push(CssNode::AtRule { name, params, children, has_body });
            }
            other => result.push(other),
        }
    }
    result
}

/// 解析选择器——替换 $var 和 & 父选择器。
fn resolve_selector(selector: &str, env: &Env) -> String {
    let mut result = selector.to_string();
    // TODO: 处理 #{...} 插值
    for (name, value) in &env.local_vars {
        let placeholder = format!("${name}");
        result = result.replace(&placeholder, &value.to_css_string());
    }
    result = result.trim().to_string();
    result
}
