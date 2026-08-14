use super::*;
use crate::css::node::CssNode;
use crate::error::Result;
use crate::eval::selector::parse::{parse_selector_list, CompoundSelector, SelectorList};

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

        // 将选择器解析为结构化表示
        let selector_list = parse_selector_list(&selector).unwrap_or_default();

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
                    selector: selector_list.clone(),
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
                            selector: selector_list.clone(),
                            declarations: std::mem::take(&mut current_decls),
                            children: vec![],
                        });
                    }
                    // 合并选择器并输出嵌套规则
                    let combined = Self::combine_selectors(&selector_list, &child_sel);
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
                            selector: selector_list.clone(),
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
                selector: selector_list.clone(),
                declarations: current_decls,
                children: vec![],
            });
        }

        // 如果没有任何输出，保留空规则
        if result.is_empty() && root_nodes.is_empty() {
            result.push(CssNode::Rule {
                selector: selector_list,
                declarations: vec![],
                children: vec![],
            });
        }

        // 添加 @at-root 节点
        result.extend(root_nodes);
        Ok((result, restored_env))
    }

    /// 组合选择器——结构化版本。
    ///
    /// 处理 & 替换和逗号分隔选择器：
    /// - 如果 child 包含 & 元素，用 parent 的 compound 替换 &
    /// - 如果 parent 为空，使用 child 原样
    /// - 否则 parent + 后代组合器 + child
    pub(crate) fn combine_selectors(
        parent: &SelectorList,
        child: &SelectorList,
    ) -> SelectorList {
        let mut result = Vec::new();

        if parent.is_empty() {
            // 父选择器为空，直接返回子选择器
            return child.clone();
        }

        for parent_complex in parent.iter() {
            for child_complex in child.iter() {
                let combined = Self::combine_complex_selectors(parent_complex, child_complex);
                result.push(combined);
            }
        }

        SelectorList(result)
    }

    /// 组合两个 complex selector。
    fn combine_complex_selectors(
        parent: &selector::ComplexSelector,
        child: &selector::ComplexSelector,
    ) -> selector::ComplexSelector {
        // 检查 child 是否包含 & 元素
        let has_ampersand = child.parts.iter().any(|p| {
            p.compound
                .element
                .as_ref()
                .map_or(false, |e| e == "&")
        });

        if has_ampersand {
            // 替换 & 元素为 parent 的 parts
            Self::replace_ampersand(parent, child)
        } else {
            // parent + 后代组合器 + child
            let mut parts = parent.parts.clone();
            // 添加后代组合器（第一个 child part 使用 Descendant）
            for (i, child_part) in child.parts.iter().enumerate() {
                let combinator = if i == 0 {
                    Some(selector::Combinator::Descendant)
                } else {
                    child_part.combinator.clone()
                };
                parts.push(selector::CompoundWithCombinator {
                    compound: child_part.compound.clone(),
                    combinator,
                });
            }
            selector::ComplexSelector { parts }
        }
    }

    /// 将 child 中的 & 替换为 parent 的 parts。
    ///
    /// SCSS 的 & 代表父选择器。当 child 为 `&:hover` 且 parent 为 `.btn` 时，
    /// 结果是 `.btn:hover` - 即将 parent 的 compound 内容合并到 & 所在的 compound 中。
    fn replace_ampersand(
        parent: &selector::ComplexSelector,
        child: &selector::ComplexSelector,
    ) -> selector::ComplexSelector {
        let mut parts = Vec::new();

        for (i, child_part) in child.parts.iter().enumerate() {
            let is_ampersand = child_part
                .compound
                .element
                .as_ref()
                .map_or(false, |e| e == "&");

            if is_ampersand && !parent.parts.is_empty() {
                // & 是一个"占位符" compound：element=&, pseudos=[hover], 等等
                // 用 parent 的第一个 compound 的属性替换 & 的 element，
                // 但保留 child 的 pseudos/classes 等附加属性
                let parent_first = &parent.parts[0].compound;

                // 构建合并后的 compound：parent 的基础 + child 的附加属性
                let merged_compound = CompoundSelector {
                    namespace: parent_first.namespace.clone(),
                    element: parent_first.element.clone(),
                    classes: parent_first.classes.clone(),
                    ids: parent_first.ids.clone(),
                    attrs: parent_first.attrs.clone(),
                    pseudos: child_part.compound.pseudos.clone(),
                };

                parts.push(selector::CompoundWithCombinator {
                    compound: merged_compound,
                    combinator: child_part.combinator.clone(),
                });

                // 如果 parent 有多个 parts，添加剩余的 parts（带适当的组合器）
                for (j, parent_part) in parent.parts.iter().enumerate().skip(1) {
                    let combinator = parent_part.combinator.clone();
                    parts.push(selector::CompoundWithCombinator {
                        compound: parent_part.compound.clone(),
                        combinator,
                    });
                }
            } else {
                // 保留原始 child part（或当 parent 为空时的 &）
                parts.push(child_part.clone());
            }
        }

        selector::ComplexSelector { parts }
    }
}
