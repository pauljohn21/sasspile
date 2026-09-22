use super::*;
use crate::css::node::CssNode;
use crate::css::selector_parser::parse_selector;
use crate::css::selector_ops;
use imbl::HashSet;

impl Evaluator {
    /// 收集 CSS 中所有选择器文本（用于 extend target 匹配检查）。
    pub(crate) fn collect_selectors(nodes: &[CssNode]) -> Vec<String> {
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
                    CssNode::AtRootDirect(inner) => Self::collect_selectors(std::slice::from_ref(inner)),
                    _ => Vec::new(),
                };
                own
            })
            .collect()
    }

    pub(crate) fn apply_extends(
        nodes: Vec<CssNode>,
        extends: &[(String, String, bool, Option<PathBuf>)],
        module_selectors: &HashMap<PathBuf, HashSet<String>>,
    ) -> Vec<CssNode> {
        let span = crate::__tracing::info_span!("apply_extends", n_extends = extends.len());
        let _enter = span.enter();
        // DEBUG: 打印所有 extends
        for (ext, tgt, opt, _mod) in extends {
            crate::__tracing::trace!(
                target: "sasspile::extend_debug",
                extender = %ext,
                target = %tgt,
                optional = %opt,
                "extend entry"
            );
        }

        // Phase 1: 构建 %placeholder → [extenders] 映射
        let mut placeholder_groups: HashMap<String, Vec<String>> = HashMap::new();
        for (extender, target, _optional, _module) in extends {
            if target.trim().starts_with('%') {
                placeholder_groups
                    .entry(target.trim().to_string())
                    .or_default()
                    .push(extender.trim().to_string());
            }
        }
        crate::__tracing::trace!(
            target: "sasspile::extend_debug",
            n_placeholder_groups = placeholder_groups.len(),
            groups = ?placeholder_groups.keys().collect::<Vec<_>>(),
            "placeholder groups"
        );

        // Phase 2: 过滤非 placeholder extends
        let placeholder_targets: HashSet<String> = placeholder_groups.keys().cloned().collect();
        let non_placeholder_extends: Vec<_> = extends
            .iter()
            .filter(|(_, target, _, _)| !placeholder_targets.contains(target.trim()))
            .cloned()
            .collect();

        // Phase 3: 就地转换 %placeholder 规则为组合选择器，并处理其余节点
        Self::transform_nodes(nodes, &placeholder_groups, &non_placeholder_extends, module_selectors)
    }

    /// 递归转换节点：
    /// - `%placeholder { decls }` 规则（被 extend 的）→ `.ext1, .ext2 { decls }`
    /// - `%placeholder { decls }` 规则（未被 extend 的）→ 移除
    /// - 其他节点 → 保留，递归处理子节点
    fn transform_nodes(
        nodes: Vec<CssNode>,
        placeholder_groups: &HashMap<String, Vec<String>>,
        non_placeholder_extends: &[(String, String, bool, Option<PathBuf>)],
        module_selectors: &HashMap<PathBuf, HashSet<String>>,
    ) -> Vec<CssNode> {
        // 构建 extender → 需要前置的占位符声明（仅单 extender 场景）
        // 注意：extender key 是 eval_rule 时 env.get_selector() 的局部选择器（如 .child），
        // 但 RuleBuilder 输出的规则选择器是完整组合形式（如 .parent .child）。
        // 因此匹配时需要用"后缀匹配"：检查 extender 是否为 rule 选择器的后缀。
        let mut extender_extra_decls: Vec<(String, Vec<CssNode>)> = Vec::new();
        let mut placeholder_decls: HashMap<String, Vec<CssNode>> = HashMap::new();
        Self::collect_placeholder_decls(&nodes, &mut placeholder_decls);
        for (placeholder, extenders) in placeholder_groups {
            if extenders.len() == 1 {
                if let Some(decls) = placeholder_decls.get(placeholder) {
                    extender_extra_decls.push((extenders[0].trim().to_string(), decls.clone()));
                }
            }
        }

        nodes
            .into_iter()
            .filter_map(|node| match node {
                CssNode::Rule { selector, children, declarations } => {
                    let sel = selector.trim();
                    if sel.starts_with('%') {
                        // 占位符规则
                        match placeholder_groups.get(sel) {
                            Some(extenders) if extenders.len() >= 2 && !declarations.is_empty() => {
                                // 多 extender：生成组合选择器规则（在占位符位置）
                                let combined_selector = extenders.join(", ");
                                crate::__tracing::debug!(
                                    target: "sasspile::extend",
                                    combined = %combined_selector,
                                    placeholder = %sel,
                                    "placeholder multi-extend → combined rule"
                                );
                                Some(CssNode::Rule {
                                    selector: combined_selector,
                                    declarations,
                                    children: vec![],
                                })
                            }
                            Some(extenders) if extenders.len() == 1 => {
                                // 单 extender：移除占位符规则，声明已合并到 extender 规则
                                crate::__tracing::debug!(
                                    target: "sasspile::extend",
                                    extender = %extenders[0],
                                    placeholder = %sel,
                                    "placeholder single-extend → remove (merged into extender)"
                                );
                                None
                            }
                            _ => {
                                // 未被 extend 或空声明 → 移除
                                None
                            }
                        }
                    } else {
                        // 普通规则：处理非 placeholder extends
                        let sel_ast = non_placeholder_extends.iter().fold(
                            parse_selector(&selector),
                            |sel_ast, (extender, target, _optional, module)| {
                                let target_trimmed = target.trim();
                                let extender_trimmed = extender.trim();
                                if extender_trimmed.ends_with('+')
                                    || extender_trimmed.ends_with('>')
                                    || extender_trimmed.ends_with('~')
                                {
                                    return sel_ast;
                                }
                                if let Some(module_path) = module {
                                    if let Some(set) = module_selectors.get(module_path) {
                                        let in_scope = set.iter().any(|s| s.contains(target_trimmed));
                                        if !in_scope {
                                            return sel_ast;
                                        }
                                    }
                                }
                                let extendee = parse_selector(target_trimmed);
                                let ext = parse_selector(extender_trimmed);
                                selector_ops::extend_selector(&sel_ast, &extendee, &ext)
                            },
                        );
                        // 递归处理子节点
                        let children = Self::transform_nodes(children, placeholder_groups, non_placeholder_extends, module_selectors);
                        let selector_str = crate::css::selector_ast::Selector(
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

                        // 单 extender 场景：前置占位符声明
                        // 后缀匹配：extender 如 ".child" 应匹配规则选择器 ".parent .child"
                        let mut final_decls = declarations;
                        let matched_extra = extender_extra_decls.iter().find_map(|(ext, decls)| {
                            if decls.is_empty() {
                                return None;
                            }
                            let ext_trimmed = ext.trim();
                            if selector_str == ext_trimmed {
                                return Some(decls);
                            }
                            // 后缀匹配：selector_str 以 " ext" 结尾（以组合器开头）
                            // 如 ".parent .child" 以 " .child" 结尾
                            for comb in [" ", ">", "+", "~"] {
                                let suffix = format!("{comb}{ext_trimmed}");
                                if selector_str.strip_suffix(&suffix).is_some() {
                                    return Some(decls);
                                }
                            }
                            None
                        });
                        if let Some(extra) = matched_extra {
                            let mut merged = extra.clone();
                            merged.extend(final_decls);
                            final_decls = merged;
                        }

                        Some(CssNode::Rule {
                            selector: selector_str,
                            declarations: final_decls,
                            children,
                        })
                    }
                }
                CssNode::AtRule { name, params, children, has_body: true } => {
                    let children = Self::transform_nodes(children, placeholder_groups, non_placeholder_extends, module_selectors);
                    Some(CssNode::AtRule { name, params, children, has_body: true })
                }
                CssNode::AtRoot(kids, q) => {
                    Some(CssNode::AtRoot(Self::transform_nodes(kids, placeholder_groups, non_placeholder_extends, module_selectors), q))
                }
                CssNode::AtRootDirect(inner) => {
                    let inner_vec = vec![*inner];
                    let applied = Self::transform_nodes(inner_vec, placeholder_groups, non_placeholder_extends, module_selectors);
                    Some(CssNode::AtRootDirect(Box::new(applied.into_iter().next().expect("transform_nodes returns one node per AtRootDirect"))))
                }
                other => Some(other),
            })
            .collect()
    }

    /// 收集 CSS 树中所有 %placeholder 规则的声明（递归）
    fn collect_placeholder_decls(nodes: &[CssNode], out: &mut HashMap<String, Vec<CssNode>>) {
        for n in nodes {
            match n {
                CssNode::Rule { selector, children, declarations } => {
                    let sel = selector.trim();
                    if sel.starts_with('%') && children.is_empty() {
                        out.insert(sel.to_string(), declarations.clone());
                    }
                    Self::collect_placeholder_decls(children, out);
                }
                CssNode::AtRule { children, .. } => Self::collect_placeholder_decls(children, out),
                CssNode::AtRoot(kids, _) => Self::collect_placeholder_decls(kids, out),
                CssNode::AtRootDirect(inner) => Self::collect_placeholder_decls(std::slice::from_ref(inner), out),
                _ => {}
            }
        }
    }

    /// 检查未匹配的 extend target——非 optional 的未匹配 target 报错。
    ///
    /// `global_placeholders`: 所有已加载模块中定义的 placeholder 选择器集合。
    /// `module_selectors`: 每个模块路径 → 该模块可见的选择器集合（含传递 @use）。
    /// 用于检测跨模块 scope 违规（target 存在但声明模块不可见）。
    pub(crate) fn check_extend_targets(
        css: &[CssNode],
        extends: &[(String, String, bool, Option<PathBuf>)],
        global_placeholders: &HashSet<String>,
        module_selectors: &HashMap<PathBuf, HashSet<String>>,
        original_selectors: &[String],
    ) -> Result<()> {
        let span = crate::__tracing::debug_span!("check_extend_targets", n_extends = extends.len());
        let _enter = span.enter();
        let all_selectors = Self::collect_selectors(css);
        // 全局 public 选择器集合（所有模块的 selectors 并集）
        let _global_selectors: HashSet<String> = module_selectors
            .values()
            .flat_map(|s| s.iter().cloned())
            .collect();
        extends
            .iter()
            .try_fold((), |(), (_extender, target, optional, module)| {
                                match *optional {
                                    true => return Ok(()),
                                    false => {}
                                }
                let target_trimmed = target.trim();
                // 占位符选择器：在最终 CSS 中不可见，但 extend 仍应成功。
                // placeholder extend 的语义：将 extender 的 declarations 复制到 placeholder 位置，
                // placeholder 本身不出现在最终输出。只要 placeholder 在某处定义即视为成功。
                // 注意：apply_extends 会过滤掉 placeholder 规则，故检查 original_selectors（原始 CSS 收集）。
                match target_trimmed.starts_with('%') {
                    true => {
                        let defined_elsewhere = global_placeholders.contains(target_trimmed);
                        // 使用 original_selectors：apply_extends 后 placeholder 规则可能已被过滤
                        let defined_in_css = original_selectors.iter().any(|s| s.contains(target_trimmed));
                        match defined_elsewhere || defined_in_css {
                            true => return Ok(()),
                            false => return Err(SassError::Eval(format!(
                                "The target selector was not found.\nUse \"@extend {target_trimmed} !optional\" to avoid this error."
                            ))),
                        }
                    }
                    false => {}
                }
                // 检查 public target 是否存在于最终 CSS（快速路径）
                let in_final_css = all_selectors.iter().any(|s| s.contains(target_trimmed));
                match in_final_css {
                    true => {
                        // target 存在于最终 CSS。检查声明模块是否能看见它。
                        // 仅当 module_selectors 非空且 module 存在于 map 中时才做 scope 检查——
                        // - 空 map：无模块化上下文（无 @use），scope 限制不适用。
                        // - module 不在 map 中（如顶层 file 不被 @use 加载）：视为完全可见。
                        if let Some(mod_path) = module {
                            if let Some(set) = module_selectors.get(mod_path) {
                                let visible = set.iter().any(|s| s.contains(target_trimmed));
                                if !visible {
                                    return Err(SassError::Eval(format!(
                                        "The target selector was not found.\nUse \"@extend {target_trimmed} !optional\" to avoid this error."
                                    )));
                                }
                            }
                        }
                        Ok(())
                    }
                    false => {
                        // target 不在最终 CSS 中。
                        // 如果它存在于某个模块但不可见 → scope 违规。
                        // 如果根本不存在 → "not found" 错误。
                        Err(SassError::Eval(format!(
                            "The target selector was not found.\nUse \"@extend {target_trimmed} !optional\" to avoid this error."
                        )))
                    }
                }
            })
    }

    /// 从模块缓存构建路径→选择器集合的映射
    pub(crate) fn build_module_selectors(
        cache: &HashMap<PathBuf, ModuleExports>,
    ) -> HashMap<PathBuf, HashSet<String>> {
        cache
            .iter()
            .map(|(k, v)| (k.clone(), v.selectors.clone()))
            .collect()
    }

    /// 从模块缓存中收集所有 placeholder 选择器（跨模块并集）。
    pub(crate) fn build_global_placeholders(
        cache: &HashMap<PathBuf, ModuleExports>,
    ) -> HashSet<String> {
        cache
            .values()
            .flat_map(|v| v.placeholder_selectors.iter().cloned())
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
    ) -> HashSet<String> {
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
                match url.starts_with("sass:") {
                    true => return None,
                    false => {}
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
