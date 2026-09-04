use super::*;
use crate::css::node::CssNode;
use crate::css::selector_parser::parse_selector;
use crate::css::selector_ops;

impl Evaluator {
    /// 收集 CSS 中所有选择器文本（用于 extend target 匹配检查）。
    fn collect_selectors(nodes: &[CssNode]) -> Vec<String> {
        nodes
            .iter()
            .flat_map(|node| {
                let own: Vec<String> = match node {
                    CssNode::Rule {
                        selector, children, ..
                    } => {
                        let mut v = vec![selector.clone()];
                        v.extend(Self::collect_selectors(children));
                        v
                    }
                    CssNode::AtRule { children, .. } => Self::collect_selectors(children),
                    CssNode::AtRoot(kids, _) => Self::collect_selectors(kids),
                    _ => Vec::new(),
                };
                own
            })
            .collect()
    }

    pub(crate) fn apply_extends(
        nodes: Vec<CssNode>,
        extends: &[(String, String, bool, Option<PathBuf>)],
        module_selectors: &HashMap<PathBuf, std::collections::HashSet<String>>,
    ) -> Vec<CssNode> {
        let span = crate::__tracing::info_span!("apply_extends", n_extends = extends.len());
        let _enter = span.enter();
        nodes
            .into_iter()
            .map(|node| {
                match node {
                    CssNode::Rule {
                        selector,
                        children,
                        declarations,
                    } => {
                        crate::__tracing::debug!(
                            target: "sasspile::extend",
                            selector = %selector,
                            "processing rule for extends"
                        );
                        // 用 AST 进行 extend——fold 累积扩展
                        let sel_ast = extends.iter().fold(
                            parse_selector(&selector),
                            |sel_ast, (extender, target, _optional, module)| {
                                let target_trimmed = target.trim();
                                let extender_trimmed = extender.trim();
                                // bogus 选择器检测
                                if extender_trimmed.ends_with('+')
                                    || extender_trimmed.ends_with('>')
                                    || extender_trimmed.ends_with('~')
                                {
                                    return sel_ast;
                                }
                                // 模块 scope 检查
                                if let Some(module_path) = module {
                                    let in_scope = module_selectors.get(module_path).map_or_else(
                                        || {
                                            module_selectors
                                                .values()
                                                .any(|s| s.contains(target_trimmed))
                                        },
                                        |s| s.contains(target_trimmed),
                                    );
                                    if !in_scope {
                                        return sel_ast;
                                    }
                                }
                                let extendee = parse_selector(target_trimmed);
                                let ext = parse_selector(extender_trimmed);
                                let new_sel = selector_ops::extend_selector(&sel_ast, &extendee, &ext);
                                crate::__tracing::debug!(
                                    target: "sasspile::extend",
                                    new_selector = %new_sel,
                                    "extend applied"
                                );
                                new_sel
                            },
                        );
                        // 递归处理子规则
                        let children = Self::apply_extends(children, extends, module_selectors);
                        // 移除未被继承的占位符选择器——filter + collect
                        let selector = crate::css::selector_ast::Selector(
                            sel_ast
                                .0
                                .into_iter()
                                .filter(|c| {
                                    !c.compounds.iter().all(|(_, comp)| {
                                        comp.0.iter().all(|s| matches!(
                                            s,
                                            crate::css::selector_ast::SimpleSelector::Placeholder(_)
                                        ))
                                    })
                                })
                                .collect(),
                        )
                        .to_string();
                        CssNode::Rule {
                            selector,
                            declarations,
                            children,
                        }
                    }
                    CssNode::AtRule {
                        name,
                        params,
                        children,
                        has_body: true,
                    } => {
                        let children = Self::apply_extends(children, extends, module_selectors);
                        CssNode::AtRule {
                            name,
                            params,
                            children,
                            has_body: true,
                        }
                    }
                    CssNode::AtRoot(kids, q) => {
                        CssNode::AtRoot(Self::apply_extends(kids, extends, module_selectors), q)
                    }
                    other => other,
                }
            })
            .collect()
    }

    /// 检查未匹配的 extend target——非 optional 的未匹配 target 报错。
    pub(crate) fn check_extend_targets(
        css: &[CssNode],
        extends: &[(String, String, bool, Option<PathBuf>)],
    ) -> Result<()> {
        let span = crate::__tracing::debug_span!("check_extend_targets", n_extends = extends.len());
        let _enter = span.enter();
        let all_selectors = Self::collect_selectors(css);
        extends
            .iter()
            .try_fold((), |(), (_extender, target, optional, _module)| {
                if *optional {
                    return Ok(());
                }
                let target_trimmed = target.trim();
                // 占位符选择器不需要在 CSS 中存在
                if target_trimmed.starts_with('%') {
                    return Ok(());
                }
                let found = all_selectors.iter().any(|s| s.contains(target_trimmed));
                if !found {
                    return Err(SassError::Eval(format!(
                        "The target selector was not found.\nUse \"@extend {target_trimmed} !optional\" to avoid this error."
                    )));
                }
                Ok(())
            })
    }

    /// 从模块缓存构建路径→选择器集合的映射
    pub(crate) fn build_module_selectors(
        cache: &HashMap<PathBuf, ModuleExports>,
    ) -> HashMap<PathBuf, std::collections::HashSet<String>> {
        cache
            .iter()
            .map(|(k, v)| (k.clone(), v.selectors.clone()))
            .collect()
    }

    /// 收集模块 CSS 中所有选择器，加上当前模块直接 @use 的模块的选择器
    /// `ast`: 当前模块的 AST，用于提取 @use 路径
    pub(crate) fn collect_all_selectors(
        cache: &HashMap<PathBuf, ModuleExports>,
        module_path: &std::path::Path,
        css: &[CssNode],
        ast: &crate::parse::ast::Ast,
        load_paths: &[PathBuf],
    ) -> std::collections::HashSet<String> {
        // 从 AST 中提取 @use 的模块路径——flat_map + collect
        let base = Some(module_path.to_path_buf());
        let base_ref = base.as_ref();
        let module_selectors: Vec<String> = ast
            .nodes
            .iter()
            .filter_map(|node| {
                let crate::parse::ast::Node::Use { url, .. } = node else {
                    return None;
                };
                if url.starts_with("sass:") {
                    return None;
                }
                let path = Self::resolve_file(base_ref, url, load_paths)?;
                let v = cache.get(&path)?;
                Some(Self::collect_selectors(&v.css).into_iter().chain(v.selectors.iter().cloned()))
            })
            .flatten()
            .collect();
        Self::collect_selectors(css)
            .into_iter()
            .chain(module_selectors)
            .collect()
    }
}
