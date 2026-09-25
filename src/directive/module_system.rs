//! 模块系统 — @use/@forward 命名空间语义 (rxrust 对齐版)
//!
//! 采用迭代 BFS 展开 @use/@forward，避免递归函数调用。
//!
//! 流程: process_module_imports 用队列驱动 BFS 处理 @use/@forward,
//! member_parse 负责底层 var/fn/mixin 解析与命名空间化输出。

use std::collections::{HashMap, HashSet, VecDeque};

use crate::directive::member_parse::{build_ns_map, emit_namespaced_members};

#[doc(inline)]
pub use crate::directive::member_parse::parse_module_members;

// ─── 数据载体 ─────────────────────────────────────────────────────────────

#[derive(Default, Debug)]
pub struct ModuleMembers {
    pub variables: Vec<(String, String, bool)>,
    pub functions: Vec<(String, Vec<String>, String)>,
    pub mixins: Vec<(String, Vec<String>, Vec<String>)>,
    pub raw_rules: Vec<String>,
}

#[derive(Default, Debug)]
pub struct NsMap {
    pub var_map: HashMap<String, String>,
    pub fn_map: HashMap<String, String>,
    pub mixin_map: HashMap<String, String>,
}

// ─── 主入口: 迭代 BFS 展开 ─────────────────────────────────────────────────

pub fn process_module_imports(input: &str, files: &HashMap<String, String>) -> (String, String) {
    let _span = tracing::info_span!("module_imports", bytes = input.len()).entered();

    let mut loading = HashSet::new();
    let mut out_injections: Vec<String> = Vec::new();
    let mut out_main_lines: Vec<String> = Vec::new();
    let mut ns_maps: Vec<(String, NsMap)> = Vec::new();

    let mut queue: VecDeque<String> = input.lines().map(String::from).collect();

    while let Some(line) = queue.pop_front() {
        let trimmed = line.trim();

        if let Some(rest) = trimmed.strip_prefix("@use ") {
            let (url, has_as, config) = parse_use_line(rest);
            let actual_url = url.trim().trim_matches('"').trim().to_string();
            if !actual_url.is_empty() && !loading.contains(&actual_url) {
                loading.insert(actual_url.clone());
                let alias = if has_as {
                    extract_as_alias(rest).unwrap_or_else(|| ns_from_url(&actual_url))
                } else {
                    ns_from_url(&actual_url)
                };
                let content = resolve_file_path(&actual_url, files);
                let effective_content = apply_all_with(content, &config);
                let members = parse_module_members(&effective_content);

                expand_forwards_in_place(
                    &members.raw_rules,
                    &mut loading,
                    files,
                    &alias,
                    &mut out_injections,
                    &mut ns_maps,
                );

                let injection = emit_namespaced_members(&members, &alias);
                if !injection.trim().is_empty() {
                    out_injections.push(injection);
                }
                let ns = build_ns_map(&members, &alias);
                ns_maps.push((alias, ns));
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("@forward ") {
            let url = extract_url(rest);
            let actual_url = url.trim().trim_matches('"').trim().to_string();
            if !actual_url.is_empty() && !loading.contains(&actual_url) {
                loading.insert(actual_url.clone());
                let content = resolve_file_path(&actual_url, files);
                let members = parse_module_members(content);
                let up_ns = ns_from_url(&actual_url);

                let injection = emit_namespaced_members(&members, &up_ns);
                if !injection.trim().is_empty() {
                    out_injections.push(injection);
                }

                let parent_ns = up_ns.clone();
                expand_forwards_in_place(
                    &members.raw_rules,
                    &mut loading,
                    files,
                    &parent_ns,
                    &mut out_injections,
                    &mut ns_maps,
                );

                let up_ns_map = build_ns_map(&members, &up_ns);
                let mut down_map = NsMap::default();
                for (old_var, new_var) in &up_ns_map.var_map {
                    down_map.var_map.insert(old_var.clone(), new_var.clone());
                }
                for (old_fn, new_fn) in &up_ns_map.fn_map {
                    down_map.fn_map.insert(old_fn.clone(), new_fn.clone());
                }
                for (old_mx, new_mx) in &up_ns_map.mixin_map {
                    down_map.mixin_map.insert(old_mx.clone(), new_mx.clone());
                }
                ns_maps.push((up_ns, down_map));
            }
            out_main_lines.push(line);
            continue;
        }

        out_main_lines.push(apply_ns_subs(&line, &ns_maps));
    }

    (out_injections.join("\n"), out_main_lines.join("\n"))
}

// ─── forward 链展开 (迭代 BFS) ─────────────────────────────────────────────

fn expand_forwards_in_place(
    raw_rules: &[String],
    loading: &mut HashSet<String>,
    files: &HashMap<String, String>,
    parent_alias: &str,
    out_injections: &mut Vec<String>,
    ns_maps: &mut Vec<(String, NsMap)>,
) {
    let mut fwd_queue: VecDeque<String> = raw_rules
        .iter()
        .filter(|l| l.trim().starts_with("@forward "))
        .cloned()
        .collect();

    while let Some(fwd_line) = fwd_queue.pop_front() {
        let rest = match fwd_line.trim().strip_prefix("@forward ") {
            Some(r) => r,
            None => continue,
        };
        let url = extract_url(rest);
        let actual_url = url.trim().trim_matches('"').trim().to_string();
        if actual_url.is_empty() || loading.contains(&actual_url) {
            continue;
        }
        loading.insert(actual_url.clone());

        let content = resolve_file_path(&actual_url, files);
        let members = parse_module_members(content);
        let up_ns = ns_from_url(&actual_url);

        let injection = emit_namespaced_members(&members, &up_ns);
        if !injection.trim().is_empty() {
            out_injections.push(injection);
        }

        for sub_fwd in &members.raw_rules {
            if sub_fwd.trim().starts_with("@forward ") {
                fwd_queue.push_back(sub_fwd.clone());
            }
        }

        let up_ns_map = build_ns_map(&members, &up_ns);
        let mut alias_map = NsMap::default();
        for (old_var, new_var) in &up_ns_map.var_map {
            alias_map.var_map.insert(old_var.clone(), new_var.clone());
        }
        for (old_fn, new_fn) in &up_ns_map.fn_map {
            alias_map.fn_map.insert(old_fn.clone(), new_fn.clone());
        }
        for (old_mx, new_mx) in &up_ns_map.mixin_map {
            alias_map.mixin_map.insert(old_mx.clone(), new_mx.clone());
        }
        ns_maps.push((parent_alias.to_string(), alias_map));
    }
}

// ─── 文本替换 ─────────────────────────────────────────────────────────────

fn apply_ns_subs(line: &str, ns_maps: &[(String, NsMap)]) -> String {
    let mut result = line.to_string();
    for (alias, ns) in ns_maps {
        for (old, new) in &ns.var_map {
            result = result.replace(&format!("{alias}.{old}"), new);
            result = result.replace(&format!("{alias}.{old}:"), &format!("{new}:"));
        }
        for (old, new) in &ns.fn_map {
            result = result.replace(&format!("{alias}.{old}("), &format!("{new}("));
        }
        for (old, new) in &ns.mixin_map {
            result = result.replace(&format!("@include {alias}.{old}"), &format!("@include {new}"));
        }
    }
    result
}

// ─── @use 行解析 ───────────────────────────────────────────────────────────

fn parse_use_line(rest: &str) -> (String, bool, Vec<(String, String)>) {
    let url = extract_url(rest);
    let has_as = rest.contains(" as ");
    let config = if let Some(idx) = rest.find(" with ") {
        let after_with = &rest[idx + 5..];
        parse_with_config(after_with)
    } else {
        Vec::new()
    };
    (url, has_as, config)
}

fn parse_with_config(s: &str) -> Vec<(String, String)> {
    let mut config = Vec::new();
    if let Some(start) = s.find('(') {
        if let Some(end) = s[start..].find(')') {
            let inner = &s[start + 1..start + end];
            for pair in inner.split(',') {
                let pair = pair.trim();
                if pair.is_empty() {
                    continue;
                }
                if let Some((name, value)) = pair.split_once(':') {
                    let bare_name = name.trim().trim_start_matches('$').to_string();
                    let value = value.trim().to_string();
                    if !bare_name.is_empty() && !value.is_empty() {
                        config.push((bare_name, value));
                    }
                }
            }
        }
    }
    config
}

fn apply_all_with(content: &str, config: &[(String, String)]) -> String {
    let mut result = content.to_string();
    for (var_name, val) in config {
        result = apply_with_override(&result, var_name, val);
    }
    result
}

fn apply_with_override(content: &str, var_name: &str, override_val: &str) -> String {
    let bare_name = var_name.trim_start_matches('$');
    content
        .lines()
        .map(|line: &str| {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix('$') {
                if let Some((name, _)) = rest.split_once(':') {
                    if name.trim() == bare_name && trimmed.contains("!default") {
                        return format!("${bare_name}: {override_val};");
                    }
                }
            }
            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn extract_as_alias(rest: &str) -> Option<String> {
    let pos = rest.find(" as ")?;
    let alias = rest[pos + 4..].trim().trim_end_matches(';').trim().to_string();
    if alias.is_empty() {
        None
    } else {
        Some(alias)
    }
}

// ─── URL / 路径解析 ─────────────────────────────────────────────────────────

pub fn ns_from_url(url: &str) -> String {
    let path = if let Some(start) = url.find('"') {
        if let Some(end) = url[start + 1..].find('"') {
            &url[start + 1..start + 1 + end]
        } else {
            url
        }
    } else if url.starts_with('"') && url.ends_with('"') {
        &url[1..url.len() - 1]
    } else {
        url
    };

    let basename = path.rsplit('/').next().unwrap_or(path);
    let no_ext = if let Some(dot) = basename.rfind('.') {
        &basename[..dot]
    } else {
        basename
    };
    no_ext.trim_start_matches('_').to_string()
}

fn extract_url(s: &str) -> String {
    let s = s.trim();
    if let Some(start) = s.find('"') {
        if let Some(end) = s[start + 1..].find('"') {
            return s[start + 1..start + 1 + end].to_string();
        }
    }
    s.to_string()
}

pub fn resolve_file_path<'a>(path: &str, files: &'a HashMap<String, String>) -> &'a str {
    if let Some(c) = files.get(path) {
        return c;
    }
    let under = format!("_{path}");
    if let Some(c) = files.get(&under) {
        return c;
    }
    let scss = format!("{path}.scss");
    if let Some(c) = files.get(&scss) {
        return c;
    }
    let under_scss = format!("_{path}.scss");
    if let Some(c) = files.get(&under_scss) {
        return c;
    }
    let sass = format!("{path}.sass");
    if let Some(c) = files.get(&sass) {
        return c;
    }
    let under_sass = format!("_{path}.sass");
    if let Some(c) = files.get(&under_sass) {
        return c;
    }
    if let Some(c) = files.get(&format!("{path}/_index.scss")) {
        return c;
    }
    if let Some(c) = files.get(&format!("{path}/index.scss")) {
        return c;
    }
    if let Some(c) = files.get(&format!("{path}/_index.sass")) {
        return c;
    }
    files
        .get(&format!("{path}/index.sass"))
        .map(|c| c.as_str())
        .unwrap_or("")
}
