//! —— EP 对比测试（lightningcss 规范化后对比）——
//!
//! EP 官方 dist 由 dart-sass 编译后 lightningcss 压缩生成。
//! 将 sasspile 编译结果也过 lightningcss minify 与 EP dist 对比，
//! 消除格式差异，只暴露**语义差异**。

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

/// 用 lightningcss 规范化 CSS：parse → minify → to_css
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

    Ok(result.code)
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
