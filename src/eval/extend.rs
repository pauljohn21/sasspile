use super::*;
use crate::css::node::CssNode;
use crate::eval::selector::{ComplexSelector, CompoundWithCombinator, SelectorList};

/// Extend 操作类型。
enum ExtendAction {
    /// 替换选择器中的某个 complex selector（用于占位符）。
    Replace(ComplexSelector),
    /// 在当前 selector 列表中添加 extender（用于普通选择器）。
    Append(SelectorList),
}

impl Evaluator {
    /// 应用 @extend —— 结构化版本。
    ///
    /// 对于每个 (extender, target) 对，在节点的选择器列表中查找 target 的结构匹配。
    /// 匹配成功时，将 extender 的复杂选择器添加到该节点的选择器列表中。
    pub(crate) fn apply_extends(nodes: &mut [CssNode], extends: &[(SelectorList, SelectorList)]) {
        let span = crate::__tracing::info_span!("apply_extends", n_extends = extends.len());
        let _enter = span.enter();
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

                    // 对每个 extend 关系应用 structural matching
                    for (extender_list, target_list) in extends {
                        Self::apply_extend_to_selector(selector, extender_list, target_list);
                    }

                    // 递归处理子规则
                    Self::apply_extends(children, extends);

                    // 移除未被继承的占位符选择器部分
                    Self::remove_unextended_placeholders(selector);
                }
                CssNode::AtRule {
                    has_body: true,
                    children,
                    ..
                } => {
                    Self::apply_extends(children, extends);
                }
                CssNode::AtRoot(kids) => {
                    Self::apply_extends(kids, extends);
                }
                _ => {}
            }
        }
    }

    /// 将单个 extend 关系应用到选择器列表。
    fn apply_extend_to_selector(
        selector: &mut SelectorList,
        extender_list: &SelectorList,
        target_list: &SelectorList,
    ) {
        // 对每个 target，检查是否在 selector 中有结构匹配
        for target in target_list.iter() {
            if let Some(matched) = Self::find_and_extend(selector, target, extender_list) {
                crate::__tracing::info!(
                    target: "sasspile::extend",
                    extender = %extender_list,
                    target_sel = %target,
                    matched = %matched,
                    "extend matched (structural)"
                );
            }
        }
    }

    /// 在选择器列表中查找 target 并应用 extender。
    /// 返回 Some(匹配到的选择器字符串) 如果成功匹配。
    fn find_and_extend(
        selector: &mut SelectorList,
        target: &ComplexSelector,
        extender_list: &SelectorList,
    ) -> Option<String> {
        // 先找到匹配的索引
        let matched_idx = selector.iter().position(|sel_complex| {
            Self::try_match_and_extend(sel_complex, target, extender_list).is_some()
        })?;

        // 获取匹配的 complex selector 的字符串表示
        let matched_str = selector.0[matched_idx].to_string();

        // 根据操作类型执行
        match Self::try_match_and_extend(&selector.0[matched_idx], target, extender_list)? {
            ExtendAction::Replace(replaced) => {
                // 占位符替换：直接替换匹配的 complex selector
                let mut new_parts = selector.0.clone();
                new_parts[matched_idx] = replaced;
                *selector = SelectorList(new_parts);
            }
            ExtendAction::Append(extender_to_add) => {
                // 普通选择器：在 selector 列表中添加 extender 的 complex selectors
                let mut new_parts = selector.0.clone();
                for ext in extender_to_add.iter() {
                    if !new_parts.contains(ext) {
                        new_parts.push(ext.clone());
                    }
                }
                *selector = SelectorList(new_parts);
            }
        }

        Some(matched_str)
    }

    /// 尝试将 target 匹配到 sel_complex 的某个部分：
    /// - 占位符：返回替换后的 complex selector
    /// - 普通选择器：返回 Some(原始 complex selector) 表示需要添加 extender
    fn try_match_and_extend(
        sel_complex: &ComplexSelector,
        target: &ComplexSelector,
        extender_list: &SelectorList,
    ) -> Option<ExtendAction> {
        let is_placeholder = Self::is_placeholder(target);

        if is_placeholder {
            // 占位符：返回替换后的 complex selector
            Self::replace_placeholder(sel_complex, target, extender_list)
                .map(ExtendAction::Replace)
        } else if Self::extend_normal_match(sel_complex, target) {
            // 普通选择器：需要在 selector 列表级别添加 extender
            Some(ExtendAction::Append(extender_list.clone()))
        } else {
            None
        }
    }

    /// 判断 target 是否是占位符选择器。
    fn is_placeholder(target: &ComplexSelector) -> bool {
        target.parts.iter().any(|part| {
            part.compound
                .element
                .as_ref()
                .map_or(false, |e| e.starts_with('%'))
        })
    }

    /// 替换占位符选择器。
    ///
    /// 将 sel_complex 中匹配 placeholder 的 compound 替换为 extender 的选择器。
    fn replace_placeholder(
        sel_complex: &ComplexSelector,
        target: &ComplexSelector,
        extender_list: &SelectorList,
    ) -> Option<ComplexSelector> {
        // 查找 target 中占位符 compound
        let _target_placeholder = target
            .parts
            .iter()
            .find(|p| p.compound.element.as_ref().map_or(false, |e| e.starts_with('%')))?
            .clone();

        // 在 sel_complex 中查找与占位符匹配的 compound
        for (sel_idx, sel_part) in sel_complex.parts.iter().enumerate() {
            let is_placeholder = sel_part
                .compound
                .element
                .as_ref()
                .map_or(false, |e| e.starts_with('%'));

            if is_placeholder {
                // 构建替换后的 complex selector
                let mut new_parts = Vec::new();

                // 添加前缀
                new_parts.extend_from_slice(&sel_complex.parts[..sel_idx]);

                // 使用 extender 的选择器替换占位符
                // 简化处理：使用第一个 extender 复杂选择器
                if let Some(ext) = extender_list.iter().next() {
                    for (ext_idx, ext_part) in ext.parts.iter().enumerate() {
                        let combinator = if ext_idx == 0 {
                            sel_part.combinator.clone()
                        } else {
                            ext_part.combinator.clone()
                        };
                        new_parts.push(CompoundWithCombinator {
                            compound: ext_part.compound.clone(),
                            combinator,
                        });
                    }
                }

                // 添加后缀
                new_parts.extend_from_slice(&sel_complex.parts[sel_idx + 1..]);

                return Some(ComplexSelector { parts: new_parts });
            }
        }
        None
    }

    /// 普通选择器扩展：检查 target 是否匹配 sel_complex 的一部分。
    /// 如果匹配，返回 Some(()) 表示需要在 selector 列表级别添加 extender。
    fn extend_normal_match(
        sel_complex: &ComplexSelector,
        target: &ComplexSelector,
    ) -> bool {
        if target.parts.is_empty() {
            return false;
        }

        // 检查 target 是否是 sel_complex 的后缀匹配
        if target.parts.len() > sel_complex.parts.len() {
            return false;
        }

        let offset = sel_complex.parts.len() - target.parts.len();
        let mut matched = true;

        for (i, target_part) in target.parts.iter().enumerate() {
            let sel_part = &sel_complex.parts[offset + i];

            // 检查 combinator 匹配（第一个 target part 的 combinator 不检查）
            if i > 0 && sel_part.combinator != target_part.combinator {
                matched = false;
                break;
            }

            // 检查 compound 匹配（sel_part 必须包含 target_part 的所有元素）
            if !Self::compound_includes(&sel_part.compound, &target_part.compound) {
                matched = false;
                break;
            }
        }

        matched
    }

    /// 检查 compound 是否匹配占位符。
    fn compound_matches_placeholder(
        compound: &crate::eval::selector::CompoundSelector,
        placeholder: &crate::eval::selector::CompoundSelector,
    ) -> bool {
        // 占位符匹配：placeholder 的所有非空字段都必须在 compound 中存在
        if let (Some(placeholder_elem), Some(compound_elem)) =
            (&placeholder.element, &compound.element)
        {
            if placeholder_elem != compound_elem {
                return false;
            }
        }

        // 检查 classes 包含关系
        for class in &placeholder.classes {
            if !compound.classes.contains(class) {
                return false;
            }
        }

        // 检查 ids 包含关系
        for id in &placeholder.ids {
            if !compound.ids.contains(id) {
                return false;
            }
        }

        true
    }

    /// 检查 compound 是否包含另一个 compound 的所有选择器组件。
    fn compound_includes(
        compound: &crate::eval::selector::CompoundSelector,
        sub: &crate::eval::selector::CompoundSelector,
    ) -> bool {
        // element 检查
        match (&sub.element, &compound.element) {
            (Some(sub_elem), Some(compound_elem)) if sub_elem != compound_elem => return false,
            (Some(_), None) => return false,
            _ => {}
        }

        // namespace 检查
        match (&sub.namespace, &compound.namespace) {
            (Some(sub_ns), Some(compound_ns)) if sub_ns != compound_ns => return false,
            (Some(_), None) => return false,
            _ => {}
        }

        // classes 包含
        for class in &sub.classes {
            if !compound.classes.contains(class) {
                return false;
            }
        }

        // ids 包含
        for id in &sub.ids {
            if !compound.ids.contains(id) {
                return false;
            }
        }

        // attrs 包含
        for attr in &sub.attrs {
            if !compound.attrs.contains(attr) {
                return false;
            }
        }

        // pseudos 包含
        for pseudo in &sub.pseudos {
            if !compound.pseudos.contains(pseudo) {
                return false;
            }
        }

        true
    }

    /// 移除未被继承的占位符选择器部分。
    fn remove_unextended_placeholders(selector: &mut SelectorList) {
        let mut new_parts = Vec::new();

        for complex in selector.iter() {
            // 检查 complex 是否包含占位符
            let has_placeholder = complex.parts.iter().any(|p| {
                p.compound
                    .element
                    .as_ref()
                    .map_or(false, |e| e.starts_with('%'))
            });

            if !has_placeholder {
                new_parts.push(complex.clone());
            }
        }

        *selector = SelectorList(new_parts);
    }
}
