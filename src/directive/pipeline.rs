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

/// 合并 @import 多行 modifier (sass 语法: 缩进行自动合并到上一行)
///
/// 例如: `@import "a.css"\n  b` → `@import "a.css" b;`
fn merge_import_lines(lines: &[String]) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = &lines[i];
        let trimmed = line.trim();

        // 检查是否是未闭合的 @import/@charset (不以 ; 结尾)
        if (trimmed.starts_with("@import ") || trimmed.starts_with("@charset "))
            && !trimmed.ends_with(';')
            && !trimmed.ends_with('}')
        {
            let mut merged = line.clone();
            // 合并后续缩进行
            while i + 1 < lines.len() {
                let next = &lines[i + 1];
                let next_trimmed = next.trim();
                // 空行终止
                if next_trimmed.is_empty() {
                    break;
                }
                // 非缩进行终止 (无 leading whitespace)
                if !next.starts_with(' ') && !next.starts_with('\t') {
                    break;
                }
                // 处理逗号开头的行 (新 @import)
                if next_trimmed.starts_with(',') {
                    // 合并逗号
                    merged = format!("{merged}{next_trimmed}");
                } else {
                    merged = format!("{merged} {next_trimmed}");
                }
                i += 1;
            }
            result.push(merged);
        } else {
            result.push(line.clone());
        }
        i += 1;
    }
    result
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
/// 预处理: 解析 @use/@forward → 命名空间重写 → 注入带前缀的模块成员 → 主文件引用替换
#[allow(clippy::redundant_clone)]
pub fn compile_pipeline_with_files(input: &str, files: &std::collections::HashMap<String, String>) -> String {
    let _root = info_span!("compile_with_files", bytes = input.len(), file_count = files.len()).entered();

    let (injection, rewritten_main) = crate::directive::module_system::process_module_imports(input, files);

    // 主管线输入 = 已注入的模块成员 (带 ns 前缀) + 主文件重写
    let combined_input = if injection.trim().is_empty() {
        rewritten_main
    } else {
        format!("{injection}\n{rewritten_main}")
    };

    compile_pipeline(&combined_input)
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

    // 驱动: 预处理 split } @else → 2 lines → merge @import → push all → flush → complete
    let preprocessed: Vec<String> = input
        .lines()
        .flat_map(split_else_line)
        .collect();
    let preprocessed = merge_import_lines(&preprocessed);
    for line in &preprocessed {
        subject.clone().next(line.to_string());
    }
    // sentinel flush: 空行触发 pending_branches 回写
    subject.clone().next(String::new());
    subject.clone().complete();

    rx.recv().unwrap_or_default()
}
