//! EP DIFF 精确诊断——逐文件分析 sasspile vs EP 官方输出的具体差异模式。

#[test]
fn diag_badge_scss() {
    sasspile::init_tracing();
    let path = "element-plus/packages/theme-chalk/src/badge.scss";
    if let Ok(css) = sasspile::compile_file(
        &std::path::PathBuf::from(path),
        sasspile::OutputStyle::Expanded,
    ) {
        let relevant: String = css.lines()
            .filter(|l| l.contains("el-badge__content") && !l.starts_with("/*"))
            .take(5)
            .collect::<Vec<_>>()
            .join("\n");
        tracing::error!("=== SP badge (content only, top 5) ===\n{}", relevant);
    }
}

#[test]
fn diag_anchor_scss() {
    sasspile::init_tracing();
    let path = "element-plus/packages/theme-chalk/src/anchor.scss";
    if let Ok(css) = sasspile::compile_file(
        &std::path::PathBuf::from(path),
        sasspile::OutputStyle::Expanded,
    ) {
        // 只输出 anchor--vertical 相关规则
        let relevant: String = css.lines()
            .filter(|l| l.contains("anchor--vertical") || l.contains("anchor__marker"))
            .collect::<Vec<_>>()
            .join("\n");
        tracing::error!("=== SP anchor (vertical+marker only) ===\n{}", relevant);

        let dist_anchor = std::fs::read_to_string(
            "element-plus/packages/theme-chalk/dist/el-anchor.css"
        ).unwrap_or_default();
        let dist_relevant: String = dist_anchor.lines()
            .filter(|l| l.contains("anchor--vertical") || l.contains("anchor__marker"))
            .collect::<Vec<_>>()
            .join("\n");
        tracing::error!("=== EP anchor (vertical+marker only) ===\n{}", dist_relevant);
    }
}

#[test]
fn diag_descriptions_ep() {
    sasspile::init_tracing();
    let path = "element-plus/packages/theme-chalk/src/descriptions.scss";
    if let Ok(css) = sasspile::compile_file(
        &std::path::PathBuf::from(path),
        sasspile::OutputStyle::Expanded,
    ) {
        let _ = std::fs::write("/tmp/sp_desc_ep.out", &css);
        let dist = std::fs::read_to_string(
            "element-plus/packages/theme-chalk/dist/el-descriptions.css"
        ).unwrap_or_default();
        if css == dist {
            tracing::error!("descriptions.scss: IDENTICAL");
        } else {
            let sp_bytes = css.as_bytes();
            let dist_bytes = dist.as_bytes();
            let min_len = sp_bytes.len().min(dist_bytes.len());
            let mut first_diff = min_len;
            for i in 0..min_len {
                if sp_bytes[i] != dist_bytes[i] {
                    first_diff = i;
                    break;
                }
            }
            let s = first_diff.saturating_sub(60);
            let e = (first_diff + 60).min(sp_bytes.len().max(dist_bytes.len()));
            tracing::error!(pos = first_diff, "SP :`{}`",
                String::from_utf8_lossy(&sp_bytes[s..e.min(sp_bytes.len())]));
            tracing::error!(pos = first_diff, "EP :`{}`",
                String::from_utf8_lossy(&dist_bytes[s..e.min(dist_bytes.len())]));
        }
    }
}

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

#[test]
#[ignore = "Slow diagnostic, run with EP_TARGET env var"]
fn diag_ep_file_diffs() {
    sasspile::init_tracing();
    let src_dir = PathBuf::from(EP_SRC);
    let dist_dir = PathBuf::from(EP_DIST);
    let target_file = std::env::var("EP_TARGET").unwrap_or_default();

    let mut entries: Vec<_> = std::fs::read_dir(&src_dir)
        .expect("无法读取 src 目录")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "scss"))
        .filter(|e| {
            if target_file.is_empty() {
                true
            } else {
                e.path().file_name().is_some_and(|s| s == target_file.as_str())
            }
        })
        .collect();
    entries.sort_by_key(|e| e.path());

    for entry in &entries {
        let path = entry.path();
        let name = path.file_name().expect("path has file_name").to_string_lossy().to_string();
        let dist_name = scss_to_dist_name(&name);
        let dist_path = dist_dir.join(&dist_name);

        let sasspile_css = match sasspile::compile_file(&path, sasspile::OutputStyle::Expanded) {
            Ok(css) => css,
            Err(e) => {
                tracing::error!(file = %name, error = %e, "COMPILE FAIL");
                continue;
            }
        };

        let normalized_sp = match normalize_css(&sasspile_css) {
            Ok(css) => css,
            Err(e) => {
                tracing::error!(file = %name, error = %e, "sp normalize FAIL");
                continue;
            }
        };

        let dist_css = match std::fs::read_to_string(&dist_path) {
            Ok(css) => normalize_css(&css).unwrap_or(css),
            Err(e) => {
                tracing::warn!(file = %name, error = %e, "dist file not found");
                continue;
            }
        };

        if normalized_sp == dist_css {
            tracing::info!(file = %name, "✓ IDENTICAL");
            continue;
        }

        // 逐字符 diff
        let sp_chars: Vec<char> = normalized_sp.chars().collect();
        let dist_chars: Vec<char> = dist_css.chars().collect();
        let max_len = sp_chars.len().max(dist_chars.len());
        let mut diff_count = 0;
        let mut in_diff = false;
        let mut diff_start = 0;

        for i in 0..max_len {
            let sp_c = sp_chars.get(i).copied().unwrap_or('\0');
            let dist_c = dist_chars.get(i).copied().unwrap_or('\0');
            let is_diff = sp_c != dist_c;

            if is_diff && !in_diff {
                in_diff = true;
                diff_start = i;
            } else if !is_diff && in_diff {
                in_diff = false;
                if diff_count < 8 {
                    let sp_slice: String = sp_chars[diff_start..i].iter().take(120).collect();
                    let dist_slice: String = dist_chars[diff_start..i].iter().take(120).collect();
                    if sp_slice != dist_slice {
                        tracing::warn!(file = %name, seg = diff_count, "DIFF[{}] SP=`{}`", diff_count, sp_slice);
                        tracing::warn!(file = %name, "DIFF[{}] EP=`{}`", diff_count, dist_slice);
                    }
                }
                diff_count += 1;
            }
        }
        if in_diff {
            diff_count += 1;
        }
        tracing::warn!(
            file = %name,
            sp_total = sp_chars.len(),
            dist_total = dist_chars.len(),
            diff_segments = diff_count,
            "� {} diff segments",
            diff_count
        );
    }
}
