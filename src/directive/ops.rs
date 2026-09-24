//! 独立 block 展开 — flat_map reducer (无 zxrust trait, 纯函数)
//!
//! DirectiveBlock → Vec<String> (借 &mut CompileState 就地修改 + &CompileState 读取)

use super::blocks::DirectiveBlock;
use super::parse::{expand_mixin, parse_include_sig, substitute_vars};
use super::state::CompileState;

pub fn process_block(block: DirectiveBlock, state: &mut CompileState) -> Vec<String> {
    // 只读视图: &CompileState 是 Copy, 闭包可安全共享
    let state_ref: &CompileState = &*state;

    match block {
        DirectiveBlock::Lines(lines) => lines
            .into_iter()
            .flat_map(|line| process_line(&line, state))
            .collect(),
        DirectiveBlock::For {
            ref var_name,
            ref values,
            ref body,
        } => values
            .iter()
            .flat_map(|v| {
                body.iter()
                    .map(|b| substitute_vars(state_ref, &b.replace(var_name, v)))
            })
            .collect(),
        DirectiveBlock::Each {
            ref var_name,
            ref items,
            ref body,
        } => items
            .iter()
            .flat_map(|i| {
                body.iter()
                    .map(|b| substitute_vars(state_ref, &b.replace(var_name, i)))
            })
            .collect(),
        DirectiveBlock::If { ref branches } => {
            for (cond, body) in branches {
                let take = match cond {
                    None => true,
                    Some(expr) => eval_condition(state_ref, expr),
                };
                if take {
                    return body.clone();
                }
            }
            vec![]
        }
        DirectiveBlock::MixinDef { name, params, body } => {
            state.scope.mixins.insert(
                name,
                crate::directive::state::MixinDef { params, body },
            );
            vec![]
        }
    }
}

/// 单行处理: 变量 def / @include / @extend @ plain CSS
fn process_line(line: &str, state: &mut CompileState) -> Vec<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return vec![];
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
    if trimmed.starts_with("@extend ") {
        return handle_extend(trimmed, &*state);
    }
    vec![substitute_vars(&*state, trimmed)]
}

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

fn handle_extend(line: &str, state: &CompileState) -> Vec<String> {
    let args = line[8..].trim().trim_end_matches(';');
    let (placeholder, _optional) = match args.find("!optional") {
        Some(idx) => (args[..idx].trim(), true),
        None => (args, false),
    };
    state
        .placeholder_defs
        .get(placeholder)
        .cloned()
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

fn eval_condition(state: &CompileState, expr: &str) -> bool {
    let expr = substitute_vars(state, expr);
    let t = expr.trim();
    if t.is_empty() || t == "false" || t == "null" || t == "0" {
        return false;
    }
    if t == "true" {
        return true;
    }
    if let Some(idx) = t.find("==") {
        let (l, r) = t.split_at(idx);
        return l.trim() == r[2..].trim().trim_matches('"');
    }
    if let Some(idx) = t.find("!=") {
        let (l, r) = t.split_at(idx);
        return l.trim() != r[2..].trim().trim_matches('"');
    }
    if let Some(rest) = t.strip_prefix("not ") {
        return !eval_condition(state, rest);
    }
    true
}
