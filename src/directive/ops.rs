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

// ─── Block 处理入口 ─────────────────────────────────────────────────────────

pub fn process_block(block: DirectiveBlock, state: &mut CompileState) -> Vec<String> {
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
    if trimmed.is_empty() {
        return vec![];
    }

    // 规则关闭: 注入 pending @extend declarations
    if trimmed == "}" && state.current_rule_name.is_some() {
        let mut out = Vec::new();
        if !state.pending_extend_decls.is_empty() {
            for d in state.pending_extend_decls.drain(..) {
                out.push(format!("  {d};"));
            }
        }
        out.push(line.to_string());
        state.current_rule_name = None;
        return out;
    }

    if trimmed.starts_with('$') && trimmed.contains(':') {
        if let Some((name, value)) = parse_var_def(trimmed) {
            state.scope.variables.insert(name, value);
        }
        return vec![];
    }
    if trimmed.starts_with("@include ") {
        return handle_include(trimmed, state);
    }
    if trimmed.starts_with("@extend ") && state.current_rule_name.is_some() {
        if let Some(decl) = parse_inline_extend(trimmed, state) {
            state.pending_extend_decls.push(decl);
            return vec![];
        }
        if let Some(marker) = build_extend_marker(trimmed, state) {
            return vec![marker];
        }
        return vec![];
    }
    // 行内 @extend: ".bar { @extend %foo; }"
    if trimmed.contains("@extend ") && trimmed.ends_with('}') {
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

        return vec![substitute_vars(&*state, trimmed)];
    }

    // 规则开启: ".wrapper {"
    if !trimmed.starts_with('@') && !trimmed.starts_with('$') && trimmed.ends_with('{') {
        if let Some(selector) = trimmed.strip_suffix('{').map(str::trim) {
            if !selector.is_empty() && !selector.starts_with('@') {
                state.current_rule_name = Some(selector.to_string());
            }
        }
        return vec![substitute_vars(&*state, trimmed)];
    }

    vec![substitute_vars(&*state, trimmed)]
}

// ─── @include 行处理 ────────────────────────────────────────────────────────

fn handle_include(line: &str, state: &CompileState) -> Vec<String> {
    let (name, args) = parse_include_sig(&line[9..]);
    state
        .scope
        .mixins
        .get(&name)
        .map(|def| {
            expand_mixin(def, &args)
                .into_iter()
                .flat_map(|b| expand_single_line(&b, state))
                .collect()
        })
        .unwrap_or_default()
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

fn parse_var_def(line: &str) -> Option<(String, String)> {
    if !line.starts_with('$') {
        return None;
    }
    let after_dollar = &line[1..];
    let (name, value_part) = after_dollar.split_once(':')?;
    let name = format!("${}", name.trim());
    let value = value_part.trim().trim_end_matches(';').trim().to_string();
    if value.is_empty() {
        return None;
    }
    Some((name, value))
}
