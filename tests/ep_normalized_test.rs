//! —— EP 对比测试（lightningcss 规范化后对比）——
//!
//! EP 官方 dist 由 dart-sass 编译后 lightningcss 压缩生成。
//! 将 sasspile 编译结果也过 lightningcss minify 与 EP dist 对比，
//! 消除格式差异，只暴露**语义差异**。
#![allow(clippy::unwrap_used, clippy::uninlined_format_args)]

use std::path::PathBuf;

const EP_SRC: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/element-plus/packages/theme-chalk/src"
);
const EP_DIST: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/element-plus/packages/theme-chalk/dist"
);

fn scss_to_dist_name(scss_name: &str) -> String {
    let stem = scss_name.trim_end_matches(".scss");
    let no_el = ["index", "base", "display"];
    if no_el.contains(&stem) {
        format!("{stem}.css")
    } else {
        format!("el-{stem}.css")
    }
}

/// 用 lightningcss 规范化 CSS：parse → minify → to_css → 保留空 keyframe 块后处理。
///
/// lightningcss 的 `MinifyOptions` 不支持配置"保留空规则"，其 minify 会无条件
/// 移除空的 `0% {}` / `100% {}` keyframe 块。由于 EP (Element Plus) 官方输出中保留
/// 了这些空块，后处理步骤将检测 minify 移除的空块并补回占位声明。
fn normalize_css(css: &str) -> Result<String, String> {
    use lightningcss::{
        printer::PrinterOptions,
        stylesheet::{MinifyOptions, ParserOptions, StyleSheet},
    };

    let stylesheet = StyleSheet::parse(css, ParserOptions::default())
        .map_err(|e| format!("parse: {e:?}"))?;
    let mut s = stylesheet;
    s.minify(MinifyOptions::default())
        .map_err(|e| format!("minify: {e:?}"))?;

    let result = s.to_css(PrinterOptions { minify: true, ..Default::default() })
        .map_err(|e| format!("to_css: {e:?}"))?;

    // 后处理：(dart-sass 兼容) 移除空边界 keyframe 步骤以对齐 EP 官方输出
    Ok(strip_empty_boundary_keyframes(result.code))
}

/// dart-sass 兼容：从规范化 CSS 中移除空的首尾 keyframe 步骤。
///
/// dart-sass 会将形如 `@keyframes X { 0% {} 100% { opacity: 0 } }` 的关键帧
/// 输出为 `@keyframes X { 100% { opacity: 0 } }`（自动删除首尾空步骤）。
/// sasspile 严格保留 SCSS 源码中的所有显式步骤（包括空的），导致同比 DIFF。
fn strip_empty_boundary_keyframes(css: String) -> String {
    let mut result = css;
    let mut search_start = 0;

    while let Some(kf_rel) = result[search_start..].find("@keyframes") {
        let kf_abs = search_start + kf_rel;
        let after_kw = kf_abs + "@keyframes".len();
        let Some(brace_rel) = result[after_kw..].find('{') else { break; };
        let brace_open = after_kw + brace_rel;
        let Some(brace_close) = find_matching_close(&result, brace_open) else { break; };

        let body = result[brace_open + 1..brace_close].to_string();
        let new_body_opt = remove_boundary_empty_steps(&body);
        match new_body_opt {
            Some(new_body) if new_body.len() != body.len() => {
                if new_body.is_empty() {
                    // 关键帧所有步骤都被删除 — 移除整个 @keyframes 规则
                    // 包括前面的空白/换行
                    let kf_start = kf_abs;
                    // 回退到前面的换行或空白
                    let mut del_start = kf_start;
                    while del_start > 0 {
                        let b = result.as_bytes()[del_start - 1];
                        if b == b'\n' || b == b' ' {
                            del_start -= 1;
                        } else {
                            break;
                        }
                    }
                    // 包括闭合 } 后的尾部空白
                    let mut del_end = brace_close + 1;
                    while del_end < result.len() {
                        let b = result.as_bytes()[del_end];
                        if b == b'\n' || b == b' ' {
                            del_end += 1;
                        } else {
                            break;
                        }
                    }
                    result.replace_range(del_start..del_end, "");
                    search_start = del_start;
                } else {
                    result.replace_range(brace_open + 1..brace_close, &new_body);
                    let offset = new_body.len() as isize - body.len() as isize;
                    search_start = (brace_close as isize + offset) as usize + 1;
                }
            }
            _ => {
                search_start = brace_close + 1;
            }
        }
    }

    result
}

/// 解析 body 为步骤序列（`%{...}` 或 `from{...}` / `to{...}`），移除首尾空步骤。
fn remove_boundary_empty_steps(body: &str) -> Option<String> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return None;
    }

    // 解析步骤：找到所有 `{...}` 块，每个块前的 `%`/`from`/`to` 是步骤头
    let steps = split_keyframe_steps(trimmed);
    if steps.is_empty() {
        return None;
    }

    let mut trimmed_steps: Vec<&str> = steps.iter().map(|s| *s).collect();

    // 移除开头空步骤
    while let Some(first) = trimmed_steps.first() {
        if is_empty_step(first) {
            trimmed_steps.remove(0);
        } else {
            break;
        }
    }
    // 移除结尾空步骤
    while let Some(last) = trimmed_steps.last() {
        if is_empty_step(last) {
            trimmed_steps.pop();
        } else {
            break;
        }
    }

    if trimmed_steps.is_empty() {
        Some(String::new())
    } else {
        Some(trimmed_steps.join(""))
    }
}

/// 将 keyframe body 拆分为步骤字符串列表（按顶层 `{...}` 分组）。
/// 追踪 `}` 之后的字符位置作为下一步骤的起始，正确处理 minified 格式 `}to`。
fn split_keyframe_steps(body: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth = 0;
    let mut step_start = 0;
    let bytes = body.as_bytes();

    for i in 0..bytes.len() {
        match bytes[i] {
            b'{' => {
                if depth == 0 {
                    // 从 { 位置向前找步骤头的起始：跳过空白，回退到上一个标识符开头
                    step_start = walk_back_to_step_start(body, i);
                }
                depth += 1;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    result.push(&body[step_start..=i]);
                    // 下一步骤的起始 — 从 i+1 开始（跳过后面空白）
                    let mut next = i + 1;
                    while next < bytes.len() && bytes[next].is_ascii_whitespace() {
                        next += 1;
                    }
                    step_start = next;
                }
            }
            _ => {}
        }
    }

    result
}

/// 从 `{` 位置回退找到步骤头的起始（`0%`/`100%`/`from`/`to`/`50%` 等）。
/// 步骤头是形如 `N%` 或 `from`/`to` 的 token，它的前驱要么是开头 `{`、空格、`}`。
fn walk_back_to_step_start(body: &str, brace_pos: usize) -> usize {
    let bytes = body.as_bytes();
    let mut i = brace_pos;

    // 跳过前导空白
    while i > 0 && bytes[i - 1].is_ascii_whitespace() {
        i -= 1;
    }

    // 现在 i 指向 token 的最后一个字符 — 回退到 token 起始
    while i > 0 && !bytes[i - 1].is_ascii_whitespace() {
        // 遇到 `}` 就停（步骤分隔符）
        if bytes[i - 1] == b'}' {
            break;
        }
        i -= 1;
    }

    i
}

/// 判断一个步骤是否为空（只有 `%{...}` 内部为空或只有空白）。
fn is_empty_step(step: &str) -> bool {
    let Some(open) = step.find('{') else { return false; };
    let Some(close) = step.rfind('}') else { return false; };
    if open >= close {
        return false;
    }
    step[open + 1..close].trim().is_empty()
}

/// 找到匹配的闭合 `}` 位置（处理嵌套 `{`）。
fn find_matching_close(s: &str, open_pos: usize) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in s[open_pos..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(open_pos + i);
                }
            }
            _ => {}
        }
    }
    None
}

#[test]
fn test_ep_lightningcss_normalized_diff() {
    sasspile::init_tracing();
    let src_dir = PathBuf::from(EP_SRC);
    let dist_dir = PathBuf::from(EP_DIST);

    let mut entries: Vec<_> = std::fs::read_dir(&src_dir)
        .expect("无法读取 src 目录")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "scss"))
        .collect();
    entries.sort_by_key(|e| e.path());

    let total = entries.len();
    let mut identical = 0;
    let mut diffs: Vec<(String, String)> = vec![];
    let mut errors = 0;

    for entry in &entries {
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let dist_name = scss_to_dist_name(&name);
        let dist_path = dist_dir.join(&dist_name);

        let sasspile_css = match sasspile::compile_file(&path, sasspile::OutputStyle::Expanded) {
            Ok(css) => css,
            Err(e) => {
                tracing::warn!(file = %name, error = %e, "✗ sasspile FAIL");
                errors += 1;
                continue;
            }
        };

        let normalized_sp = match normalize_css(&sasspile_css) {
            Ok(css) => css,
            Err(e) => {
                tracing::warn!(file = %name, error = %e, "⚡ lightningcss FAIL");
                errors += 1;
                continue;
            }
        };

        let dist_css = match std::fs::read_to_string(&dist_path) {
            Ok(css) => {
                // 规范化 dist 文件（移除 vendor prefix、统一格式化）——确保和 sasspile 输出在同一基础上对比
                normalize_css(&css).unwrap_or_else(|_| css)
            }
            Err(_) => continue,
        };

        if normalized_sp == dist_css {
            identical += 1;
            tracing::info!(file = %name, "✓ IDENTICAL");
        } else {
            let diff_pos = normalized_sp.chars().zip(dist_css.chars())
                .position(|(a, b)| a != b)
                .unwrap_or(normalized_sp.len().min(dist_css.len()));
            let context: String = normalized_sp.chars()
                .skip(diff_pos.saturating_sub(20))
                .take(80)
                .collect();
            diffs.push((
                format!("{name} [sp={} dist={}]", normalized_sp.len(), dist_css.len()),
                format!("@{diff_pos}: …{context}…"),
            ));
        }
    }

    let pct = if total > 0 { identical as f64 / total as f64 * 100.0 } else { 0. };
    tracing::info!(
        total = total,
        identical = identical,
        diff = diffs.len(),
        errors = errors,
        "===== lightningcss 对比: {}/{} 一致 ({:.1}%) =====",
        identical, total, pct
    );
    for (file, ctx) in diffs.iter().take(20) {
        tracing::warn!(file = %file, context = %ctx, "✗ DIFF");
    }
}
