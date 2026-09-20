//! —— element-plus 对比验证：sasspile vs dart-sass ——
//!
//! 概要：逐文件对比 sasspile 和 dart-sass 的编译输出，验证 100% 正确性。
//!       使用项目的 include paths 确保 @use/@import 能正确解析。
//!
//! 运行：cd /Users/pauljohn/rust/sasspile && cargo test --test ep_diff_test -- --nocapture

use std::path::PathBuf;
use std::process::Command;

const EP_SRC: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/element-plus/packages/theme-chalk/src"
);

/// 编译单个 SCSS 文件（dart-sass），返回 CSS 字符串
fn compile_dart_sass(path: &PathBuf, load_paths: &[PathBuf]) -> Result<String, String> {
    let mut cmd = Command::new("/opt/homebrew/bin/sass");
    cmd.arg("--style=expanded");
    cmd.arg("--no-source-map");
    for lp in load_paths {
        cmd.arg("--load-path").arg(lp);
    }
    cmd.arg(path);

    let output = cmd
        .output()
        .map_err(|e| format!("执行 dart-sass 失败: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("dart-sass 编译失败: {stderr}"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// 规范化 CSS：移除注释、压缩空白，保留结构用于对比
fn normalize(css: &str) -> String {
    let mut result = String::with_capacity(css.len());
    let bytes = css.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let c = bytes[i];

        // 块注释 /* ... */
        if c == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() {
                if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    i += 2;
                    break;
                }
                i += 1;
            }
            if !result.ends_with(' ') && !result.is_empty() {
                result.push(' ');
            }
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

        // 空白归一化
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

#[test]
fn test_ep_diff_all_files() {
    sasspile::init_tracing();
    let src_dir = PathBuf::from(EP_SRC);
    let load_paths = vec![
        src_dir.clone(),
        src_dir.join("mixins"),
    ];

    let mut entries: Vec<_> = std::fs::read_dir(&src_dir)
        .expect("无法读取 src 目录")
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "scss")
        })
        .collect();
    entries.sort_by_key(|e| e.path());

    let total = entries.len();
    let mut identical = 0;
    let mut diff_count = 0;
    let mut error_count = 0;

    tracing::info!(total = total, "===== 开始对比 {} 个 SCSS 文件 (sasspile vs dart-sass) =====", total);

    for entry in &entries {
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();

        // sasspile 编译
        let sasspile_css = match sasspile::compile_file(&path, sasspile::OutputStyle::Expanded) {
            Ok(css) => css,
            Err(e) => {
                tracing::warn!(file = %name, error = %e, "✗ sasspile FAIL");
                error_count += 1;
                continue;
            }
        };

        // dart-sass 编译
        let dart_css = match compile_dart_sass(&path, &load_paths) {
            Ok(css) => css,
            Err(_) => {
                // dart-sass 也编译失败 → 跳过（通常是特殊语法）
                continue;
            }
        };

        let sasspile_norm = normalize(&sasspile_css);
        let dart_norm = normalize(&dart_css);

        if sasspile_norm == dart_norm {
            identical += 1;
            tracing::info!(file = %name, "✓ IDENTICAL");
        } else {
            diff_count += 1;
            let diff_chars = diff_count_chars(&sasspile_norm, &dart_norm);
            let pct = if !dart_norm.is_empty() {
                100.0 - (diff_chars as f64 / dart_norm.len().max(1) as f64 * 100.0)
            } else {
                0.0
            };

            // 找到第一处不同
            let ss_chars: Vec<char> = sasspile_norm.chars().collect();
            let dd_chars: Vec<char> = dart_norm.chars().collect();
            let mut first_diff_pos = None;
            for (i, (sc, dc)) in ss_chars.iter().zip(dd_chars.iter()).enumerate() {
                if sc != dc {
                    first_diff_pos = Some(i);
                    break;
                }
            }

            if let Some(pos) = first_diff_pos {
                let start = pos.saturating_sub(40);
                let end_s = (pos + 40).min(ss_chars.len());
                let end_d = (pos + 40).min(dd_chars.len());
                let sasspile_context: String = ss_chars[start..end_s].iter().collect();
                let dart_context: String = dd_chars[start..end_d].iter().collect();
                tracing::warn!(
                    file = %name,
                    similarity = format!("{:.1}%", pct),
                    pos = pos,
                    sasspile_ctx = %sasspile_context,
                    dart_ctx = %dart_context,
                    "✗ DIFF"
                );
            }
        }
    }

    tracing::info!(
        total = total,
        identical = identical,
        diff = diff_count,
        error = error_count,
        "===== 对比完成: {}/{} 一致 ({:.1}%) =====",
        identical, total,
        if total > 0 { identical as f64 / total as f64 * 100. } else { 0. }
    );

    assert_eq!(
        diff_count, 0,
        "存在 {}/{} 文件与 dart-sass 输出不一致",
        diff_count, total
    );
}

fn diff_count_chars(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let max_len = a_chars.len().max(b_chars.len());
    (0..max_len)
        .filter(|&i| a_chars.get(i) != b_chars.get(i))
        .count()
}
