//! @import 解析 — 文件内容注入 (Sass 遗留模块系统)
//!
//! 约束: 遵循 `specs/module-system-rx/spec.md` — 模块加载须为不可变数据流,
//!       禁止跨文件共享可变状态, 中间产物零 clone。
// TODO: 按 specs/module-system-rx/spec.md 重构 — 当前文件内 @import 的解析和注入
//       采用显式队列 + shared HashSet 模式, 应改为 ModuleEvent Subject + scan_map(ModuleResolver)
//!
//! 与 @use/@forward 的区别:
//!   - @import: 所有内容注入全局作用域 (无命名空间)
//!   - @use:    带命名空间前缀, 不污染全局
//!
//! 语义:
//!   1. @import "foo" 查找 _foo.scss / foo.scss / _foo.sass / foo.sass
//!   2. 被导入文件的内容替换 @import 行 (注入到当前位置)
//!   3. @forward 在 @import 链中展开为裸名成员 (全局作用域语义)
//!   4. !default 变量不覆盖已有定义 (由编译器 ops.rs 处理)
//!   5. 循环导入防护: 同文件不重复加载

use std::collections::{HashSet, VecDeque};

use tracing::info_span;

use super::member_parse::parse_module_members;
use super::member_parse::emit_namespaced_members;
use super::module_system::resolve_file_path;

/// 解析并展开所有 @import 行, 返回合并后的完整输入
///
/// 同时展开 @import 链中的 @forward (注入裸名成员到全局作用域)
///
/// 嵌套 @import 语义:
///   - 顶层 @import → BFS 展开, 内容提升到文件顶部
///   - 嵌套 @import (在 selector 块内) → 文件内容内联到 @import 位置
///     (Sass 规范: 被导入文件的内容如同写在父作用域内)
pub fn resolve_imports(input: &str, files: &std::collections::HashMap<String, String>) -> String {
    let _span = info_span!("import_resolve", bytes = input.len()).entered();

    // 共享状态: @forward 展开的去重集合
    let mut fwd_loaded: HashSet<String> = HashSet::new();

    // 辅助函数: 展开单个文件内容中的 @forward 和 @import (BFS)
    // 返回展开后的行列表, 同时将嵌套 @import 的 URL 写入 nested_imports
    fn expand_file_content(
        url: &str,
        files: &std::collections::HashMap<String, String>,
        fwd_loaded: &mut HashSet<String>,
        nested_imports: &mut Vec<String>,
    ) -> Vec<String> {
        let content = resolve_file_path(url, files);
        if content.is_empty() {
            return vec![];
        }
        let mut expanded: Vec<String> = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("@import ") {
                let nested_url = extract_import_url(rest);
                let bare_nested = nested_url.trim_matches('"').trim().to_string();
                if !bare_nested.is_empty() && !fwd_loaded.contains(&bare_nested) {
                    fwd_loaded.insert(bare_nested.clone());
                    nested_imports.push(bare_nested);
                }
            } else if let Some(rest) = trimmed.strip_prefix("@forward ") {
                let (fwd_url, fwd_as_prefix) = parse_forward_line(rest);
                let bare_fwd = fwd_url.trim_matches('"').trim().to_string();
                if !bare_fwd.is_empty() && !fwd_loaded.contains(&bare_fwd) {
                    fwd_loaded.insert(bare_fwd.clone());
                    let fwd_content = resolve_file_path(&bare_fwd, files);
                    if !fwd_content.is_empty() {
                        let members = parse_module_members(fwd_content);
                        let alias = fwd_as_prefix.unwrap_or_default();
                        let emitted = emit_namespaced_members(&members, &alias);
                        expanded.push(emitted);
                    }
                }
            } else {
                expanded.push(line.to_string());
            }
        }
        expanded
    }

    // Phase A: 逐行扫描, 区分顶层 @import 和嵌套 @import
    let mut import_urls: Vec<String> = Vec::new(); // 顶层 @import 的 URL 队列
    let mut output_lines: Vec<String> = Vec::new();
    let mut loaded: HashSet<String> = HashSet::new();
    let mut brace_depth: i32 = 0;

    for line in input.lines() {
        let trimmed = line.trim();
        let depth_delta = line.chars().fold(0i32, |d, c| match c {
            '{' => d + 1,
            '}' => d - 1,
            _ => d,
        });
        let in_nesting = brace_depth > 0;
        brace_depth += depth_delta;

        if let Some(rest) = trimmed.strip_prefix("@import ") {
            let url = extract_import_url(rest);
            let bare_url = url.trim_matches('"').trim().to_string();
            if !bare_url.is_empty() && !loaded.contains(&bare_url) {
                let content = resolve_file_path(&bare_url, files);
                if !content.is_empty() {
                    loaded.insert(bare_url.clone());
                    fwd_loaded.insert(bare_url.clone());
                    if in_nesting {
                        // 嵌套 @import → 展开 @forward 后内联到当前位置
                        let mut nested_imports: Vec<String> = Vec::new();
                        let expanded = expand_file_content(&bare_url, files, &mut fwd_loaded, &mut nested_imports);
                        output_lines.extend(expanded);
                        // 处理嵌套 @import 链中的子 import
                        while let Some(nested_url) = nested_imports.pop() {
                            let mut sub_imports: Vec<String> = Vec::new();
                            let sub_expanded = expand_file_content(&nested_url, files, &mut fwd_loaded, &mut sub_imports);
                            output_lines.extend(sub_expanded);
                            nested_imports.extend(sub_imports);
                        }
                    } else {
                        // 顶层 @import → BFS 提升
                        import_urls.push(bare_url);
                    }
                    continue;
                }
            }
            // 无法解析的 @import → 保留原行
            output_lines.push(line.to_string());
        } else {
            output_lines.push(line.to_string());
        }
    }

    // Phase B: BFS 展开顶层 @import 链
    let mut expanded: Vec<String> = Vec::new();
    let mut queue: VecDeque<String> = import_urls.into_iter().collect();

    while let Some(url) = queue.pop_front() {
        let mut nested_imports: Vec<String> = Vec::new();
        let lines = expand_file_content(&url, files, &mut fwd_loaded, &mut nested_imports);
        expanded.extend(lines);
        queue.extend(nested_imports);
    }

    // Phase C: 拼接 — 顶层展开内容在前, 原始行 (含嵌套内联) 在后
    let mut result = String::new();
    for line in &expanded {
        result.push_str(line);
        result.push('\n');
    }
    for line in &output_lines {
        result.push_str(line);
        result.push('\n');
    }
    result
}

/// 解析 @forward 行, 提取 URL 和可选的 as prefix-* 前缀
///
/// 返回 (url, Option<prefix>)
/// 示例:
///   @forward "foo"              -> ("foo", None)
///   @forward "foo" as bar-*     -> ("foo", Some("bar"))
fn parse_forward_line(rest: &str) -> (String, Option<String>) {
    let s = rest.trim().trim_end_matches(';').trim();
    
    // 提取 URL (引号形式)
    let url = if let Some(inner) = s.strip_prefix('"').and_then(|r| r.find('"').map(|i| &r[..i])) {
        inner.to_string()
    } else if let Some(inner) = s.strip_prefix('\'').and_then(|r| r.find('\'').map(|i| &r[..i])) {
        inner.to_string()
    } else {
        // 无引号或 url(...) 形式, 取第一个空格前的部分
        s.split_whitespace().next().unwrap_or(s).to_string()
    };
    
    // 检查是否有 as prefix-* 前缀
    let prefix = if let Some(after_url) = s.find(&url).map(|i| &s[i + url.len()..]) {
        if let Some(after_as) = after_url.trim().strip_prefix("as ") {
            // 提取前缀 (去掉末尾的 -*)
            let prefix_part = after_as.trim().split_whitespace().next().unwrap_or("");
            let prefix = prefix_part.trim_end_matches("-*").trim().to_string();
            if prefix.is_empty() { None } else { Some(prefix) }
        } else {
            None
        }
    } else {
        None
    };
    
    (url, prefix)
}

/// 从 @import 行提取 URL
///
/// 支持: @import "foo"; @import 'foo'; @import url(foo);
fn extract_import_url(rest: &str) -> String {
    let s = rest.trim().trim_end_matches(';').trim();

    // url(...) 形式
    if let Some(inner) = s.strip_prefix("url(").and_then(|r| r.strip_suffix(')')) {
        return inner.trim_matches('"').trim_matches('\'').trim().to_string();
    }

    // 引号形式 (单引号或双引号)
    let first = s.chars().next();
    if first == Some('"') || first == Some('\'') {
        let quote = first.unwrap();
        if let Some(end) = s[1..].find(quote) {
            return s[1..1 + end].to_string();
        }
    }

    // 无引号
    s.to_string()
}
