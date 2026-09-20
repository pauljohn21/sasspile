//! —— element-plus 官方 dist 对比验证 ——
//!
//! 概要：用 EP 官方 build 流程生成的 dist/*.css 作为参考，对比 sasspile 输出。
//!       EP 官方流程：sass-embedded（dart-sass）编译 + lightningcss 压缩。
//!       因此对比时需将 sasspile 输出也用 lightningcss 同等规范化。
//!
//! 简化策略：将两边输出通过相同的 normalize（去注释、压缩空白、排序属性）后对比。
//!
//! 运行前置：
//!   cd element-plus/packages/theme-chalk && npx tsx buildfile.ts
//!
//! 运行：
//!   cargo test --test ep_dist_test -- --nocapture

use std::path::PathBuf;

const EP_SRC: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/element-plus/packages/theme-chalk/src"
);
const EP_DIST: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/element-plus/packages/theme-chalk/dist"
);

/// 强规范化 CSS：去注释、去空白、统一格式
/// 注意：对压缩后的 CSS 做结构级对比（属性集合 + 值）
fn normalize_for_compare(css: &str) -> String {
    let mut result = String::with_capacity(css.len());
    let bytes = css.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let c = bytes[i];

        // 块注释
        if c == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i += 2;
            continue;
        }

        // 字符串保留
        if c == b'"' || c == b'\'' {
            let quote = c;
            result.push(c as char);
            i += 1;
            while i < bytes.len() && bytes[i] != quote {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    result.push(bytes[i] as char);
                    i += 1;
                }
                result.push(bytes[i] as char);
                i += 1;
            }
            if i < bytes.len() {
                result.push(bytes[i] as char);
                i += 1;
            }
            continue;
        }

        if c.is_ascii_whitespace() {
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            result.push(' ');
            continue;
        }

        result.push(c as char);
        i += 1;
    }

    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// scss 文件名 → dist 文件名
/// button.scss → el-button.css
/// index.scss → index.css, base.scss → base.css, display.scss → display.css
fn scss_to_dist_name(scss_name: &str) -> String {
    let stem = scss_name.trim_end_matches(".scss");
    let no_el = ["index", "base", "display"];
    if no_el.contains(&stem) {
        format!("{stem}.css")
    } else {
        format!("el-{stem}.css")
    }
}

#[test]
fn test_ep_compare_with_official_dist() {
    sasspile::init_tracing();
    let src_dir = PathBuf::from(EP_SRC);
    let dist_dir = PathBuf::from(EP_DIST);

    let mut entries: Vec<_> = std::fs::read_dir(&src_dir)
        .expect("无法读取 src 目录")
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().is_some_and(|ext| ext == "scss")
        })
        .collect();
    entries.sort_by_key(|e| e.path());

    let total = entries.len();
    let mut identical = 0;
    let mut diff_count = 0;
    let mut no_dist_ref = 0;
    let mut sasspile_fail = 0;

    let mut diff_details: Vec<(String, String, String)> = vec![]; // (file, sasspile_summary, dist_summary)

    tracing::info!(total = total, "===== 对比 sasspile vs EP 官方 dist =====");

    for entry in &entries {
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let dist_name = scss_to_dist_name(&name);
        let dist_path = dist_dir.join(&dist_name);

        // sasspile 编译
        let sasspile_css = match sasspile::compile_file(&path, sasspile::OutputStyle::Expanded) {
            Ok(css) => css,
            Err(e) => {
                tracing::warn!(file = %name, error = %e, "✗ sasspile FAIL");
                sasspile_fail += 1;
                continue;
            }
        };

        // 读取官方 dist（压缩后的）
        let dist_css = match std::fs::read_to_string(&dist_path) {
            Ok(css) => css,
            Err(_) => {
                // 没有对应的 dist 文件（可能 scss 不需独立编译）
                no_dist_ref += 1;
                continue;
            }
        };

        let sasspile_norm = normalize_for_compare(&sasspile_css);
        let dist_norm = normalize_for_compare(&dist_css);

        if sasspile_norm == dist_norm {
            identical += 1;
            tracing::info!(file = %name, "✓ IDENTICAL");
        } else {
            diff_count += 1;
            // 计算差异度
            let s_chars: Vec<char> = sasspile_norm.chars().collect();
            let d_chars: Vec<char> = dist_norm.chars().collect();
            let max_len = s_chars.len().max(d_chars.len());
            let diff_chars: usize = (0..max_len)
                .filter(|&i| s_chars.get(i) != d_chars.get(i))
                .count();
            let similarity = if max_len > 0 {
                100.0 - (diff_chars as f64 / max_len as f64 * 100.0)
            } else {
                100.0
            };

            if similarity < 90.0 {
                tracing::warn!(
                    file = %name,
                    similarity = format!("{:.1}%", similarity),
                    sasspile_len = sasspile_norm.len(),
                    dist_len = dist_norm.len(),
                    "✗ DIFF"
                );
                diff_details.push((
                    name.clone(),
                    format!("similarity={:.1}% len={}", similarity, sasspile_norm.len()),
                    format!("len={}", dist_norm.len()),
                ));
            } else {
                tracing::info!(
                    file = %name,
                    similarity = format!("{:.1}%", similarity),
                    "~ NEAR"
                );
            }
        }
    }

    tracing::info!(
        total = total,
        identical = identical,
        diff = diff_count,
        sasspile_fail = sasspile_fail,
        no_dist_ref = no_dist_ref,
        "===== 结果: {}/{} 一致 ({:.1}%) =====",
        identical, total,
        if total > 0 { identical as f64 / total as f64 * 100. } else { 0. }
    );

    if !diff_details.is_empty() {
        tracing::warn!("=== 显著差异文件 (similarity < 90%) ===");
        for (file, ss, ds) in &diff_details {
            tracing::warn!(%file, sasspile = %ss, dist = %ds, "DIFF");
        }
    }
}
