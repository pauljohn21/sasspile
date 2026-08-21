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
        let (css, new_env) = Self::eval_nodes(body, &env.with_selector(selector.clone()))?;

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
    // 规则体创建局部作用域——局部变量绑定不传播到外层（sass-spec 作用域规则）。
    // 但以下变更需要传播到外层：
    // - 命名空间变量赋值（midstream.$var 写入模块变量，非局部）
    // - !global 变量赋值（写入全局作用域）
    // - mixin/function 定义（Sass 语义允许规则体内定义的 mixin 可见）
    // - @extend 关系（规则体内的 @extend 需要应用到全局）
    // - 命名空间模块（规则体内的 @use 需要可见）
    // - 内建模块注册
    let mut return_env = env.clone();
    // 传播命名空间变量赋值（名字含 . 的）
    for (name, val) in &new_env.local_vars {
        if name.contains('.') {
            return_env = return_env.bind(name.clone(), val.clone());
        }
    }
    // 传播 !global 变量赋值
    for (name, val) in &new_env.global_writes {
        return_env = return_env.bind(name.clone(), val.clone());
    }
    for (name, def) in &new_env.local_mixins {
        if !env.local_mixins.contains_key(name) {
            return_env = return_env.define_local_mixin(name.clone(), def.clone());
        }
    }
    for (name, def) in &new_env.local_functions {
        if !env.local_functions.contains_key(name) {
            return_env = return_env.define_local_function(name.clone(), def.clone());
        }
    }
    // 传播 forwarded 成员
    for (name, def) in &new_env.forwarded_mixins {
        if !env.forwarded_mixins.contains_key(name) {
            return_env = return_env.define_forwarded_mixin(name.clone(), def.clone());
        }
    }
    for (name, def) in &new_env.forwarded_functions {
        if !env.forwarded_functions.contains_key(name) {
            return_env = return_env.define_forwarded_function(name.clone(), def.clone());
        }
    }
    for (name, val) in &new_env.forwarded_vars {
        if !env.forwarded_vars.contains_key(name) {
            return_env.forwarded_vars.insert(name.clone(), val.clone());
        }
    }
    return_env.extends = new_env.extends.clone();
    return_env.namespaces = new_env.namespaces.clone();
    return_env.builtin_modules = new_env.builtin_modules.clone();
    Ok((result, return_env))
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
