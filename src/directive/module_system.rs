//! 模块系统 — @use/@forward 命名空间语义 (rxrust 对齐版)
//!
//! 约束: 遵循 `specs/module-system-rx/spec.md` — 模块加载须为不可变数据流,
//!       禁止跨文件共享可变状态 (Arc<Mutex> 模式), 中间产物零 clone。
//!
//! 采用迭代 BFS 展开 @use/@forward，避免递归函数调用。
//!
//! 流程: process_module_imports 用队列驱动 BFS 处理 @use/@forward,
//! member_parse 负责底层 var/fn/mixin 解析与命名空间化输出。

use std::collections::{HashMap, HashSet, VecDeque};

use crate::directive::import_resolver::resolve_imports;
use crate::directive::member_parse::{build_forward_use_site_map, build_ns_map, emit_namespaced_members};

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

// TODO: 按 specs/module-system-rx/spec.md 重构为 ModuleEvent Subject + scan_map(ModuleResolver)
//       当前 BFS 模式在共享 files HashMap 上有隐式状态依赖, 应改为不可变数据流驱动的解析管线
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
            let raw_url = url.trim().trim_matches(|c| c == '"' || c == '\'').trim();
            let actual_url = normalize_path(raw_url);
            if !actual_url.is_empty() && !loading.contains(&actual_url) {
                loading.insert(actual_url.clone());
                let alias = if has_as {
                    extract_as_alias(rest).unwrap_or_else(|| ns_from_url(&actual_url))
                } else {
                    ns_from_url(&actual_url)
                };
                let raw = resolve_file_path(&actual_url, files);
                let content = resolve_imports(raw, files);
                let effective_content = apply_all_with(&content, &config);
                let members = parse_module_members(&effective_content);

                expand_forwards_in_place(
                    &members.raw_rules,
                    &mut loading,
                    files,
                    &alias,
                    &mut out_injections,
                    &mut ns_maps,
                );

                let is_global = alias == "*";
                let injection = if is_global {
                    // @use "url" as *: 成员直接注入全局作用域 (无前缀)
                    emit_namespaced_members(&members, "")
                } else {
                    emit_namespaced_members(&members, &alias)
                };
                if !injection.trim().is_empty() {
                    out_injections.push(injection);
                }
                if !is_global {
                    let ns = build_ns_map(&members, &alias);
                    ns_maps.push((alias, ns));
                }
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("@forward ") {
            let url = extract_url(rest);
            let raw_url = url.trim().trim_matches(|c| c == '"' || c == '\'').trim();
            let actual_url = normalize_path(raw_url);
            // 解析 @forward "url" as prefix-* 中的可选前缀
            let as_prefix = extract_forward_as_prefix(rest);
            if !actual_url.is_empty() && !loading.contains(&actual_url) {
                loading.insert(actual_url.clone());
                let raw = resolve_file_path(&actual_url, files);
                let content = resolve_imports(raw, files);
                let members = parse_module_members(&content);

                // 使用 as 前缀（如果有）或 URL 派生的命名空间
                let up_ns = as_prefix.clone().unwrap_or_else(|| ns_from_url(&actual_url));

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

                // @forward as prefix-* 场景需要反向映射：
                // 注入的成员已带前缀 (如 $d-c), 用户也写带前缀名称
                let up_ns_map = if as_prefix.is_some() {
                    build_forward_use_site_map(&members, &up_ns)
                } else {
                    build_ns_map(&members, &up_ns)
                };
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
// TODO: 按 specs/module-system-rx/spec.md 重构为 ModuleEvent Subject + scan_map(ModuleResolver)
//       当前迭代 BFS 模式通过 &mut HashSet<String> loading 跟踪循环依赖, 应改为不可变数据流驱动的拓扑排序

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
        let raw_url = url.trim().trim_matches(|c| c == '"' || c == '\'').trim();
        let actual_url = normalize_path(raw_url);
        // 解析 @forward "url" as prefix-* 中的可选前缀
        let as_prefix = extract_forward_as_prefix(rest);
        if actual_url.is_empty() || loading.contains(&actual_url) {
            continue;
        }
        loading.insert(actual_url.clone());

        let raw = resolve_file_path(&actual_url, files);
        let content = resolve_imports(raw, files);
        let members = parse_module_members(&content);

        // 使用 as 前缀（如果有）或 parent_alias 来注入成员
        let effective_alias = if let Some(ref prefix) = as_prefix {
            prefix.clone()
        } else if parent_alias == "*" {
            String::new()
        } else {
            parent_alias.to_string()
        };
        let injection = emit_namespaced_members(&members, &effective_alias);
        if !injection.trim().is_empty() {
            out_injections.push(injection);
        }

        for sub_fwd in &members.raw_rules {
            if sub_fwd.trim().starts_with("@forward ") {
                fwd_queue.push_back(sub_fwd.clone());
            }
        }

        // ns_maps 的键使用 @use 的命名空间（parent_alias）
        //
        // @forward as prefix-* 场景：用户写 midstream.$d-c（带前缀名称）
        //   需要用 build_forward_use_site_map（反向映射 prefixed -> prefixed）
        //
        // 普通 @forward（无前缀）场景：用户写 midstream.$c（原始名称）
        //   需要用 build_ns_map（正向映射 original -> prefixed/injected）
        let up_ns_map = if as_prefix.is_some() {
            build_forward_use_site_map(&members, &effective_alias)
        } else {
            build_ns_map(&members, &effective_alias)
        };

        // 使用 parent_alias（@use 的命名空间）作为键
        if parent_alias == "*" || parent_alias.is_empty() {
            // 全局 @use "url" as *: 无需额外映射
        } else {
            ns_maps.push((parent_alias.to_string(), up_ns_map));
        }
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
    let mut overridden = false;
    content
        .lines()
        .map(|line: &str| {
            let trimmed = line.trim();
            if !overridden {
                if let Some(rest) = trimmed.strip_prefix('$') {
                    if let Some((name, _)) = rest.split_once(':') {
                        if name.trim() == bare_name {
                            overridden = true;
                            return format!("${bare_name}: {override_val};");
                        }
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

/// 规范化路径 — 解析 . 和 .. 段
fn normalize_path(path: &str) -> String {
    let mut stack: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                stack.pop();
            }
            other => stack.push(other),
        }
    }
    if stack.is_empty() {
        String::new()
    } else {
        stack.join("/")
    }
}

pub fn ns_from_url(url: &str) -> String {
    // 同时支持单引号和双引号 URL
    let path = if let Some(start) = url.find('"') {
        if let Some(end) = url[start + 1..].find('"') {
            &url[start + 1..start + 1 + end]
        } else {
            url
        }
    } else if let Some(start) = url.find('\'') {
        if let Some(end) = url[start + 1..].find('\'') {
            &url[start + 1..start + 1 + end]
        } else {
            url
        }
    } else if url.starts_with('"') && url.ends_with('"') {
        &url[1..url.len() - 1]
    } else if url.starts_with('\'') && url.ends_with('\'') {
        &url[1..url.len() - 1]
    } else {
        url
    };

    let basename = path.rsplit('/').next().unwrap_or(path);
    // Sass 规范: 所有扩展名都应丢弃 (第一个 . 之前的内容为命名空间)
    let no_ext = if let Some(dot) = basename.find('.') {
        &basename[..dot]
    } else {
        basename
    };
    no_ext.trim_start_matches('_').to_string()
}

fn extract_url(s: &str) -> String {
    let s = s.trim();
    // 优先匹配双引号
    if let Some(start) = s.find('"') {
        if let Some(end) = s[start + 1..].find('"') {
            return s[start + 1..start + 1 + end].to_string();
        }
    }
    // 再匹配单引号
    if let Some(start) = s.find('\'') {
        if let Some(end) = s[start + 1..].find('\'') {
            return s[start + 1..start + 1 + end].to_string();
        }
    }
    s.to_string()
}

/// 解析 @forward 行的可选 as prefix-* 前缀
///
/// 返回 Some(prefix) 如果存在 as 前缀, 否则返回 None
/// 返回的前缀包含分隔符 (如 "d-" 或 "d_")
/// 示例:
///   @forward "foo"              -> None
///   @forward "foo" as bar-*     -> Some("bar-")
///   @forward "foo" as bar_*     -> Some("bar_")
fn extract_forward_as_prefix(rest: &str) -> Option<String> {
    // 查找 "as " 关键字
    if let Some(after_url) = rest.find("as ") {
        let after_as = &rest[after_url + 3..];
        // 提取前缀部分 (空格或 ; 之前)
        let prefix_part = after_as.trim().split_whitespace().next().unwrap_or("");
        // 找到 -* 或 _* 的位置，保留分隔符
        let prefix = if let Some(star_idx) = prefix_part.find("-*") {
            prefix_part[..star_idx + 1].to_string() // 包含 "-"
        } else if let Some(star_idx) = prefix_part.find("_*") {
            prefix_part[..star_idx + 1].to_string() // 包含 "_"
        } else if let Some(star_idx) = prefix_part.find('*') {
            prefix_part[..star_idx].to_string()
        } else {
            prefix_part.trim_end_matches(';').trim().to_string()
        };
        let prefix = prefix.trim().to_string();
        if prefix.is_empty() {
            None
        } else {
            Some(prefix)
        }
    } else {
        None
    }
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
