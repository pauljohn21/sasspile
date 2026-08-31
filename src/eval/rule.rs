use super::*;
use crate::css::node::CssNode;
use crate::error::Result;

impl Evaluator {
    /// 求值规则——按顺序穿插输出声明组和嵌套规则。
    pub(crate) fn eval_rule(
        selector: &str,
        body: &[Node],
        env: Env,
    ) -> Result<(Vec<CssNode>, Env)> {
        let span = crate::__tracing::info_span!("eval_rule", selector = selector);
        let _enter = span.enter();
        // 对选择器中的 #{...} 插值求值
        let selector = if selector.contains("#{") {
            crate::eval::value::eval_interp_str(selector, &env)
        } else {
            selector.to_string()
        };

        // 保存 local 表（规则体局部作用域不传播）
        let saved_local_vars = env.get_local_vars().clone();
        let saved_local_mixins = env.get_local_mixins().clone();
        let saved_local_functions = env.get_local_functions().clone();
        let saved_forwarded_vars = env.get_forwarded_vars().clone();
        let saved_forwarded_mixins = env.get_forwarded_mixins().clone();
        let saved_forwarded_functions = env.get_forwarded_functions().clone();

        let (css, new_env) = Self::eval_nodes(body, env.with_selector(selector.clone()))?;

        // plain CSS 模式——不合并选择器，保留嵌套结构
        if new_env.is_plain_css() {
            let mut declarations = Vec::new();
            let mut children = Vec::new();
            let mut root_nodes = Vec::new();
            for node in css {
                match &node {
                    CssNode::Declaration { .. } => declarations.push(node.clone()),
                    CssNode::AtRoot(nodes) => root_nodes.extend(nodes.clone()),
                    CssNode::AtRule { name, .. } if matches!(name.as_str(), "media" | "supports" | "container") => root_nodes.push(node.clone()),
                    other => children.push(other.clone()),
                }
            }
            let mut result = Vec::new();
            if !declarations.is_empty() || !children.is_empty() {
                result.push(CssNode::Rule { selector: selector.clone(), declarations, children });
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
                CssNode::Rule { selector: child_sel, declarations: child_decls, children: child_kids } => {
                    if !current_decls.is_empty() {
                        result.push(CssNode::Rule { selector: selector.clone(), declarations: std::mem::take(&mut current_decls), children: vec![] });
                    }
                    let combined = Self::combine_selectors(&selector, &child_sel);
                    if !child_decls.is_empty() {
                        result.push(CssNode::Rule { selector: combined.clone(), declarations: child_decls, children: vec![] });
                    }
                    for kid in child_kids {
                        if let CssNode::Rule { selector: kid_sel, declarations: kid_decls, .. } = kid {
                            let kid_combined = Self::combine_selectors(&combined, &kid_sel);
                            if !kid_decls.is_empty() {
                                result.push(CssNode::Rule { selector: kid_combined, declarations: kid_decls, children: vec![] });
                            }
                        } else {
                            result.push(kid);
                        }
                    }
                }
                other => {
                    if !current_decls.is_empty() {
                        result.push(CssNode::Rule { selector: selector.clone(), declarations: std::mem::take(&mut current_decls), children: vec![] });
                    }
                    // AtRule 嵌套处理：将父选择器传播到 AtRule 内的 Rule 子节点
                    let other = match other {
                        CssNode::AtRule { name, params, children, has_body: true } => {
                            let n = name.clone();
                            let p = params.clone();
                            let ch = if n == "keyframes" || n == "-webkit-keyframes" || n == "-moz-keyframes" {
                                // @keyframes 不传播父选择器
                                children
                            } else {
                                // 其他 AtRule：将父选择器传播到内部 Rule
                                Self::nest_rule_in_children(&selector, children)
                            };
                            CssNode::AtRule { name: n, params: p, children: ch, has_body: true }
                        }
                        CssNode::AtRule { name, params, children: _, has_body: false } => {
                            // 无 body 的 AtRule（如 @b c;）包裹在父选择器下
                            CssNode::Rule { selector: selector.clone(), declarations: vec![], children: vec![CssNode::AtRule { name, params, children: vec![], has_body: false }] }
                        }
                        other => other,
                    };
                    result.push(other);
                }
            }
        }
        if !current_decls.is_empty() {
            result.push(CssNode::Rule { selector: selector.clone(), declarations: current_decls, children: vec![] });
        }
        if result.is_empty() && root_nodes.is_empty() {
            result.push(CssNode::Rule { selector, declarations: vec![], children: vec![] });
        }
        result.extend(root_nodes);

        // 作用域传播：从 new_env 提取需要传播的字段，合并回 saved 状态
        // 使用 exit_scope 方法替代手动 save/restore
        let return_env = new_env.exit_scope(saved_local_vars, saved_local_mixins, saved_local_functions, saved_forwarded_vars, saved_forwarded_mixins, saved_forwarded_functions);

        Ok((result, return_env))
    }

    /// 组合选择器——处理 & 替换和逗号分隔选择器。
    pub(crate) fn combine_selectors(parent: &str, child: &str) -> String {
        let parents: Vec<&str> = parent.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        let children: Vec<&str> = child.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
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

    /// 将父选择器传播到 AtRule children 内的 Rule 子节点。
    ///
    /// 用于 `a {@import "other"}` 场景——被导入文件中的规则需要嵌套在父选择器 `a` 下。
    fn nest_rule_in_children(parent: &str, children: Vec<CssNode>) -> Vec<CssNode> {
        let mut result = Vec::new();
        let mut current_decls = Vec::new();
        for child in children {
            match child {
                CssNode::Declaration { .. } => current_decls.push(child),
                CssNode::Rule { selector, declarations, children } => {
                    if !current_decls.is_empty() {
                        result.push(CssNode::Rule {
                            selector: parent.to_string(),
                            declarations: std::mem::take(&mut current_decls),
                            children: vec![],
                        });
                    }
                    let combined = Self::combine_selectors(parent, &selector);
                    result.push(CssNode::Rule { selector: combined, declarations, children });
                }
                CssNode::AtRule { name, params, children, has_body: true } => {
                    use crate::parse::at_rule_kinds::CssAtRule;
                    let ch = if CssAtRule::is_keyframes(&name) {
                        children
                    } else {
                        Self::nest_rule_in_children(parent, children)
                    };
                    result.push(CssNode::AtRule { name, params, children: ch, has_body: true });
                }
                other => {
                    if !current_decls.is_empty() {
                        result.push(CssNode::Rule {
                            selector: parent.to_string(),
                            declarations: std::mem::take(&mut current_decls),
                            children: vec![],
                        });
                    }
                    result.push(other);
                }
            }
        }
        if !current_decls.is_empty() {
            result.push(CssNode::Rule {
                selector: parent.to_string(),
                declarations: current_decls,
                children: vec![],
            });
        }
        result
    }
}
