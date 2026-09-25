//! @extend 辅助函数 — 从 ops.rs 拆出
//!
//! 包含: extend 标记构建, inline extend 解析/处理, placeholder declarations 提取

use super::parse::substitute_vars;
use super::state::CompileState;

// ─── @extend 标记构建 ───────────────────────────────────────────────────────

/// 构建 extend 标记字符串 (注入管线 → CssBuilder → CssNode::ExtendMarker)
pub(super) fn build_extend_marker(line: &str, state: &CompileState) -> Option<String> {
    let extender = if let Some(name) = &state.current_rule_name {
        name.clone()
    } else {
        line.split('{').next()?.trim().to_string()
    };
    if extender.is_empty() {
        return None;
    }
    let after_extend = line.split("@extend ").nth(1)?;
    let target_spec = after_extend
        .split(|c: char| c == ';' || c == '}')
        .next()?
        .trim();
    if target_spec.is_empty() {
        return None;
    }
    let target_spec = if let Some(idx) = target_spec.find("//") {
        &target_spec[..idx]
    } else {
        target_spec
    };
    let target_spec = target_spec.replace("/**/", "").replace("/*", "").replace("*/", "");
    let target_spec = target_spec.trim();
    if target_spec.is_empty() {
        return None;
    }
    let optional = target_spec.contains("!optional");
    let targets: Vec<String> = target_spec
        .split(',')
        .map(|t| t.trim().trim_end_matches("!optional").trim().to_string())
        .filter(|t| !t.is_empty())
        .collect();
    if targets.is_empty() {
        return None;
    }
    if targets.iter().any(|t| t == &extender) {
        return None;
    }
    let target = targets.join(":");
    Some(format!(">>EXTEND:{}:{}:{}", extender, target, optional))
}

// ─── @extend 辅助函数 ───────────────────────────────────────────────────────

/// 从单行 @extend 提取 placeholder declaration (跨行/单行通用)
pub(super) fn parse_inline_extend(line: &str, state: &CompileState) -> Option<String> {
    let after = if let Some(extend_idx) = line.find("@extend ") {
        &line[extend_idx + 8..]
    } else {
        line
    };
    let end_rel = after.find(|c: char| c == ';' || c == '}')?;
    let placeholder = after[..end_rel].trim().trim_end_matches(';').trim();
    let placeholder = placeholder.strip_suffix("!optional").unwrap_or(placeholder).trim();
    let placeholder_name = placeholder.strip_prefix('%').unwrap_or(placeholder);

    let body = state.placeholder_defs.get(placeholder_name)?;
    Some(extract_declarations(body).trim().to_string())
}

/// 处理行内 @extend: ".bar { @extend %foo; }" → ".bar { color: red; }"
pub(super) fn handle_inline_extend(line: &str, state: &CompileState) -> Vec<String> {
    let Some(extend_start) = line.find("@extend ") else {
        return vec![substitute_vars(state, line)];
    };
    let after_extend = &line[extend_start + 8..];
    let Some(semi_rel) = after_extend.find(|c: char| c == ';' || c == '}') else {
        return vec![substitute_vars(state, line)];
    };
    let placeholder_name = after_extend[..semi_rel].trim().trim_end_matches(';').trim();
    let (placeholder_name, _optional) = match placeholder_name.strip_suffix("!optional") {
        Some(name) => (name.trim(), true),
        None => (placeholder_name, false),
    };
    let placeholder_name = placeholder_name.strip_prefix('%').unwrap_or(placeholder_name);

    let body = state.placeholder_defs.get(placeholder_name);

    let without_extend_slice = format!("{}{}", &line[..extend_start], &line[extend_start + 8 + semi_rel + 1..]);
    let without_extend = without_extend_slice.trim();

    let Some(body) = body else {
        if without_extend.is_empty() || without_extend == "}" || without_extend == "{ }" || without_extend == "{}" {
            return vec![];
        }
        return vec![substitute_vars(state, without_extend)];
    };

    let decls = extract_declarations(body);

    let Some(open_brace) = without_extend.rfind('{') else {
        return vec![decls];
    };
    let Some(close_brace) = without_extend.rfind('}') else {
        return vec![decls];
    };

    let inner = format!("{}{}{}", &without_extend[..open_brace + 1], decls, &without_extend[close_brace..]);
    vec![substitute_vars(state, &inner)]
}

/// 从 placeholder body 提取 declarations (去掉外层花括号)
pub(super) fn extract_declarations(body: &[String]) -> String {
    let joined = body.join(" ");
    let trimmed = joined.trim();
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        trimmed[1..trimmed.len() - 1].trim().to_string()
    } else {
        trimmed.to_string()
    }
}

/// 从规则行中移除 @extend 行 (保留其余声明)
pub(super) fn remove_extend_line(line: &str) -> String {
    let Some(extend_rel) = line.find("@extend ") else {
        return line.to_string();
    };
    let after_extend = &line[extend_rel + 8..];
    let end_rel = after_extend.find(|c: char| c == ';' || c == '}').unwrap_or(after_extend.len());
    let before = &line[..extend_rel];
    let after = &line[extend_rel + 8 + end_rel..];
    let after = after.strip_prefix(';').unwrap_or(after);
    let result = format!("{before}{after}");
    let result = result.trim();
    if let Some(open) = result.find('{') {
        if let Some(close) = result.rfind('}') {
            let inner = &result[open + 1..close].trim();
            if inner.is_empty() {
                return String::new();
            }
        }
    }
    result.to_string()
}
