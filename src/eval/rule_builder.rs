//! —— 规则构建器 ——
//!
//! 封装 `eval_rule` 的累积器状态，处理 CSS 节点的分发、选择器组合和 AtRoot 提升。

use super::*;
use crate::css::node::CssNode;

use super::selector_combine::{
    combine_selectors, has_descendant_prefix, nest_rule_in_children,
};

/// 规则构建器——封装 `eval_rule` 的累积器状态。
///
/// `result` 是最终输出节点列表，`current_decls` 是当前累积的声明，
/// `root_nodes` 是 @at-root 提升的节点。
/// `resolved_self` 是 selector 展开 `&` 后的值，用于 compound 前缀检测。
pub(super) struct RuleBuilder {
    selector: String,
    resolved_self: String,
    result: Vec<CssNode>,
    current_decls: Vec<CssNode>,
    root_nodes: Vec<CssNode>,
}

impl RuleBuilder {
    pub(super) fn new(selector: String) -> Self {
        Self {
            resolved_self: selector.clone(),
            selector,
            result: Vec::new(),
            current_decls: Vec::new(),
            root_nodes: Vec::new(),
        }
    }

    /// 设置 resolved_self（展开 `&` 后的选择器）。
    pub(super) fn set_resolved_self(&mut self, resolved: String) {
        self.resolved_self = resolved;
    }

    /// flush 当前累积的声明为一条 Rule 节点。
    fn flush_decls(&mut self) {
        if !self.current_decls.is_empty() {
            let decls = std::mem::take(&mut self.current_decls);
            self.result.push(CssNode::Rule {
                selector: self.selector.clone(),
                declarations: decls,
                children: vec![],
            });
        }
    }

    /// push 一个 CSS 节点到构建器。
    #[tracing::instrument(skip(self, node), fields(sel = %self.selector, node = ?std::mem::discriminant(&node)))]
    pub(super) fn push(&mut self, node: CssNode) {
        match node {
            CssNode::Declaration { .. } => {
                self.current_decls.push(node);
            }
            CssNode::AtRoot(nodes, query) => {
                // 解析 @at-root query 语义（官方文档）
                // without: media/supports → 脱离 @media 但保留父选择器
                // without: rule → 脱离父选择器（默认行为）
                // without: all → 脱离所有包裹
                // with: rule → 只保留 style rules，排除所有 at-rules
                // 无 query → 脱离父选择器（默认行为）
                let without_media = query
                    .as_ref()
                    .is_some_and(|q| q.contains("without: media") || q.contains("without:media"));
                let without_supports = query.as_ref().is_some_and(|q| {
                    q.contains("without: supports") || q.contains("without:supports")
                });
                let without_all = query
                    .as_ref()
                    .is_some_and(|q| q.contains("without: all") || q.contains("without:all"));
                let with_rule = query
                    .as_ref()
                    .is_some_and(|q| q.contains("with: rule") || q.contains("with:rule"));
                // `&` 父选择器检测：即使无 query，含 `&` 的规则也需父选择器展开
                let has_parent_ref = nodes.iter().any(Evaluator::selector_contains_ampersand);
                if without_media || without_supports || without_all || with_rule || has_parent_ref {
                    let nested = nest_rule_in_children(&self.selector, nodes);
                    self.flush_decls();
                    self.result.push(CssNode::AtRoot(nested, query));
                } else {
                    self.root_nodes.extend(nodes);
                }
            }
            // AtRootDirect：来自 mixin at-rule，selector 可能含字面 &（如 when() mixin 的 &.disabled）。
            CssNode::AtRootDirect(inner) => {
                self.flush_decls();
                self.push_atroot_direct(*inner);
            }
            CssNode::Rule {
                selector: child_sel,
                declarations: child_decls,
                children: child_kids,
            } => {
                self.flush_decls();
                // 用 resolved_self（& 已展开为完整 compound）检测 inner eval 输出
                // 是否已包含 self 语义前缀。
                let has_prefix = has_descendant_prefix(&self.resolved_self, &child_sel);
                let combined = if has_prefix {
                    child_sel.clone()
                } else {
                    combine_selectors(&self.selector, &child_sel)
                };
                if !child_decls.is_empty() {
                    self.result.push(CssNode::Rule {
                        selector: combined.clone(),
                        declarations: child_decls,
                        children: vec![],
                    });
                }
                for kid in child_kids {
                    if let CssNode::Rule {
                        selector: kid_sel,
                        declarations: kid_decls,
                        ..
                    } = kid
                    {
                        let kid_combined = combine_selectors(&combined, &kid_sel);
                        if !kid_decls.is_empty() {
                            self.result.push(CssNode::Rule {
                                selector: kid_combined,
                                declarations: kid_decls,
                                children: vec![],
                            });
                        }
                    } else if let CssNode::AtRootDirect(inner) = kid {
                        // BEM AtRootDirect 嵌套修复
                        let orig_selector = self.selector.clone();
                        let orig_resolved = self.resolved_self.clone();
                        self.selector = combined.clone();
                        self.resolved_self = combined.clone();
                        self.push_atroot_direct(*inner);
                        self.selector = orig_selector;
                        self.resolved_self = orig_resolved;
                    } else {
                        // Group 4 修复：递归 dispatch
                        self.push(kid);
                    }
                }
            }
            other => {
                self.flush_decls();
                let other = match other {
                    CssNode::AtRule {
                        name,
                        params,
                        children,
                        has_body: true,
                    } => {
                        let is_kf = name == "keyframes"
                            || name == "-webkit-keyframes"
                            || name == "-moz-keyframes";
                        let ch = if is_kf {
                            children
                        } else {
                            nest_rule_in_children(&self.selector, children)
                        };
                        CssNode::AtRule {
                            name,
                            params,
                            children: ch,
                            has_body: true,
                        }
                    }
                    CssNode::AtRule {
                        name,
                        params,
                        children: _,
                        has_body: false,
                    } => CssNode::Rule {
                        selector: self.selector.clone(),
                        declarations: vec![],
                        children: vec![CssNode::AtRule {
                            name,
                            params,
                            children: vec![],
                            has_body: false,
                        }],
                    },
                    other => other,
                };
                self.result.push(other);
            }
        }
    }

    /// 处理 AtRootDirect 节点：组合选择器 + 递归处理嵌套子节点。
    ///
    /// 核心语义：
    /// 1. AtRootDirect 的 selector 与 self.selector 组合（& 替换或嵌套）
    /// 2. 父 Rule 仅含自身声明，不含子节点
    /// 3. 子节点（含嵌套 AtRootDirect）生成独立 Rule，selector 与 resolved 组合
    fn push_atroot_direct(&mut self, inner: CssNode) {
        match inner {
            CssNode::Rule { selector, declarations, children } => {
                // 关键：trim 尾随逗号——SCSS @at-root mixin 的选择器可能有 "&--large," 形式
                let clean_sel = selector.trim().trim_end_matches(',').trim();
                let resolved_selector = if clean_sel.contains('&') {
                    let combined = combine_selectors(&self.selector, clean_sel);
                    combined.trim().trim_end_matches(',').trim().to_string()
                } else if clean_sel.starts_with(':') || clean_sel.starts_with('[') {
                    // 伪类/属性选择器：直接拼接（后缀型，需依附于父选择器）
                    format!("{}{}", self.selector, clean_sel)
                } else {
                    // 普通类选择器：已是完整路径（m() mixin 后缀输出或 e() mixin 字面输出），直接使用。
                    clean_sel.to_string()
                };
                crate::__tracing::debug!(
                    target: "sasspile::rule_builder",
                    self_sel = %self.selector,
                    atroot_sel = %selector,
                    resolved = %resolved_selector,
                    n_children = children.len(),
                    "AtRootDirect push"
                );
                // 组合后的父 Rule（仅含自身声明）
                self.result.push(CssNode::Rule {
                    selector: resolved_selector.clone(),
                    declarations,
                    children: vec![],
                });
                // 子节点：AtRootDirect 需展开并组合，Rule 直接嵌套
                for child in children {
                    match child {
                        CssNode::Rule { selector: kid_sel, declarations: kid_decls, children: kid_kids } => {
                            // kid_sel 也可能含尾随逗号——trim 后组合
                            let clean_kid = kid_sel.trim().trim_end_matches(',').trim();
                            // DOUBLE-PREFIX FIX：检测 child 是否已包含 parent 前缀。
                            let kid_already_has = has_descendant_prefix(&resolved_selector, clean_kid);
                            let kid_combined = if kid_already_has {
                                clean_kid.to_string()
                            } else {
                                combine_selectors(&resolved_selector, clean_kid)
                            };
                            self.result.push(CssNode::Rule {
                                selector: kid_combined,
                                declarations: kid_decls,
                                children: kid_kids,
                            });
                        }
                        CssNode::AtRootDirect(nested_inner) => {
                            // 嵌套 AtRootDirect：用 resolved_selector 作为新的 self.selector 递归处理
                            let orig_selector = self.selector.clone();
                            self.selector = resolved_selector.clone();
                            self.push_atroot_direct(*nested_inner);
                            self.selector = orig_selector;
                        }
                        other => {
                            // 其他节点（Declaration 等）：直接 push
                            self.result.push(other);
                        }
                    }
                }
            }
            other => {
                // 非 Rule 内部节点：直接 push
                self.result.push(other);
            }
        }
    }

    /// 消费构建器，返回最终节点列表。
    ///
    /// 关键语义：@at-root 提升节点紧跟在父规则声明块之后、嵌套子规则之前——
    /// 与 dart-sass 一致（EP 的 BEM mixin 链依赖此顺序）。
    pub(super) fn build(mut self) -> Vec<CssNode> {
        self.flush_decls();
        if self.result.is_empty() && self.root_nodes.is_empty() {
            self.result.push(CssNode::Rule {
                selector: self.selector,
                declarations: vec![],
                children: vec![],
            });
        } else if !self.root_nodes.is_empty() {
            // 将 root_nodes 插入到父声明块之后、嵌套子规则之前——
            // 与 dart-sass 语义一致：@at-root 规则在父声明块后立即输出，
            // 在所有非-@at-root 嵌套子规则之前。
            //
            // 当父规则无声明块时（纯 mixin  mixin 调用），
            // root_nodes 插入到所有子节点之前。
            let mut combined: Vec<CssNode> = Vec::new();
            let mut hoisted = false;
            for node in self.result {
                if !hoisted {
                    if let CssNode::Rule { selector, .. } = &node {
                        if selector == &self.selector {
                            combined.push(node);
                            combined.extend(self.root_nodes.iter().cloned());
                            hoisted = true;
                            continue;
                        }
                    }
                }
                combined.push(node);
            }
            self.result = if hoisted {
                combined
            } else {
                // 父声明块不存在：root_nodes 插在所有子节点之前
                let mut all = self.root_nodes;
                all.extend(combined);
                all
            };
        }
        self.result
    }
}
