//! 独立 block 展开 — scan_map reducer 入口 + @include 行处理
//!
//! 原 610 行已拆分为:
//!   - extend_ops.rs: @extend 辅助函数 (~90 行)
//!   - while_ops.rs: @while 展开 + @if 条件求值 (~80 行)
//!
//! 本文件保留: process_block 入口, process_line, handle_include, expand_single_line

use super::blocks::DirectiveBlock;
use super::extend_ops::{build_extend_marker, handle_inline_extend, parse_inline_extend, remove_extend_line};
use super::parse::{expand_mixin, parse_include_sig, substitute_vars};
use super::state::CompileState;
use super::while_ops::eval_condition;
use tracing::info_span;

// ─── 辅助函数 ───────────────────────────────────────────────────────────────

fn block_variant_name(block: &DirectiveBlock) -> &'static str {
    match block {
        DirectiveBlock::Lines(_) => "Lines",
        DirectiveBlock::For { .. } => "For",
        DirectiveBlock::Each { .. } => "Each",
        DirectiveBlock::If { .. } => "If",
        DirectiveBlock::FunctionDef { .. } => "FunctionDef",
        DirectiveBlock::MixinDef { .. } => "MixinDef",
        DirectiveBlock::PlaceholderDef { .. } => "PlaceholderDef",
        DirectiveBlock::While { .. } => "While",
        DirectiveBlock::Include { .. } => "Include",
    }
}

// ─── 行类型枚举 — 用于 process_line 的单层 match 分发 ─────────────────────

/// 行类型 — 结构化分类, 消除嵌套 if/return 链
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineKind {
    /// 空行
    Empty,
    /// 规则关闭: "}"
    CloseBrace,
    /// 变量定义: "$var: value;"
    VarDef,
    /// @include 指令
    Include,
    /// @extend 指令 (规则内)
    AtExtend,
    /// 行内 @extend: ".bar { @extend %foo; }"
    InlineExtend,
    /// 规则开启: ".selector {"
    RuleStart,
    /// 普通 CSS 行
    Plain,
}

/// 将行文本分类为 LineKind (纯函数, &str 借用 → Copy 输出, 无副作用)
fn classify_line(line: &str) -> LineKind {
    if line.is_empty() {
        return LineKind::Empty;
    }
    if line == "}" {
        return LineKind::CloseBrace;
    }
    if line.starts_with('$') && line.contains(':') {
        return LineKind::VarDef;
    }
    if line.starts_with("@include ") {
        return LineKind::Include;
    }
    if line.starts_with("@extend ") {
        return LineKind::AtExtend;
    }
    if line.contains("@extend ") && line.ends_with('}') {
        return LineKind::InlineExtend;
    }
    if !line.starts_with('@') && !line.starts_with('$') && line.ends_with('{') {
        return LineKind::RuleStart;
    }
    LineKind::Plain
}

// ─── Block 处理入口 ─────────────────────────────────────────────────────────

pub fn process_block(block: DirectiveBlock, state: &mut CompileState) -> Vec<String> {
    let _span = info_span!("process_block", stage = "phase1", variant = ?block_variant_name(&block)).entered();
    let state_ref: &CompileState = &*state;

    match block {
        DirectiveBlock::Lines(lines) => {
            lines.into_iter().flat_map(|line| process_line(&line, state)).collect()
        }
        DirectiveBlock::For { ref var_name, ref values, ref body } => values
            .iter()
            .flat_map(|v| body.iter().map(|b| {
                // 先替换 #{$var} 整体, 再替换裸 $var, 避免破坏插值语法
                let step1 = b.replace(&format!("#{{{var_name}}}"), v);
                substitute_vars(state_ref, &step1.replace(var_name, v))
            }))
            .collect(),
        DirectiveBlock::Each { ref var_name, ref items, ref body } => items
            .iter()
            .flat_map(|i| body.iter().map(|b| {
                let step1 = b.replace(&format!("#{{{var_name}}}"), i);
                substitute_vars(state_ref, &step1.replace(var_name, i))
            }))
            .collect(),
        DirectiveBlock::If { ref branches } => {
            for (cond, body) in branches {
                let take = match cond {
                    None => true,
                    Some(expr) => eval_condition(state_ref, expr),
                };
                if take { return body.clone(); }
            }
            vec![]
        }
        DirectiveBlock::FunctionDef { name, params, return_value, body } => {
            state.define_function(name, crate::directive::state::FunctionDef { params, return_value, body });
            vec![]
        }
        DirectiveBlock::MixinDef { name, params, body } => {
            state.scope.mixins.insert(name, crate::directive::state::MixinDef { params, body });
            vec![]
        }
        DirectiveBlock::PlaceholderDef { name, body } => {
            state.placeholder_defs.insert(name, body);
            vec![]
        }
        DirectiveBlock::While { cond, body } => {
            // while 展开逻辑已迁移到 while_ops.rs
            super::while_ops::expand_while(state, &cond, &body)
        }
        DirectiveBlock::Include { name, args, using, body } => {
            expand_include_multi(state, &name, &args, &using, &body)
        }
    }
}

// ─── 多行 @include 展开 (含 @content 替换) ──────────────────────────────────

fn expand_include_multi(state: &CompileState, name: &str, args: &[String], using: &[String], body: &[String]) -> Vec<String> {
    let Some(def) = state.scope.mixins.get(name) else {
        return vec![];
    };

    let mut effective_args = args.to_vec();
    if !using.is_empty() {
        effective_args.extend_from_slice(using);
    }

    let expanded = expand_mixin(def, &effective_args);

    expanded.iter().flat_map(|line| {
        if line.contains("@content") {
            body.iter().map(|b| b.as_str()).collect::<Vec<&str>>()
        } else {
            vec![line.as_str()]
        }
    }).map(|s| s.to_string())
    .collect()
}

// ─── 单行处理 (变量 def / @include / @extend / plain CSS) ──────────────────

fn process_line(line: &str, state: &mut CompileState) -> Vec<String> {
    let trimmed = line.trim();
    let kind = classify_line(trimmed);

    // 单层 match 分发 — 消除嵌套 if/return 链
    match kind {
        LineKind::Empty => vec![],

        LineKind::CloseBrace => {
            // 规则关闭: 注入 pending @extend declarations
            if state.current_rule_name.is_some() {
                let mut out = Vec::new();
                if !state.pending_extend_decls.is_empty() {
                    for d in state.pending_extend_decls.drain(..) {
                        out.push(format!("  {d};"));
                    }
                }
                out.push(line.to_string());
                state.current_rule_name = None;
                out
            } else {
                vec![substitute_vars(&*state, trimmed)]
            }
        }

        LineKind::VarDef => {
            if let Some((name, value, is_default)) = parse_var_def(trimmed) {
                // !default 语义: 仅当变量未定义时赋值
                if is_default && state.scope.variables.contains_key(&name) {
                    return vec![];
                }
                state.scope.variables.insert(name, value);
                // 处理行内 ";" 之后的剩余内容
                if let Some(semi_rel) = trimmed.find(';') {
                    let remainder = trimmed[semi_rel + 1..].trim();
                    if !remainder.is_empty() {
                        return process_line(remainder, state);
                    }
                }
            }
            vec![]
        }

        LineKind::Include => handle_include(trimmed, state),

        LineKind::AtExtend => {
            // !optional + 不存在的 placeholder: 剥离 @extend 部分, 保留行内其余声明
            if state.current_rule_name.is_some() && is_optional_nonexistent_extend(trimmed, state) {
                let after_extend = trimmed.strip_prefix("@extend ").unwrap_or(trimmed);
                let end_rel = after_extend.find(|c: char| c == ';' || c == '}').unwrap_or(after_extend.len());
                let remainder = after_extend[end_rel..].trim_start_matches(';').trim();
                if remainder.is_empty() {
                    return vec![];
                }
                return process_line(remainder, state);
            }
            if state.current_rule_name.is_some() {
                if let Some(decl) = parse_inline_extend(trimmed, state) {
                    if !decl.is_empty() {
                        state.pending_extend_decls.push(decl);
                        return vec![];
                    }
                }
                if let Some(marker) = build_extend_marker(trimmed, state) {
                    return vec![marker];
                }
            }
            vec![]
        }

        LineKind::InlineExtend => {
            // 行内 @extend: ".bar { @extend %foo; }"
            let after_extend = trimmed.split("@extend ").nth(1).unwrap_or("");
            let target_spec = after_extend
                .split(|c: char| c == ';' || c == '}')
                .next()
                .unwrap_or("")
                .trim();
            let is_placeholder = target_spec.starts_with('%');
            let placeholder_name = target_spec.strip_prefix('%')
                .map(|s| s.trim_end_matches("!optional").trim())
                .unwrap_or("");
            let is_optional = target_spec.contains("!optional");
            let placeholder_exists = state.placeholder_defs.contains_key(placeholder_name);

            if is_placeholder && placeholder_exists {
                let placeholder_result = handle_inline_extend(trimmed, state);
                if !placeholder_result.is_empty() {
                    return placeholder_result;
                }
            }

            if let Some(marker) = build_extend_marker(trimmed, state) {
                let inner = trimmed
                    .trim_start_matches(|c: char| c != '{')
                    .trim_start_matches('{')
                    .trim_end_matches('}')
                    .trim();
                let only_extend = {
                    let without_semi = inner.trim_end_matches(';').trim();
                    without_semi.starts_with("@extend")
                        && !without_semi[7..].contains(';')
                };

                if is_placeholder && is_optional && !placeholder_exists {
                    if only_extend {
                        return vec![];
                    }
                    let without_extend = remove_extend_line(trimmed);
                    if without_extend.is_empty() || without_extend == "{" || without_extend == "{}" {
                        return vec![];
                    }
                    return vec![without_extend];
                }

                if only_extend {
                    return vec![marker];
                }
                let without_extend = remove_extend_line(trimmed);
                if without_extend.is_empty() || without_extend == "{" || without_extend == "{}" {
                    return vec![marker];
                }
                return vec![marker, without_extend];
            }

            vec![substitute_vars(&*state, trimmed)]
        }

        LineKind::RuleStart => {
            // 规则开启: ".wrapper {"
            if let Some(selector) = trimmed.strip_suffix('{').map(str::trim) {
                if !selector.is_empty() && !selector.starts_with('@') {
                    state.current_rule_name = Some(selector.to_string());
                }
            }
            vec![substitute_vars(&*state, trimmed)]
        }

        LineKind::Plain => vec![substitute_vars(&*state, trimmed)],
    }
}

// ─── @include 行处理 ────────────────────────────────────────────────────────

fn handle_include(line: &str, state: &CompileState) -> Vec<String> {
    let (name, args) = parse_include_sig(&line[9..]);
    if let Some(def) = state.scope.mixins.get(&name) {
        // Phase 1 找到 mixin → 展开
        expand_mixin(def, &args)
            .into_iter()
            .flat_map(|b| expand_single_line(&b, state))
            .collect()
    } else {
        // Phase 1 未找到 (mixin 可能定义在嵌套规则内, 由 CssBuilder 收集)
        // 原样返回给 Phase 2 CssBuilder 处理
        vec![line.to_string()]
    }
}

fn expand_single_line(line: &str, state: &CompileState) -> Vec<String> {
    let tr = line.trim();
    if let (true, true) = (tr.starts_with('{'), tr.ends_with('}')) {
        let inner = &tr[1..tr.len() - 1];
        inner
            .split(';')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| format!("{};", substitute_vars(state, s)))
            .collect()
    } else {
        vec![substitute_vars(state, tr)]
    }
}

// ─── 变量定义解析 ───────────────────────────────────────────────────────────

/// 检测 @extend 行是否为 !optional + 不存在的 placeholder
/// 匹配模式: @extend %name !optional; 或 @extend %name ... !optional;
fn is_optional_nonexistent_extend(line: &str, state: &CompileState) -> bool {
    let after = match line.strip_prefix("@extend ") {
        Some(a) => a,
        None => return false,
    };
    // 提取 targets 部分 (; 或 } 之前)
    let targets_part = after
        .split(|c: char| c == ';' || c == '}')
        .next()
        .unwrap_or("")
        .trim();
    if !targets_part.contains("!optional") {
        return false;
    }
    // 检查所有 placeholder target 是否都不存在
    let targets: Vec<&str> = targets_part
        .split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .collect();
    if targets.is_empty() {
        return false;
    }
    targets.iter().all(|t| {
        let name = t
            .trim_end_matches("!optional")
            .trim()
            .strip_prefix('%')
            .unwrap_or("");
        !state.placeholder_defs.contains_key(name)
    })
}

fn parse_var_def(line: &str) -> Option<(String, String, bool)> {
    if !line.starts_with('$') {
        return None;
    }
    let after_dollar = &line[1..];
    let (name, value_part) = after_dollar.split_once(':')?;
    let name = format!("${}", name.trim());
    // 值从 ":" 后第一个非空字符开始到第一个 ";" 结束 (CSS 声明终结符)
    let raw_value = value_part.trim();
    // 检测 !default 标志
    let is_default = raw_value.contains("!default");
    // 在第一个 ";" 处截断, 避免吞掉行内后续内容
    let value_until_semi = raw_value.split(';').next().unwrap_or(raw_value).trim();
    let value = value_until_semi
        .trim_end_matches("!default")
        .trim()
        .to_string();
    if value.is_empty() {
        return None;
    }
    Some((name, value, is_default))
}
