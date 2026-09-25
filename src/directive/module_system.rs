//! 模块系统 — @use / @forward 命名空间语义
//!
//! 设计:
//!   - `@use "path"` 抽出模块内容, 重命名后注入主管线
//!   - 重命名规则: `$member` → `$ns_member`, `@function foo` → `@function ns_foo`
//!   - 主文件引用 `other.$member` → `$other_member` (纯文本替换)
//!   - `with()` 配置: 模块内 `$var: original !default` 被覆盖为 `$ns_var: override_val;` 前缀
//!   - 变量赋值 `other.$member: new value` → `$other_member: new value;` (即时生效)

use std::collections::{HashMap, HashSet};

// ─── 模块成员解析 ─────────────────────────────────────────────────────────

/// 模块内析出的顶级成员集合
#[derive(Default, Debug)]
pub struct ModuleMembers {
    pub variables: Vec<(String, String, bool)>, // (name, value, is_default)
    pub functions: Vec<(String, Vec<String>, String)>, // (name, params, return_value)
    pub mixins: Vec<(String, Vec<String>, Vec<String>)>, // (name, params, body)
    pub raw_rules: Vec<String>,                 // 无法归类的顶层 CSS 行
}

/// 扫描模块头部的顶级定义 (递归1层 @use/@forward)
/// 返回: (成员集合, 顶层 CSS 规则行)
#[allow(clippy::redundant_clone)]
pub fn parse_module_members(content: &str) -> ModuleMembers {
    let mut members = ModuleMembers::default();
    let mut current_block: Option<(String, Vec<String>)> = None; // (kind+name, body)
    let mut brace_depth = 0i32;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // 在未闭合的 block 内
        if let Some((_, ref mut body)) = current_block {
            body.push(line.to_string());
            brace_depth += count_braces(line);
            if brace_depth <= 0 {
                if let Some((head, body)) = current_block.take() {
                    if head.starts_with("fn ") {
                        let name = head[3..].trim().to_string();
                        let params = extract_fn_params(&name);
                        let return_value = extract_return_value(&body);
                        let name_only = name.split('(').next().unwrap_or(&name).trim().to_string();
                        members.functions.push((name_only, params, return_value));
                    } else if head.starts_with("mx ") {
                        let name = head[3..].trim().to_string();
                        let params = extract_fn_params(&name);
                        let name_only = name.split('(').next().unwrap_or(&name).trim().to_string();
                        members.mixins.push((name_only, params, body));
                    }
                }
                brace_depth = 0;
            }
            continue;
        }

        // 变量定义
        if let Some((name, value, is_default)) = try_extract_var(trimmed) {
            members.variables.push((name, value, is_default));
            continue;
        }

        // 函数定义
        if let Some(rest) = trimmed.strip_prefix("@function ") {
            let (name_paren, body_open) = split_fn_head(rest);
            if body_open {
                // 单行: @function name() { @return val; }
                let name = name_paren.trim().to_string();
                let params = extract_fn_params(&name);
                let name_only = name.split('(').next().unwrap_or(&name).trim().to_string();
                let return_value = extract_return_value(&[rest.to_string()]);
                members.functions.push((name_only, params, return_value));
            } else {
                let name = name_paren.trim().to_string();
                current_block = Some((format!("fn {name}"), vec![]));
                brace_depth = count_braces(line);
                if brace_depth <= 0 {
                    current_block = None;
                    brace_depth = 0;
                }
            }
            continue;
        }

        // mixin 定义
        if let Some(rest) = trimmed.strip_prefix("@mixin ") {
            let (name_paren, body_open) = split_mixin_head(rest);
            if body_open {
                let name = name_paren.trim().to_string();
                let params = extract_mixin_params(&name);
                let name_only = name.split('(').next().unwrap_or(&name).trim().to_string();
                // 提取 { ... } 之间的内容（去掉 mixin 名称和外层括号）
                let body_inner = if let Some(open) = rest.find('{') {
                    if let Some(close) = rest.rfind('}') {
                        rest[open + 1..close].trim().to_string()
                    } else {
                        rest.to_string()
                    }
                } else {
                    rest.to_string()
                };
                members.mixins.push((name_only, params, vec![body_inner]));
            } else {
                let name = name_paren.trim().to_string();
                current_block = Some((format!("mx {name}"), vec![]));
                brace_depth = count_braces(line);
                if brace_depth <= 0 {
                    current_block = None;
                    brace_depth = 0;
                }
            }
            continue;
        }

        // @include — 可能被 @use 调用
        if trimmed.starts_with("@include ") {
            members.raw_rules.push(line.to_string());
            continue;
        }

        // @return — 函数里已处理
        if trimmed.starts_with("@return ") {
            continue;
        }

        // 其他行 (CSS 选择器规则 / @import / @charset / 等)
        members.raw_rules.push(line.to_string());
    }

    members
}

/// 模块重写结果: 注入主管线的文本 + 命名空间替换表
#[derive(Debug)]
pub struct RewrittenModule {
    /// 注入主管线的内容 (已重命名)
    pub injection: String,
    /// 主文件替换表: 原词 → 新词
    pub var_map: HashMap<String, String>,    // "$member" → "$ns_member"
    pub fn_map: HashMap<String, String>,     // "foo" → "ns_foo"
    pub mixin_map: HashMap<String, String>,  // "bar" → "ns_bar"
}

/// 重写整个模块为带命名空间的版本
///
/// - variables: `$member: value` → `$ns_member: value` (并收集替换表)
/// - functions: `@function foo() {...}` → `@function ns_foo() {...}`
/// - mixins: `@mixin bar {...}` → `@mixin ns_bar {...}`
/// - raw_rules: 原样 emit
#[allow(clippy::redundant_clone)]
pub fn rewrite_module(
    members: &ModuleMembers,
    ns: &str,
    config: &[(String, String)],
) -> RewrittenModule {
    let mut injection = String::new();
    let mut var_map = HashMap::new();
    let mut fn_map = HashMap::new();
    let mut mixin_map = HashMap::new();
    let prefix = format!("{ns}_");

    // 1) 变量声明 (with() 覆盖的变量已经被替换成 override value)
    // 注入主管线时保留 $ 前缀，否则主管线无法识别为变量声明
    for (name, value, _is_default) in &members.variables {
        let new_name = format!("${prefix}{}", &name[1..]); // 保留 $ 前缀
        var_map.insert(name.clone(), new_name.clone());
        injection.push_str(&format!("{new_name}: {value};\n"));
    }

    // 2) 函数定义
    for (name, params, return_value) in &members.functions {
        let new_name = format!("{prefix}{name}");
        fn_map.insert(name.clone(), new_name.clone());

        let params_str = if params.is_empty() {
            String::new()
        } else {
            format!("({})", params.join(", "))
        };
        // 函数内引用其他成员: 替换变量名
        let rewritten_return = apply_subs(return_value, &var_map, &fn_map, &mixin_map);
        injection.push_str(&format!(
            "@function {new_name}{params_str} {{ @return {rewritten_return}; }}\n"
        ));
    }

    // 3) mixin 定义
    for (name, params, body) in &members.mixins {
        let new_name = format!("{prefix}{name}");
        mixin_map.insert(name.clone(), new_name.clone());

        let params_str = if params.is_empty() {
            String::new()
        } else {
            format!("({})", params.join(", "))
        };
        injection.push_str(&format!("@mixin {new_name}{params_str} {{\n"));
        for line in body {
            let rewritten = apply_subs(line, &var_map, &fn_map, &mixin_map);
            injection.push_str(&format!("{rewritten}\n"));
        }
        injection.push_str("}\n");
    }

    // 4) 顶层 CSS 规则 — 直接 emit (apply subs on the way)
    for line in &members.raw_rules {
        let rewritten = apply_subs(line, &var_map, &fn_map, &mixin_map);
        injection.push_str(&format!("{rewritten}\n"));
    }

    RewrittenModule {
        injection,
        var_map,
        fn_map,
        mixin_map,
    }
}

/// 对单行内应用替换表
fn apply_subs(
    line: &str,
    var_map: &HashMap<String, String>,
    fn_map: &HashMap<String, String>,
    mixin_map: &HashMap<String, String>,
) -> String {
    let mut result = line.to_string();
    // 先替换函数名 (长匹配优先), 再替换变量
    for (old, new) in fn_map {
        result = result.replace(&format!("{old}("), &format!("{new}("));
    }
    for (old, new) in mixin_map {
        // @include old → @include new (只匹配单词边界)
        result = result.replace(&format!("@include {old}"), &format!("@include {new}"));
    }
    for (old, new) in var_map {
        result = result.replace(old, new);
    }
    result
}

// ─── 命名空间解析 (从 URL → 短名) ─────────────────────────────────────────

/// 从 URL 取命名空间 (basename 去掉扩展名和前导下划线)
/// "foo/bar/baz/other" → "other"
/// "_other" → "other"
/// "other.scss" → "other"
pub fn ns_from_url(url: &str) -> String {
    let path = if url.starts_with('"') && url.ends_with('"') {
        &url[1..url.len() - 1]
    } else if let Some(start) = url.find('"') {
        if let Some(end) = url[start + 1..].find('"') {
            &url[start + 1..start + 1 + end]
        } else {
            url
        }
    } else {
        url
    };

    let basename = path.rsplit('/').next().unwrap_or(path);
    // 去扩展名
    let no_ext = if let Some(dot) = basename.rfind('.') {
        &basename[..dot]
    } else {
        basename
    };
    // 去前导下划线
    no_ext.trim_start_matches('_').to_string()
}

// ─── 主文件替换入口 ─────────────────────────────────────────────────────────

/// 在主管线前扫描 @use / @forward / @as, 处理模块路径, 重写主文件
///
/// 返回: (注入主管线的完整文本, 重命名后的主文件内容)
pub fn process_module_imports(
    input: &str,
    files: &HashMap<String, String>,
    loading: &mut HashSet<String>,
) -> (String, String) {
    let mut injections: Vec<String> = Vec::new();
    let mut main_lines: Vec<String> = Vec::new();
    let mut ns_maps: Vec<(RewrittenModule, String)> = Vec::new(); // (rewritten, raw_ns)

    let mut pending: Option<(bool /* is_forward */, bool /* has_as */, String /* url_or_acc */)> = None;
    let mut in_as_clause = false;

    for line in input.lines() {
        let trimmed = line.trim();

        // 跨行累积
        if let Some((is_forward, has_as, acc)) = pending.clone() {
            let mut acc = acc;
            acc.push(' ');
            acc.push_str(trimmed);
            // 平衡逻辑: 括号数相等且出现过 ')'
            let total_open: usize = acc.chars().filter(|c| *c == '(').count();
            let total_close: usize = acc.chars().filter(|c| *c == ')').count();
            if total_close >= total_open && total_close > 0 {
                flush_pending_module(&acc, is_forward, has_as, files, loading, &mut injections, &mut ns_maps);
                pending = None;
            } else {
                pending = Some((is_forward, has_as, acc));
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("@use ") {
            // 括号平衡判断基于整行 rest（包含 with 子句的括号）
            let open_count = rest.chars().filter(|c| *c == '(').count();
            let close_count = rest.chars().filter(|c| *c == ')').count();
            let has_as = rest.contains(" as ");
            if open_count > close_count {
                // 跨行累积：保留完整 rest（含 with 子句）
                pending = Some((false, has_as, rest.to_string()));
            } else {
                // 单行完成：传入完整 rest 以保留 with 子句
                flush_pending_module(rest, false, has_as, files, loading, &mut injections, &mut ns_maps);
            }
            in_as_clause = has_as;
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("@forward ") {
            let open_count = rest.chars().filter(|c| *c == '(').count();
            let close_count = rest.chars().filter(|c| *c == ')').count();
            if open_count > close_count {
                pending = Some((true, false, rest.to_string()));
            } else {
                flush_pending_module(rest, true, false, files, loading, &mut injections, &mut ns_maps);
            }
            continue;
        }

        main_lines.push(line.to_string());
    }

    // 处理 @as alias (在主文件里)
    // 对于 `other.$x` 在主文件的引用, 做纯文本替换
    let mut main_text = main_lines.join("\n");
    for (rewritten, raw_ns) in &ns_maps {
        let alias = raw_ns.clone();
        // var access: alias.$var → $ns_var (其中 ns 模块文件名优化)
        for (old_var, new_var) in &rewritten.var_map {
            let old_ref = format!("{alias}.{old_var}");
            main_text = main_text.replace(&old_ref, new_var);
        }
        // var assignment: `alias.$var: ...` → `$ns_var: ...`
        for (old_var, new_var) in &rewritten.var_map {
            let old_assign = format!("{alias}.{old_var}:");
            let new_assign = format!("{new_var}:");
            main_text = main_text.replace(&old_assign, &new_assign);
        }
        // fn call: alias.foo(...) → ns_foo(...)
        for (old_fn, new_fn) in &rewritten.fn_map {
            let old_call = format!("{alias}.{old_fn}(");
            let new_call = format!("{new_fn}(");
            main_text = main_text.replace(&old_call, &new_call);
        }
        // mixin include: @include alias.foo → @include ns_foo
        for (old_mx, new_mx) in &rewritten.mixin_map {
            let old_inc = format!("@include {alias}.{old_mx}");
            let new_inc = format!("@include {new_mx}");
            main_text = main_text.replace(&old_inc, &new_inc);
        }
    }

    // 拼接 injection (去除空模块的 empty CSS)
    let injection = injections.join("\n");
    (injection, main_text)
}

#[allow(clippy::too_many_arguments)]
fn flush_pending_module(
    url_or_acc: &str,
    is_forward: bool,
    has_as: bool,
    files: &HashMap<String, String>,
    loading: &mut HashSet<String>,
    injections: &mut Vec<String>,
    ns_maps: &mut Vec<(RewrittenModule, String)>,
) {
    let _span = tracing::info_span!("flush_module", url = %url_or_acc, is_forward).entered();

    let (url_raw, config) = if is_forward {
        (extract_url(url_or_acc), Vec::new())
    } else {
        parse_use_with_url(url_or_acc)
    };

    let actual_url = url_raw.trim().trim_matches('"').trim().to_string();
    let ns = ns_from_url(&actual_url);

    if loading.contains(&actual_url) {
        return;
    }
    loading.insert(actual_url.clone());

    // 加载文件
    let content = match resolve_file_path(&actual_url, files) {
        Some(c) => c,
        None => {
            // 找不到文件时 fallback: 原样 emit (尝试保留)
            return;
        }
    };

    // 应用 with() 配置: 覆盖 module 内的 !default 变量
    let mut effective_content = content.clone();
    for (var_name, val) in &config {
        effective_content = apply_with_override(&effective_content, var_name, val);
    }

    // 成员解析 (在 forward 合并之前，先解析当前文件的成员)
    let members = parse_module_members(&effective_content);
    let mut rewritten = rewrite_module(&members, &ns, &[]);

    // 递归处理文件内部的 @forward / @use 指令（模块系统依赖）
    // 注意: 无论当前入口是 @use 还是 @forward，只要被引用的模块内部有 @forward，
    // 就需要递归加载上游，并把上游成员合并到当前 ns_map 中。
    let mut upstream_injections = Vec::new();
    let mut upstream_ns_maps = Vec::new();
    for line in content.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("@forward ") {
            // 递归加载 forward 引用的文件 (is_forward=true)
            flush_pending_module(rest, true, false, files, loading, &mut upstream_injections, &mut upstream_ns_maps);
        } else if let Some(rest) = t.strip_prefix("@use ") {
            // 内部 @use 的模块也递归加载
            flush_pending_module(rest, false, rest.contains(" as "), files, loading, &mut upstream_injections, &mut upstream_ns_maps);
        }
    }
    if !upstream_injections.is_empty() {
        // upstream 的注入排在前面（依赖顺序）
        injections.extend(upstream_injections);

        // 上游的 var/fn/mixin map 合并到当前 rewritten
        // 这样主文件 `midstream.$c` 能映射到 upstream 的变量
        for (up_rewritten, _up_alias) in &upstream_ns_maps {
            for (old_var, new_var) in &up_rewritten.var_map {
                rewritten.var_map.insert(old_var.clone(), new_var.clone());
            }
            for (old_fn, new_fn) in &up_rewritten.fn_map {
                rewritten.fn_map.insert(old_fn.clone(), new_fn.clone());
            }
            for (old_mx, new_mx) in &up_rewritten.mixin_map {
                rewritten.mixin_map.insert(old_mx.clone(), new_mx.clone());
            }
        }
    }

    // forward 与 use 的差别:
    //   forward: 模块 CSS rules 都应 emit, 变量/fn/mixin 都应公开
    //   use: 只有主文件通过 ns.xxx 引用的才调用; 变量/fn/mixin 注册, CSS rules 也 emit
    // 当前 sasspile 简化: 两者行为相同 (CSS rules + namespaced members 均注入)
    if !rewritten.injection.trim().is_empty() {
        injections.push(rewritten.injection.clone());
    }

    let alias = if has_as {
        // @use "x" as foo — 用 foo 作主文件引用名
        trim_as_alias(url_or_acc)
    } else {
        ns.clone()
    };
    ns_maps.push((rewritten, alias));
}

/// 从 "@use "foo" with ($x: val)" 类字符串中提取 as 后的别名字符串
fn trim_as_alias(s: &str) -> String {
    if let Some(pos) = s.find(" as ") {
        s[pos + 4..].trim().trim_end_matches(';').trim().to_string()
    } else {
        ns_from_url(s)
    }
}

/// 应用 with() 覆盖: 找到 `$var: ... !default;` 替换为 `$var: override_val;`
fn apply_with_override(content: &str, var_name: &str, override_val: &str) -> String {
    // var_name 可能带 $ 前缀，统一去掉
    let bare_name = var_name.trim_start_matches('$');
    content
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix('$') {
                if let Some((name, _)) = rest.split_once(':') {
                    let trimmed_name = name.trim();
                    if trimmed_name == bare_name && trimmed.contains("!default") {
                        return format!("${bare_name}: {override_val};");
                    }
                }
            }
            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ─── URL 解析辅助 ─────────────────────────────────────────────────────────

fn parse_use_with_url(s: &str) -> (String, Vec<(String, String)>) {
    let url = extract_url(s);
    let mut config = Vec::new();
    if let Some(with_start) = s.find("with") {
        let after_with = &s[with_start + 4..];
        if let Some(p_start) = after_with.find('(') {
            if let Some(p_end) = after_with.rfind(')') {
                let params = &after_with[p_start + 1..p_end];
                for pair in params.split(',') {
                    let pair = pair.trim();
                    if pair.is_empty() {
                        continue;
                    }
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
    (url, config)
}

fn parse_forward_url(s: &str) -> String {
    extract_url(s)
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

fn split_use_url(rest: &str) -> (String, bool) {
    let url = extract_url(rest);
    let has_as = rest.contains(" as ");
    (url, has_as)
}

// ─── 文件路径解析 ─────────────────────────────────────────────────────────

/// 尝试: 精确 / 加下划线 / 加扩展名 / index文件
pub fn resolve_file_path<'a>(
    path: &str,
    files: &'a HashMap<String, String>,
) -> Option<&'a String> {
    if let Some(c) = files.get(path) {
        return Some(c);
    }
    let under = format!("_{path}");
    if let Some(c) = files.get(&under) {
        return Some(c);
    }
    let scss = format!("{path}.scss");
    if let Some(c) = files.get(&scss) {
        return Some(c);
    }
    let under_scss = format!("_{path}.scss");
    if let Some(c) = files.get(&under_scss) {
        return Some(c);
    }
    let sass = format!("{path}.sass");
    if let Some(c) = files.get(&sass) {
        return Some(c);
    }
    let under_sass = format!("_{path}.sass");
    if let Some(c) = files.get(&under_sass) {
        return Some(c);
    }
    // index 文件
    let idx1 = format!("{path}/_index.scss");
    if let Some(c) = files.get(&idx1) {
        return Some(c);
    }
    let idx2 = format!("{path}/index.scss");
    if let Some(c) = files.get(&idx2) {
        return Some(c);
    }
    let idx3 = format!("{path}/_index.sass");
    if let Some(c) = files.get(&idx3) {
        return Some(c);
    }
    let idx4 = format!("{path}/index.sass");
    files.get(&idx4)
}

// ─── 行解析辅助 ─────────────────────────────────────────────────────────

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

fn split_fn_head(rest: &str) -> (String, bool) {
    // "name() { @return ... }" or "name() {...}" or "name() {"
    if let Some(paren_start) = rest.find('(') {
        let name = rest[..paren_start].trim().to_string();
        let after_paren = &rest[paren_start..];
        let has_brace = after_paren.contains('{');
        (format!("{name}({after_paren}"), has_brace)
    } else {
        // 无参: "name {" or "name { @return x }"
        let name = rest.split_whitespace().next().unwrap_or(rest).trim().to_string();
        let has_brace = rest.contains('{');
        (name, has_brace)
    }
}

fn split_mixin_head(rest: &str) -> (String, bool) {
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

fn extract_fn_params(s: &str) -> Vec<String> {
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

fn extract_mixin_params(s: &str) -> Vec<String> {
    extract_fn_params(s)
}

fn extract_return_value(body: &[String]) -> String {
    // 先从行首匹配 @return (跨行函数)
    for line in body {
        let trimmed = line.trim();
        if let Some(after_return) = trimmed.strip_prefix("@return ") {
            return after_return.trim().trim_end_matches(';').trim().to_string();
        }
    }
    // 单行函数: 搜索函数体内任意位置的 @return 关键字
    // 格式: @function name() { @return value } 或 name() { @return value }
    for line in body {
        let trimmed = line.trim();
        if let Some(idx) = trimmed.find("@return ") {
            let after = &trimmed[idx + 8..];
            // 提取值：到 ; } 或行尾为止
            let value: String = after
                .chars()
                .take_while(|c| *c != ';' && *c != '}')
                .collect();
            let value = value.trim();
            if !value.is_empty() {
                return value.to_string();
            }
        }
    }
    String::new()
}

fn count_braces(line: &str) -> i32 {
    line.chars()
        .fold(0, |d, c| match c {
            '{' => d + 1,
            '}' => d - 1,
            _ => d,
        })
}
