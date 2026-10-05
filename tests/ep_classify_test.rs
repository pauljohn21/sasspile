//! EP DIFF 分类诊断——区分 sasspile bug 与 EP 管线产物（autoprefixer/lightningcss）。
//!
//! 核心思路：将 sasspile 输出和 EP dist 都通过 lightningcss minify，
//! 然后**移除 autoprefixer 注入的属性**（它们不在 sasspile 控制范围内），
//! 再比较剩余差异 → 暴露真正的 sasspile bug。
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

/// 移除 autoprefixer 注入的 CSS 属性（EP 管线产物，非 sasspile 职责）
fn strip_autoprefixer(css: &str) -> String {
    // 移除以下 autoprefixer 注入的属性行：
    // -webkit-appearance, -webkit-user-select, -moz-user-select,
    // -ms-user-select, -webkit-inner-spin-button, -webkit-outer-spin-button,
    // -webkit-tap-highlight-color, -webkit-box-orient, -webkit-line-clamp,
    // backface-visibility (有些场景是 autoprefixer), -webkit-box-shadow (某些情况)
    let lines: Vec<&str> = css.split(';')
        .filter(|prop| {
            let p = prop.trim();
            !(p.contains("-webkit-appearance")
                || p.contains("-webkit-user-select")
                || p.contains("-moz-user-select")
                || p.contains("-ms-user-select")
                || p.contains("-webkit-inner-spin-button")
                || p.contains("-webkit-outer-spin-button")
                || p.contains("-webkit-tap-highlight-color")
                || p.contains("-webkit-box-orient")
                || p.contains("-webkit-line-clamp")
                || p.contains("backface-visibility"))
        })
        .collect();
    lines.join(";")
}

#[test]
fn test_classify_ep_diffs() {
    sasspile::init_tracing();
    let src_dir = PathBuf::from(EP_SRC);
    let dist_dir = PathBuf::from(EP_DIST);

    let mut entries: Vec<_> = std::fs::read_dir(&src_dir)
        .expect("无法读取 src 目录")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "scss"))
        .collect();
    entries.sort_by_key(|e| e.path());

    let mut bug_files: Vec<(String, String)> = vec![];
    let mut autoprefixer_only: Vec<String> = vec![];
    let mut identical_after_strip: Vec<String> = vec![];
    let mut identical: Vec<String> = vec![];

    for entry in &entries {
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let dist_name = scss_to_dist_name(&name);
        let dist_path = dist_dir.join(&dist_name);

        let sasspile_css = match sasspile::compile_file(&path, sasspile::OutputStyle::Expanded) {
            Ok(css) => css,
            Err(e) => {
                tracing::error!(file = %name, error = %e, "✗ COMPILE FAIL");
                continue;
            }
        };

        let normalized_sp = match normalize_css(&sasspile_css) {
            Ok(css) => css,
            Err(_) => continue,
        };

        let dist_css = match std::fs::read_to_string(&dist_path) {
            Ok(css) => normalize_css(&css).unwrap_or(css),
            Err(_) => continue,
        };

        if normalized_sp == dist_css {
            identical.push(name.clone());
            continue;
        }

        // 移除 autoprefixer 注入后再比较
        let sp_stripped = strip_autoprefixer(&normalized_sp);
        let dist_stripped = strip_autoprefixer(&dist_css);

        if sp_stripped == dist_stripped {
            autoprefixer_only.push(name.clone());
        } else {
            // 还有差异 → 真正的 bug 或 lightningcss 产物
            // 找第一个 diff 位置，输出上下文
            let sp_chars: Vec<char> = sp_stripped.chars().collect();
            let dist_chars: Vec<char> = dist_stripped.chars().collect();
            let diff_pos = sp_chars.iter()
                .zip(dist_chars.iter())
                .position(|(a, b)| a != b)
                .unwrap_or(sp_chars.len().min(dist_chars.len()));
            let start = diff_pos.saturating_sub(30);
            let end = (diff_pos + 60).min(sp_chars.len());
            let ctx: String = sp_chars[start..end].iter().collect();
            let dist_ctx: String = dist_chars[start..end.min(dist_chars.len())].iter().collect();

            // 进一步判断是否为 lightningcss artifact
            let is_lightning = ctx.contains("translate") || ctx.contains("calc(")
                || ctx.contains("color-scheme");
            if is_lightning {
                bug_files.push((name.clone(), format!("lightningcss? @{diff_pos}: SP=`{}` EP=`{}`", ctx, dist_ctx)));
            } else {
                bug_files.push((name.clone(), format!("sasspile-bug @{diff_pos}: SP=`{}` EP=`{}`", ctx, dist_ctx)));
            }
            identical_after_strip.push(name.clone());
        }
    }

    tracing::error!("=============== EP DIFF 分类报告 ===============");
    tracing::error!(count = identical.len(), "✅ 完全一致 ({} files)", identical.len());
    tracing::error!(count = autoprefixer_only.len(), "🔧 仅 autoprefixer 差异 ({} files): {}", autoprefixer_only.len(), autoprefixer_only.join(", "));
    tracing::error!(count = bug_files.len(), "🐛 移除 autoprefixer 后仍有差异 ({} files):", bug_files.len());
    for (name, ctx) in &bug_files {
        tracing::error!(file = %name, ctx = %ctx, "  → {}", name);
    }
    tracing::error!("================================================");
    tracing::error!(
        "总结: {} 一致 + {} autoprefixer + {} bug = {} / {} total",
        identical.len(),
        autoprefixer_only.len(),
        bug_files.len(),
        identical.len() + autoprefixer_only.len() + bug_files.len(),
        entries.len()
    );
}
