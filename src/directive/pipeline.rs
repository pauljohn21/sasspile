//! 统一响应式管线 — Flux 思维 + rxrust 算子组合
//!
//! Flux 模型:  groupBy(classify) → flatMap(各 group 独立 scanWith) → merge
//! rxrust 等价: scan_map(accumulate_block) → flat_map(process_block) → collect
//!
//! 每个指令类型不是手写 handler,而是 rxrust 算子链的一个 sub-flow

use crate::css::{CssBuilder, CssNode, render_node};
use rxrust::prelude::*;
use std::convert::Infallible;
use tracing::{debug_span, info_span};

use super::blocks::{accumulate_block, expand_block, BlockAccumulator, DirectiveBlock};
use super::state::CompileState;

// ─── Sass 缩进语法: +name → @include name ───────────────────────────────────

#[inline]
fn transform_indented_include(line: String) -> String {
    let t = line.trim_start();
    if let Some(after_plus) = t.strip_prefix('+') {
        let after = after_plus.trim_start();
        // +foo → @include foo
        // +foo($a, $b) → @include foo($a, $b)
        if !after.is_empty() && !after.starts_with('@') {
            // 替换行首 + 为 @include (保留缩进)
            let leading = &line[..line.len() - t.len()];
            return format!("{leading}@include {after}");
        }
    }
    line
}

// ─── @media 合并 (函数式 fold, into_iter 零 clone) ──────────────────────────

/// 预处理: 拆分 "} @else" 行为单独行
fn split_else_line(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    // "} @else {"
    if let Some(rest) = trimmed.strip_prefix("}").map(str::trim_start) {
        if rest.starts_with("@else if ") || rest.starts_with("@elseif ") || rest == "@else" || rest.starts_with("@else ") {
            let mut result = vec!["}".to_string()];
            result.push(rest.to_string());
            return result;
        }
    }
    vec![line.to_string()]
}

/// CSS 选择器嵌套展平: .parent { .child { color: red; } } → .parent .child { color: red; }
fn flatten_nested_selectors(nodes: Vec<CssNode>) -> Vec<CssNode> {
    let mut result = Vec::new();
    for node in nodes {
        result.push(flatten_node(node, ""));
    }
    result
}

fn flatten_node(node: CssNode, parent_sel: &str) -> CssNode {
    match node {
        CssNode::Rule { selector, children } => {
            let full_selector = if parent_sel.is_empty() {
                selector
            } else if selector.starts_with('&') {
                format!("{}{}", parent_sel, &selector[1..])
            } else {
                format!("{} {}", parent_sel, selector)
            };
            let new_children: Vec<CssNode> = children
                .into_iter()
                .map(|c| flatten_node(c, &full_selector))
                .collect();
            CssNode::Rule {
                selector: full_selector,
                children: new_children,
            }
        }
        CssNode::AtRule { query, children } => {
            // AtRule 保持子节点嵌套 (不展平)
            CssNode::AtRule { query, children }
        }
        other => other,
    }
}

/// Post-processing: 解析 @extend 标记, 合并选择器
/// 将 ExtendMarker 中找到的目标规则的选择器扩展为 "target, extender"
#[allow(clippy::redundant_clone)]
fn resolve_extend_markers(nodes: Vec<CssNode>) -> Vec<CssNode> {
    // 收集所有 extend 标记
    let mut markers: Vec<(String, String, bool)> = vec![]; // (extender, target, optional)
    let mut rule_indices: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    // 第一遍: 收集所有规则索引
    for (i, node) in nodes.iter().enumerate() {
        if let CssNode::Rule { selector, .. } = node {
            rule_indices.insert(selector.clone(), i);
        }
    }

    // 收集 extend 标记
    for node in &nodes {
        if let CssNode::ExtendMarker { extender, target, optional } = node {
            markers.push((extender.clone(), target.clone(), *optional));
        }
    }

    // 构建: target(原始选择器字符串) → 该规则当前最新的合并选择器
    let mut current_selector: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    // 初始化: 每个已知规则的选择器
    for node in &nodes {
        if let CssNode::Rule { selector, .. } = node {
            current_selector.insert(selector.clone(), selector.clone());
        }
    }

    // 应用 extend: 将 extender 追加到 target 选择器
    for (extender, target, optional) in &markers {
        // 支持多目标: target 可能是逗号分隔的列表
        let targets: Vec<&str> = target.split(',').map(str::trim).filter(|t| !t.is_empty()).collect();
        let mut merged_any = false;
        for single_target in &targets {
            // 查找: 是否有规则的(当前)选择器包含此 target?
            // 先尝试精确匹配 current_selector 的 key
            if let Some(existing) = current_selector.get(*single_target) {
                let merged = format!("{existing}, {extender}");
                current_selector.insert(single_target.to_string(), merged.clone());
                current_selector.insert(extender.clone(), merged);
                merged_any = true;
            } else {
                // 尝试部分匹配: 某个规则的选择器逗号列表包含此 target
                let mut found = false;
                for (rule_sel, current_val) in current_selector.clone() {
                    let parts: Vec<&str> = rule_sel.split(',').map(str::trim).collect();
                    if parts.contains(single_target) {
                        let merged = format!("{current_val}, {extender}");
                        current_selector.insert(rule_sel.clone(), merged.clone());
                        current_selector.insert(extender.clone(), merged);
                        merged_any = true;
                        found = true;
                        break;
                    }
                }
                if found {
                    continue;
                }
            }
        }
        if !merged_any {
            if *optional {
                continue; // optional + 目标不存在: 静默忽略
            }
            // 非 optional: 至少让 extender 自身可查
            current_selector.entry(extender.clone()).or_insert_with(|| extender.clone());
        }
    }

    // 第二遍: 应用选择器修改, 移除 ExtendMarker
    let mut result: Vec<CssNode> = Vec::new();
    let mut removed_extenders: std::collections::HashSet<String> = std::collections::HashSet::new();

    // 收集需要移除的 extender (成功匹配的)
    for (extender, target, _) in &markers {
        let target_trimmed = target.split(',').next().unwrap_or(target).trim();
        if let Some(current) = current_selector.get(target_trimmed) {
            if rule_indices.contains_key(target_trimmed) || current != target_trimmed {
                removed_extenders.insert(extender.clone());
            }
        }
    }

    for node in &nodes {
        match node {
            CssNode::ExtendMarker { .. } => continue,
            CssNode::Rule { selector, children } => {
                // 如果是 extender (成功匹配), 跳过 (已被合并到 target)
                if removed_extenders.contains(selector) {
                    continue;
                }
                // 应用选择器合并: 查找当前最新选择器
                if let Some(new_sel) = current_selector.get(selector) {
                    if new_sel != selector {
                        result.push(CssNode::Rule {
                            selector: new_sel.clone(),
                            children: children.clone(),
                        });
                        continue;
                    }
                }
                result.push(node.clone());
            }
            _ => result.push(node.clone()),
        }
    }
    result
}

fn merge_media_nodes(nodes: Vec<CssNode>) -> Vec<CssNode> {
    let (result, _media_idx) = nodes.into_iter().fold(
        (Vec::<CssNode>::new(), std::collections::HashMap::<String, usize>::new()),
        |(mut acc, mut idx), node| {
            let merge_target = match &node {
                CssNode::AtRule { query, .. } if query.starts_with("@media ") => {
                    idx.get(query).copied()
                }
                _ => None,
            };

            if let Some(merge_idx) = merge_target {
                if let Some(CssNode::AtRule { children: existing, .. }) = acc.get_mut(merge_idx) {
                    if let CssNode::AtRule { children: new_children, .. } = node {
                        existing.extend(new_children);
                    }
                }
                (acc, idx)
            } else if let CssNode::AtRule { query, .. } = &node {
                if query.starts_with("@media ") {
                    idx.insert(query.clone(), acc.len());
                }
                acc.push(node);
                (acc, idx)
            } else {
                acc.push(node);
                (acc, idx)
            }
        },
    );
    result
}

// ═══════════════════════════════════════════════════════════════════════════
// 管线入口 — 算子链 (chain = 声明, subscribe = 执行边界)
// ═══════════════════════════════════════════════════════════════════════════

/// 多文件编译 — @use/@forward 模块系统
///
/// 预处理: 抽取 @use/@forward 行 → 解析路径 + 配置 → 递归编译模块 → 拼接到主输入前
#[allow(clippy::redundant_clone)]
pub fn compile_pipeline_with_files(input: &str, files: &std::collections::HashMap<String, String>) -> String {
    let _root = info_span!("compile_with_files", bytes = input.len(), file_count = files.len()).entered();

    let (module_css, remaining_input) = process_use_directives(input, files, &mut std::collections::HashSet::new());

    // 主管线输入 = 模块 CSS (已编译) + 去除 @use 行的主文件
    let combined_input = if module_css.is_empty() {
        remaining_input
    } else {
        format!("{module_css}\n{remaining_input}")
    };

    compile_pipeline(&combined_input)
}

/// 抽取并编译 @use/@forward 行, 返回 (模块 CSS 输出, 去除 @use 行后的输入)
fn process_use_directives(
    input: &str,
    files: &std::collections::HashMap<String, String>,
    loading: &mut std::collections::HashSet<String>,
) -> (String, String) {
    let mut module_outputs: Vec<String> = Vec::new();
    let mut remaining_lines: Vec<String> = Vec::new();
    let mut pending_use: Option<(bool, String)> = None;

    for line in input.lines() {
        let trimmed = line.trim();

        // 累积跨行 @use / @forward (直到括号平衡)
        if let Some((is_forward, acc)) = pending_use.clone() {
            let mut accumulated = acc;
            accumulated.push(' ');
            accumulated.push_str(trimmed);
            if accumulated.matches('(').count() <= accumulated.matches(')').count() {
                flush_pending_use(&accumulated, is_forward, files, loading, &mut module_outputs);
                pending_use = None;
            } else {
                pending_use = Some((is_forward, accumulated));
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("@use ") {
            if rest.matches('(').count() > rest.matches(')').count() {
                pending_use = Some((false, rest.to_string()));
            } else {
                let (path, config) = parse_use_with(rest);
                if !loading.contains(&path) {
                    loading.insert(path.clone());
                    if let Some(css) = compile_module(&path, &config, files, loading) {
                        module_outputs.push(css);
                    }
                }
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("@forward ") {
            if rest.matches('(').count() > rest.matches(')').count() {
                pending_use = Some((true, rest.to_string()));
            } else {
                let path = parse_forward_path(rest);
                if !loading.contains(&path) {
                    loading.insert(path.clone());
                    if let Some(css) = compile_module(&path, &[], files, loading) {
                        module_outputs.push(css);
                    }
                }
            }
            continue;
        }

        remaining_lines.push(line.to_string());
    }

    (module_outputs.join("\n"), remaining_lines.join("\n"))
}

/// 处理累积的跨行 @use / @forward 指令 (rest = 去掉 @use / @forward 前缀后的内容)
fn flush_pending_use(
    rest: &str,
    is_forward: bool,
    files: &std::collections::HashMap<String, String>,
    loading: &mut std::collections::HashSet<String>,
    module_outputs: &mut Vec<String>,
) {
    if is_forward {
        let path = parse_forward_path(rest);
        if !loading.contains(&path) {
            loading.insert(path.clone());
            if let Some(css) = compile_module(&path, &[], files, loading) {
                module_outputs.push(css);
            }
        }
    } else {
        let (path, config) = parse_use_with(rest);
        if !loading.contains(&path) {
            loading.insert(path.clone());
            if let Some(css) = compile_module(&path, &config, files, loading) {
                module_outputs.push(css);
            }
        }
    }
}

/// 解析 @use "path" with ($a: val, $b: val) → (path, 配置变量表)
fn parse_use_with(s: &str) -> (String, Vec<(String, String)>) {
    let s = s.trim();
    // 提取引号内的路径
    let path = if let Some(start) = s.find('"') {
        if let Some(end) = s[start + 1..].find('"') {
            s[start + 1..start + 1 + end].to_string()
        } else {
            s.to_string()
        }
    } else {
        s.to_string()
    };

    // 解析 with() 配置
    let mut config = Vec::new();
    if let Some(with_start) = s.find("with") {
        let after_with = &s[with_start + 4..];
        if let Some(p_start) = after_with.find('(') {
            if let Some(p_end) = after_with.rfind(')') {
                let params = &after_with[p_start + 1..p_end];
                for pair in params.split(',') {
                    let pair = pair.trim();
                    if pair.is_empty() { continue; }
                    if let Some((name, value)) = pair.split_once(':') {
                        let name = name.trim().to_string();
                        let value = value.trim().to_string();
                        if !name.is_empty() && !value.is_empty() {
                            config.push((name, value));
                        }
                    }
                }
            }
        }
    }

    (path, config)
}

/// 解析 @forward "path" → path
fn parse_forward_path(s: &str) -> String {
    let s = s.trim();
    if let Some(start) = s.find('"') {
        if let Some(end) = s[start + 1..].find('"') {
            return s[start + 1..start + 1 + end].to_string();
        }
    }
    s.to_string()
}

/// 文件路径解析: 精确 / "_"前缀 / 扩展名 / index文件
fn resolve_file_path<'a>(path: &str, files: &'a std::collections::HashMap<String, String>) -> Option<&'a String> {
    // 直接 (含扩展名)
    if let Some(content) = files.get(path) {
        return Some(content);
    }
    // 加下划线前缀
    let with_underscore = format!("_{path}");
    if let Some(content) = files.get(&with_underscore) {
        return Some(content);
    }
    // 加 .scss 扩展名
    let with_scss = format!("{path}.scss");
    if let Some(content) = files.get(&with_scss) {
        return Some(content);
    }
    let with_underscore_scss = format!("_{path}.scss");
    if let Some(content) = files.get(&with_underscore_scss) {
        return Some(content);
    }
    // 加 .sass 扩展名
    let with_sass = format!("{path}.sass");
    if let Some(content) = files.get(&with_sass) {
        return Some(content);
    }
    let with_underscore_sass = format!("_{path}.sass");
    if let Some(content) = files.get(&with_underscore_sass) {
        return Some(content);
    }
    // 目录 index 文件 (for @use "module" => module/_index.scss / module/index.scss)
    let index_scss = format!("{path}/_index.scss");
    if let Some(content) = files.get(&index_scss) {
        return Some(content);
    }
    let index_scss2 = format!("{path}/index.scss");
    if let Some(content) = files.get(&index_scss2) {
        return Some(content);
    }
    let index_sass = format!("{path}/_index.sass");
    if let Some(content) = files.get(&index_sass) {
        return Some(content);
    }
    let index_sass2 = format!("{path}/index.sass");
    if let Some(content) = files.get(&index_sass2) {
        Some(content)
    } else {
        None
    }
}

/// 编译单个模块: 注入配置变量 + 递归预处理 + 主管线编译
fn compile_module(
    path: &str,
    config: &[(String, String)],
    files: &std::collections::HashMap<String, String>,
    loading: &mut std::collections::HashSet<String>,
) -> Option<String> {
    let _span = info_span!("compile_module", path = %path, config_count = config.len()).entered();

    let content = resolve_file_path(path, files)?;

    // 注入配置变量 (with() 覆盖 !default): 作为最高优先级变量预置
    let config_prefix: String = config
        .iter()
        .map(|(name, value)| format!("{name}: {value};"))
        .collect::<Vec<_>>()
        .join("\n");

    let combined = if config_prefix.is_empty() {
        content.to_string()
    } else {
        format!("{config_prefix}\n{content}")
    };

    // 递归处理: 模块内部可能还有 @use
    let (module_css, remaining_module_input) =
        process_use_directives(&combined, files, loading);

    let module_main_input = if module_css.is_empty() {
        remaining_module_input
    } else {
        format!("{module_css}\n{remaining_module_input}")
    };

    // 编译主模块内容 (reuse single-file pipeline)
    let compiled = compile_pipeline(&module_main_input);

    // 过滤掉空行后返回
    let filtered: String = compiled
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    if filtered.is_empty() {
        None
    } else {
        Some(filtered)
    }
}

pub fn compile_pipeline(input: &str) -> String {
    let _root = info_span!("compile_pipeline", bytes = input.len()).entered();

    // Shared Subject 入口 (String: Shared 需要 'static + Send)
    // Flux 思维: 这就是 Flux.create() 的 Sinks.Many.asFlux()
    let subject = Shared::subject::<String, Infallible>();
    let (tx, rx) = std::sync::mpsc::channel::<String>();

    // 构建算子链 (声明式, 此时不执行)
    subject
        .clone()
        // ══════════════════════════════════════════════════════════════════
        // Phase 0: 预处理 — Sass 缩进语法 +name → @include name
        // ══════════════════════════════════════════════════════════════════
        .map(transform_indented_include)
        .tap(|line: &String| {
            let _s = debug_span!("phase0_preprocess", line = %line).entered();
        })
        // ══════════════════════════════════════════════════════════════════
        // Phase 1: 指令分块 + 展开 (scan_map + flat_map)
        // ══════════════════════════════════════════════════════════════════
        .scan_map(BlockAccumulator::default(), accumulate_block)
        .tap(|blocks: &Vec<DirectiveBlock>| {
            let _s = debug_span!("phase1_blocks", count = blocks.len(), kinds = ?blocks.iter().map(std::any::type_name_of_val).collect::<Vec<_>>()).entered();
        })
        .flat_map(|v: Vec<DirectiveBlock>| Shared::from_iter(v))
        .scan_map(CompileState::new(), expand_block)
        .tap(|lines: &Vec<String>| {
            let _s = debug_span!("phase1_expanded", count = lines.len(), first = lines.first().map(|s| s.as_str()).unwrap_or("")).entered();
        })
        .flat_map(|v: Vec<String>| Shared::from_iter(v))
        .tap(|line: &String| {
            let _s = debug_span!("phase1_out", line = %line).entered();
        })
        // ══════════════════════════════════════════════════════════════════
        // Phase 2: CSS AST 构建 (scan_map CssBuilder)
        // ══════════════════════════════════════════════════════════
        .scan_map(CssBuilder::new(), |builder: &mut CssBuilder, line: String| {
            builder.feed(&line)
        })
        .flat_map(|v: Vec<CssNode>| Shared::from_iter(v))
        .collect::<Vec<CssNode>>()
        .last()
        .map(flatten_nested_selectors)
        .map(resolve_extend_markers)
        .map(merge_media_nodes)
        .tap(|nodes: &Vec<CssNode>| {
            let _s = debug_span!("phase2_merged", count = nodes.len()).entered();
        })
        .flat_map(|nodes| Shared::from_iter(nodes))
        // ══════════════════════════════════════════════════════════════════
        // Phase 3: 渲染 (&借用 → String, 零 clone)
        // ══════════════════════════════════════════════════════════════════
        .map(|node: CssNode| render_node(&node))
        .tap(|css: &String| {
            let _s = debug_span!("phase3_css", css = %css).entered();
        })
        .collect::<Vec<String>>()
        .last()
        // ══════════════════════════════════════════════════════════════════
        // 终端: subscribe = 执行边界, move 转移终态
        // ══════════════════════════════════════════════════════════════════
        .subscribe(move |css_vec: Vec<String>| {
            let _ = tx.send(css_vec.join("\n"));
        });

    // 驱动: 预处理 split } @else → 2 lines → push all → flush → complete
    let preprocessed: Vec<String> = input
        .lines()
        .flat_map(split_else_line)
        .collect();
    for line in &preprocessed {
        subject.clone().next(line.to_string());
    }
    // sentinel flush: 空行触发 pending_branches 回写
    subject.clone().next(String::new());
    subject.clone().complete();

    rx.recv().unwrap_or_default()
}
