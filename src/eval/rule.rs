use super::*;
use crate::css::node::CssNode;
use crate::error::Result;
use std::mem;

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

        // 保存 local_vars（规则体局部作用域不传播）
        let saved_local_vars = env.local_vars.clone();
        let saved_local_mixins = env.local_mixins.clone();
        let saved_local_functions = env.local_functions.clone();
        let saved_forwarded_vars = env.forwarded_vars.clone();
        let saved_forwarded_mixins = env.forwarded_mixins.clone();
        let saved_forwarded_functions = env.forwarded_functions.clone();

        let (css, new_env) = Self::eval_nodes(body, env.with_selector(selector.clone()))?;

        // plain CSS 模式——不合并选择器，保留嵌套结构
        if new_env.plain_css {
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
        let mut return_env = new_env;

        // 恢复 local_vars：保留命名空间变量（含 .）和 global_writes 传播
        let rule_local_vars = mem::take(&mut return_env.local_vars);
        let rule_global_writes = mem::take(&mut return_env.global_writes);
        let rule_local_mixins = mem::take(&mut return_env.local_mixins);
        let rule_local_functions = mem::take(&mut return_env.local_functions);
        let rule_forwarded_mixins = mem::take(&mut return_env.forwarded_mixins);
        let rule_forwarded_functions = mem::take(&mut return_env.forwarded_functions);
        let rule_forwarded_vars = mem::take(&mut return_env.forwarded_vars);

        // 恢复 saved 的 local 表
        return_env.local_vars = saved_local_vars;
        return_env.local_mixins = saved_local_mixins;
        return_env.local_functions = saved_local_functions;
        return_env.forwarded_vars = saved_forwarded_vars;
        return_env.forwarded_mixins = saved_forwarded_mixins;
        return_env.forwarded_functions = saved_forwarded_functions;

        // 传播命名空间变量赋值（名字含 . 的）
        for (name, val) in &rule_local_vars {
            if name.contains('.') {
                return_env.local_vars.insert(name.clone(), val.clone());
            }
        }
        // 传播 !global 变量赋值
        for (name, val) in &rule_global_writes {
            return_env.local_vars.insert(name.clone(), val.clone());
        }
        // 传播新增 mixin/function（规则体内定义的）
        for (name, def) in &rule_local_mixins {
            return_env.local_mixins.entry(name.clone()).or_insert_with(|| def.clone());
        }
        for (name, def) in &rule_local_functions {
            return_env.local_functions.entry(name.clone()).or_insert_with(|| def.clone());
        }
        // 传播新增 forwarded 成员
        for (name, def) in &rule_forwarded_mixins {
            return_env.forwarded_mixins.entry(name.clone()).or_insert_with(|| def.clone());
        }
        for (name, def) in &rule_forwarded_functions {
            return_env.forwarded_functions.entry(name.clone()).or_insert_with(|| def.clone());
        }
        for (name, val) in &rule_forwarded_vars {
            return_env.forwarded_vars.entry(name.clone()).or_insert_with(|| val.clone());
        }

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
}
