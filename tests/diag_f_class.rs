//! F 类文件快速诊断——仅编译 sasspile-bug 候选文件
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

const F_FILES: &[&str] = &[
    "drawer.scss",
    "empty.scss",
    "image-viewer.scss",
    "message-box.scss",
    "pagination.scss",
    "popover.scss",
    "rate.scss",
    "select.scss",
    "select-v2.scss",
    "splitter.scss",
    "step.scss",
    "table-v2.scss",
    "date-picker-panel.scss",
    "option.scss",
];

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

/// 从 minified CSS 中移除 autoprefixer 注入的 -webkit-/-moz-/-ms- 属性。
///
/// minified CSS 结构: `selector{prop;prop;prop}`。扫描属性边界（`{` 或 `;` 之后），
/// 当属性名以 `-webkit-`/`-moz-`/`-ms-`/`backface-visibility` 开头时丢弃。
/// 边界情况：末尾属性（`}` 前）被删时需同时移除其前导分隔符 `;`。
fn strip_autoprefixer(css: &str) -> String {
    let mut result = String::with_capacity(css.len());
    let bytes = css.as_bytes();
    let mut i = 0;
    let mut last_semi_in_result: Option<usize> = None;
    while i < bytes.len() {
        let c = bytes[i] as char;
        let at_property_start = i == 0
            || bytes[i - 1] as char == '{'
            || bytes[i - 1] as char == ';';
        if at_property_start {
            let rest = &css[i..];
            if rest.starts_with("-webkit-")
                || rest.starts_with("-moz-")
                || rest.starts_with("-ms-")
                || rest.starts_with("backface-visibility")
            {
                // 跳过此属性值，直到 ; 或 }
                while i < bytes.len() && bytes[i] as char != ';' && bytes[i] as char != '}' {
                    i += 1;
                }
                if i < bytes.len() && bytes[i] as char == ';' {
                    // 跳过分号（属性后的分隔符）
                    i += 1;
                } else if i < bytes.len() && bytes[i] as char == '}' {
                    // 该属性是规则最后一条，移除 result 中尾部的 ;
                    if let Some(pos) = last_semi_in_result {
                        result.truncate(pos);
                        last_semi_in_result = None;
                    }
                }
                continue;
            }
        }
        result.push(c);
        if c == ';' {
            last_semi_in_result = Some(result.len() - 1);
        } else if c == '{' {
            last_semi_in_result = None;
        }
        i += 1;
    }
    result
}

#[test]
fn diag_f_class_files() {
    sasspile::init_tracing();
    let dist_dir = PathBuf::from(EP_DIST);
    let src_dir = PathBuf::from(EP_SRC);

    for name in F_FILES {
        let path = src_dir.join(name);
        let dist_name = scss_to_dist_name(name);
        let dist_path = dist_dir.join(&dist_name);

        let sasspile_css = match sasspile::compile_file(&path, sasspile::OutputStyle::Expanded) {
            Ok(css) => css,
            Err(e) => {
                tracing::error!(file = name, error = %e, "✗ COMPILE FAIL");
                continue;
            }
        };

        let normalized_sp = match normalize_css(&sasspile_css) {
            Ok(css) => css,
            Err(e) => {
                tracing::error!(file = name, error = %e, "✗ SP NORMALIZE FAIL");
                continue;
            }
        };

        let dist_css = match std::fs::read_to_string(&dist_path) {
            Ok(css) => match normalize_css(&css) {
                Ok(n) => n,
                Err(_) => css,
            },
            Err(_) => {
                tracing::warn!(file = name, "⚠ NO DIST FILE");
                continue;
            }
        };

        if normalized_sp == dist_css {
            tracing::info!(file = name, "✅ 完全一致");
            continue;
        }

        let sp_stripped = strip_autoprefixer(&normalized_sp);
        let dist_stripped = strip_autoprefixer(&dist_css);

        if sp_stripped == dist_stripped {
            tracing::info!(file = name, "🔧 仅 autoprefixer 差异");
        } else {
            let sp_chars: Vec<char> = sp_stripped.chars().collect();
            let dist_chars: Vec<char> = dist_stripped.chars().collect();
            let diff_pos = sp_chars.iter()
                .zip(dist_chars.iter())
                .position(|(a, b)| a != b)
                .unwrap_or(sp_chars.len().min(dist_chars.len()));
            let start = diff_pos.saturating_sub(40);
            let end = (diff_pos + 80).min(sp_chars.len());
            let ctx: String = sp_chars[start..end].iter().collect();
            let dist_ctx: String = dist_chars[start..end.min(dist_chars.len())].iter().collect();

            tracing::error!(
                file = name,
                pos = diff_pos,
                sp_ctx = %ctx,
                dist_ctx = %dist_ctx,
                "🐛 sasspile-bug"
            );
        }
    }
}
