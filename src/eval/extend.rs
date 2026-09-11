use super::*;
use crate::css::node::CssNode;
use crate::css::selector_parser::parse_selector;
use crate::css::selector_ops;
use imbl::HashSet;

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
        module_selectors: &HashMap<PathBuf, HashSet<String>>,
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
                                match extender_trimmed.ends_with('+')
                                    || extender_trimmed.ends_with('>')
                                    || extender_trimmed.ends_with('~')
                                {
                                    true => return sel_ast,
                                    false => {}
                                }
                                // 模块 scope 检查——仅当 module_selectors 非空且 module 在 map 中时执行
                                // - module_selectors 为空：无模块化编译，跳过检查（全局匹配）
                                // - module_path 不在 map 中：extend 来自入口文件（全局上下文），视为可见
                                // - module_path 在 map 中：检查 target 在该模块 selector set 中的可见性
                                 if let Some(module_path) = module {
                                    if let Some(set) = module_selectors.get(module_path) {
                                        // 子串匹配：选择器字符串是组合形式（如 "%in-other.a"），
                                        // 而 target 是子部分（如 "%in-other"），需用 iter+contains 做子串检查
                                        let in_scope = set.iter().any(|sel| sel.contains(target_trimmed));
                                        if !in_scope {
                                            return sel_ast;
                                        }
                                    }
                                    // module_path 不在 module_selectors 中 → 入口文件/forwarded 上下文
                                    // 视为全局可见（与 no-modules 编译语义一致）
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
    ///
    /// `global_placeholders`: 所有已加载模块中定义的 placeholder 选择器集合。
    /// `module_selectors`: 每个模块路径 → 该模块可见的选择器集合（含传递 @use）。
    /// 用于检测跨模块 scope 违规（target 存在但声明模块不可见）。
    pub(crate) fn check_extend_targets(
        css: &[CssNode],
        extends: &[(String, String, bool, Option<PathBuf>)],
        global_placeholders: &HashSet<String>,
        module_selectors: &HashMap<PathBuf, HashSet<String>>,
    ) -> Result<()> {
        let span = crate::__tracing::debug_span!("check_extend_targets", n_extends = extends.len());
        let _enter = span.enter();
        let all_selectors = Self::collect_selectors(css);
        // 全局 public 选择器集合（所有模块的 selectors 并集）
        let global_selectors: HashSet<String> = module_selectors
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
                match target_trimmed.starts_with('%') {
                    true => {
                        let defined_elsewhere = global_placeholders.contains(target_trimmed);
                        // 单文件（无 @use）场景下 placeholder 不被模块缓存收集——额外扫描 CSS
                        let defined_in_css = all_selectors.iter().any(|s| s.contains(target_trimmed));
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
