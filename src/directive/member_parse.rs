//! 模块成员解析 — 从模块文件内容提取 var/fn/mixin/raw_rules

use crate::directive::module_system::{ModuleMembers, NsMap};

/// 解析模块内容，提取变量/函数/mixin/原始规则 (最高层成员)
pub fn parse_module_members(content: &str) -> ModuleMembers {
    let mut members = ModuleMembers::default();
    let mut block: Option<(String, Vec<String>)> = None; // (head, body)
    let mut depth = 0i32;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some((_, ref mut body)) = block {
            body.push(line.to_string());
            depth += count_braces(line);
            if depth <= 0 {
                if let Some((head, body)) = block.take() {
                    emit_parsed_block(&head, &body, &mut members);
                }
                depth = 0;
            }
            continue;
        }

        if let Some((name, value, is_default)) = try_extract_var(trimmed) {
            members.variables.push((name, value, is_default));
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("@function ") {
            let (name_paren, body_open) = split_fn_head(rest);
            if body_open {
                let name_only = extract_name_only(&name_paren);
                let params = extract_fn_params(&name_paren);
                let return_value = extract_return_value(&[rest.to_string()]);
                members.functions.push((name_only, params, return_value));
            } else {
                let name = name_paren.trim().to_string();
                block = Some((format!("fn {name}"), vec![]));
                depth = count_braces(line);
                depth = if depth <= 0 { block = None; 0 } else { depth };
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("@mixin ") {
            let (name_paren, body_open) = split_fn_head(rest);
            if body_open {
                let name_only = extract_name_only(&name_paren);
                let params = extract_fn_params(&name_paren);
                // 提取 { ... } 之间的内容作为 body
                let inner = if let Some(open) = rest.find('{') {
                    if let Some(close) = rest.rfind('}') {
                        rest[open + 1..close].trim().to_string()
                    } else {
                        rest.to_string()
                    }
                } else {
                    rest.to_string()
                };
                members.mixins.push((name_only, params, vec![inner]));
            } else {
                let name = name_paren.trim().to_string();
                block = Some((format!("mx {name}"), vec![]));
                depth = count_braces(line);
                depth = if depth <= 0 { block = None; 0 } else { depth };
            }
            continue;
        }

        if trimmed.starts_with("@include ") || trimmed.starts_with("@return ") {
            members.raw_rules.push(line.to_string());
            continue;
        }

        members.raw_rules.push(line.to_string());
    }

    members
}

fn emit_parsed_block(head: &str, body: &[String], members: &mut ModuleMembers) {
    if head.starts_with("fn ") {
        let name = head[3..].trim().to_string();
        let params = extract_fn_params(&name);
        let name_only = extract_name_only(&name);
        let return_value = extract_return_value(body);
        members.functions.push((name_only, params, return_value));
    } else if head.starts_with("mx ") {
        let name = head[3..].trim().to_string();
        let params = extract_fn_params(&name);
        let name_only = extract_name_only(&name);
        members.mixins.push((name_only, params, body.to_vec()));
    }
}

// ─── 行解析辅助 ─────────────────────────────────────────────────────────────

fn try_extract_var(trimmed: &str) -> Option<(String, String, bool)> {
    let after = trimmed.strip_prefix('$')?;
    let (name, rest) = after.split_once(':')?;
    let name = format!("${}", name.trim());
    let is_default = rest.contains("!default");
    let value = rest
        .trim()
        .trim_end_matches(';')
        .trim()
        .trim_end_matches("!default")
        .trim()
        .trim_end_matches(';')
        .trim()
        .to_string();
    if value.is_empty() {
        return None;
    }
    Some((name, value, is_default))
}

fn extract_name_only(s: &str) -> String {
    s.split('(').next().unwrap_or(s).trim().to_string()
}

fn split_fn_head(rest: &str) -> (String, bool) {
    if let Some(paren_start) = rest.find('(') {
        let name = rest[..paren_start].trim().to_string();
        let after_paren = &rest[paren_start..];
        let has_brace = after_paren.contains('{');
        (format!("{name}({after_paren}"), has_brace)
    } else {
        let name = rest.split_whitespace().next().unwrap_or(rest).trim().to_string();
        let has_brace = rest.contains('{');
        (name, has_brace)
    }
}

pub(crate) fn extract_fn_params(s: &str) -> Vec<String> {
    if let Some(start) = s.find('(') {
        if let Some(end) = s[start..].find(')') {
            let inner = &s[start + 1..start + end];
            return inner
                .split(',')
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .map(String::from)
                .collect();
        }
    }
    vec![]
}

pub(crate) fn extract_return_value(body: &[String]) -> String {
    for line in body {
        if let Some(after) = line.trim().strip_prefix("@return ") {
            return after.trim().trim_end_matches(';').trim().to_string();
        }
    }
    for line in body {
        if let Some(idx) = line.trim().find("@return ") {
            let after = &line.trim()[idx + 8..];
            let value: String = after.chars().take_while(|c| *c != ';' && *c != '}').collect();
            let value = value.trim();
            if !value.is_empty() {
                return value.to_string();
            }
        }
    }
    String::new()
}

fn count_braces(line: &str) -> i32 {
    line.chars().fold(0, |d, c| match c {
        '{' => d + 1,
        '}' => d - 1,
        _ => d,
    })
}

// ─── 命名空间化输出 ──────────────────────────────────────────────────────

pub fn build_ns_map(members: &ModuleMembers, alias: &str) -> NsMap {
    let prefix = format!("{alias}_");
    let mut ns = NsMap::default();

    for (name, _, _) in &members.variables {
        let new_name = format!("${prefix}{}", &name[1..]);
        ns.var_map.insert(name.clone(), new_name);
    }
    for (name, _, _) in &members.functions {
        ns.fn_map.insert(name.clone(), format!("{prefix}{name}"));
    }
    for (name, _, _) in &members.mixins {
        ns.mixin_map.insert(name.clone(), format!("{prefix}{name}"));
    }

    ns
}

/// 输出命名空间化变量/函数/mixin/raw_rules 为文本 (注入主管线)
pub fn emit_namespaced_members(members: &ModuleMembers, alias: &str) -> String {
    let mut out = String::new();
    let prefix = format!("{alias}_");

    for (name, value, _) in &members.variables {
        let stripped_name = &name[1..];
        out.push_str(&format!("${prefix}{stripped_name}: {value};\n"));
    }

    for (name, params, return_val) in &members.functions {
        let new_name = format!("{prefix}{name}");
        let p = if params.is_empty() {
            String::new()
        } else {
            format!("({})", params.join(", "))
        };
        let subbed_return = substitute_vars_in_line(return_val, alias, members);
        out.push_str(&format!("@function {new_name}{p} {{ @return {subbed_return}; }}\n"));
    }

    for (name, params, body) in &members.mixins {
        let new_name = format!("{prefix}{name}");
        let p = if params.is_empty() {
            String::new()
        } else {
            format!("({})", params.join(", "))
        };
        out.push_str(&format!("@mixin {new_name}{p} {{\n"));
        for bline in body {
            let subbed = substitute_vars_in_line(bline, alias, members);
            out.push_str(&format!("{subbed}\n"));
        }
        out.push_str("}\n");
    }

    for rule in &members.raw_rules {
        let subbed = substitute_vars_in_line(rule, alias, members);
        out.push_str(&format!("{subbed}\n"));
    }

    out
}

fn substitute_vars_in_line(text: &str, alias: &str, members: &ModuleMembers) -> String {
    let prefix = format!("{alias}_");
    let mut result = text.to_string();
    for (var_name, _, _) in &members.variables {
        result = replace_var_ref(&result, &var_name, &format!("${prefix}{}", &var_name[1..]));
    }
    result
}

fn replace_var_ref(text: &str, old_var: &str, new_var: &str) -> String {
    let old_bytes = old_var.as_bytes();
    let text_bytes = text.as_bytes();
    let mut result = String::with_capacity(text.len());
    let mut i = 0;
    while i < text_bytes.len() {
        if text_bytes[i..].starts_with(old_bytes) {
            let next_idx = i + old_bytes.len();
            if next_idx >= text_bytes.len()
                || (!text_bytes[next_idx].is_ascii_alphanumeric()
                    && text_bytes[next_idx] != b'_'
                    && text_bytes[next_idx] != b'-')
            {
                result.push_str(new_var);
                i = next_idx;
                continue;
            }
        }
        result.push(text_bytes[i] as char);
        i += 1;
    }
    result
}
