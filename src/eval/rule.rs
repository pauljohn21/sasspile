use super::*;
use crate::css::node::CssNode;
use crate::error::Result;

impl Evaluator {
    /// 求值规则——按顺序穿插输出声明组和嵌套规则。
    pub(crate) fn eval_rule(
        selector: &str,
        body: &[Node],
        env: &Env,
    ) -> Result<(Vec<CssNode>, Env)> {
        let span = crate::__tracing::info_span!("eval_rule", selector = selector);
        let _enter = span.enter();
        // 对选择器中的 #{...} 插值求值
        let selector = if selector.contains("#{") {
            crate::eval::value::eval_interp_str(selector, env)
        } else {
            selector.to_string()
        };
        // 进入新作用域：规则体内的变量赋值不影响外层
        let scoped_env = env.enter_scope().with_selector(selector.clone());
        let (css, new_env) = Self::eval_nodes(body, &scoped_env)?;
        // 离开作用域：移除规则体内定义的局部变量
        let restored_env = new_env.leave_scope();

        // plain CSS 模式——不合并选择器，保留嵌套结构
        if env.plain_css {
            let mut declarations = Vec::new();
            let mut children = Vec::new();
            let mut root_nodes = Vec::new();
            for node in css {
                match &node {
                    CssNode::Declaration { .. } => declarations.push(node.clone()),
                    CssNode::AtRoot(nodes) => root_nodes.extend(nodes.clone()),
                    CssNode::AtRule { name, .. }
                        if matches!(name.as_str(), "media" | "supports" | "container") =>
                    {
                        root_nodes.push(node.clone())
                    }
                    other => children.push(other.clone()),
                }
            }
            let mut result = Vec::new();
            if !declarations.is_empty() || !children.is_empty() {
                result.push(CssNode::Rule {
                    selector: selector.clone(),
                    declarations,
                    children,
                });
            }
            result.extend(root_nodes);
            return Ok((result, new_env));
        }

        let mut result = Vec::new();
        let mut current_decls = Vec::new();
        let mut root_nodes = Vec::new();

        for node in css {
            match node {
                CssNode::Declaration { .. } => current_decls.push(node),
                CssNode::AtRoot(nodes) => root_nodes.extend(nodes),
                CssNode::Rule {
                    selector: child_sel,
                    declarations: child_decls,
                    children: child_kids,
                } => {
                    // 遇到嵌套规则——先刷新当前声明组
                    if !current_decls.is_empty() {
                        result.push(CssNode::Rule {
                            selector: selector.clone(),
                            declarations: std::mem::take(&mut current_decls),
                            children: vec![],
                        });
                    }
                    // 合并选择器并输出嵌套规则
                    let combined = Self::combine_selectors(&selector, &child_sel);
                    if !child_decls.is_empty() {
                        result.push(CssNode::Rule {
                            selector: combined.clone(),
                            declarations: child_decls,
                            children: vec![],
                        });
                    }
                    // 递归展平子规则的子规则
                    for kid in child_kids {
                        if let CssNode::Rule {
                            selector: kid_sel,
                            declarations: kid_decls,
                            ..
                        } = kid
                        {
                            let kid_combined = Self::combine_selectors(&combined, &kid_sel);
                            if !kid_decls.is_empty() {
                                result.push(CssNode::Rule {
                                    selector: kid_combined,
                                    declarations: kid_decls,
                                    children: vec![],
                                });
                            }
                        } else {
                            result.push(kid);
                        }
                    }
                }
                other => {
                    // 其他节点（AtRule 等）——先刷新当前声明组
                    if !current_decls.is_empty() {
                        result.push(CssNode::Rule {
                            selector: selector.clone(),
                            declarations: std::mem::take(&mut current_decls),
                            children: vec![],
                        });
                    }
                    result.push(other);
                }
            }
        }

        // 输出最后的声明组
        if !current_decls.is_empty() {
            result.push(CssNode::Rule {
                selector: selector.clone(),
                declarations: current_decls,
                children: vec![],
            });
        }

        // 如果没有任何输出，保留空规则
        if result.is_empty() && root_nodes.is_empty() {
            result.push(CssNode::Rule {
                selector,
                declarations: vec![],
                children: vec![],
            });
        }

        // 添加 @at-root 节点
        result.extend(root_nodes);
        Ok((result, restored_env))
    }

    /// 组合选择器——处理 & 替换和逗号分隔选择器。
    pub(crate) fn combine_selectors(parent: &str, child: &str) -> String {
        let parents: Vec<&str> = parent
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        let children: Vec<&str> = child
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        let mut result = Vec::new();
        for p in &parents {
            for c in &children {
                if c.contains('&') {
                    result.push(c.replace('&', p));
                } else if p.is_empty() {
                    result.push(c.to_string());
                } else {
                    result.push(format!("{p} {c}"));
                }
            }
        }
        result.join(", ")
    }
}
