//! sass-spec manifest——目录索引 + 跳过列表。
//!
//! 跳过已弃用的目录（libsass 系列、non_conformant）。
//! 颜色测试已纳入全量统计，不再跳过。
//!
//! **.sass 缩进式语法跳过**：sasspile 只支持 `.scss` 大括号语法，
//! 不支持 `.sass` 缩进式语法。`sass_spec_full.rs` 的 `run_spec_dir`
//! 在 case 级别跳过所有 `input.sass` 测试用例（不跳过整个 HRX 文件，
//! 因为同一文件可能同时包含 `.scss` 和 `.sass` 用例）。

use std::path::{Path, PathBuf};

/// 跳过的 spec 目录（已弃用/非标准）。
///
/// - `libsass` 系列：LibSass 实现的旧测试，已被 SCSS 规范弃用
/// - `non_conformant`：不符合规范的旧测试
pub const SKIP_DIRS: &[&str] = &[
    // —— LibSass 弃用目录 ——
    "libsass",
    "libsass-closed-issues",
    "libsass-todo-issues",
    "libsass-todo-tests",
    // —— 不符合规范的旧测试 ——
    "non_conformant",
];

/// 检查文件相对 `spec_root` 的路径是否在跳过列表中。
#[allow(dead_code)]
fn should_skip(rel_path: &str) -> bool {
    // 检查是否在跳过的顶层目录下
    SKIP_DIRS
        .iter()
        .any(|skip| rel_path.starts_with(skip) || rel_path == *skip)
}

/// 收集 spec 目录下所有 HRX 文件，跳过 `SKIP_DIRS` 和 >100KB 的文件。
///
/// 参数：`dir` 要扫描的目录，`spec_root` spec 根目录（用于计算相对路径）。
/// 返回 (files, `skipped_count`)。
#[allow(dead_code)]
#[must_use]
pub fn collect_hrx_files(dir: &Path, spec_root: &Path) -> (Vec<PathBuf>, usize) {
    let mut files = Vec::new();
    let mut skipped = 0;
    collect_recursive(dir, spec_root, &mut files, &mut skipped);
    (files, skipped)
}

#[allow(dead_code)]
fn collect_recursive(dir: &Path, spec_root: &Path, files: &mut Vec<PathBuf>, skipped: &mut usize) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_recursive(&path, spec_root, files, skipped);
            } else if path.extension().and_then(|s| s.to_str()) == Some("hrx") {
                let rel = path
                    .strip_prefix(spec_root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();
                if should_skip(&rel) {
                    *skipped += 1;
                    continue;
                }
                if let Ok(meta) = std::fs::metadata(&path)
                    && meta.len() < 100_000
                {
                    files.push(path);
                }
            }
        }
    }
}

/// 递归收集所有 HRX 文件（含跳过的），用于 manifest 统计。
#[allow(dead_code)]
#[must_use]
pub fn collect_all_hrx(spec_root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_all_recursive(spec_root, &mut files);
    files
}

#[allow(dead_code)]
fn collect_all_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_all_recursive(&path, files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("hrx")
                && let Ok(meta) = std::fs::metadata(&path)
                && meta.len() < 100_000
            {
                files.push(path);
            }
        }
    }
}

/// 按一级目录统计 HRX 文件分布。
#[allow(dead_code)]
#[must_use]
pub fn stats_by_dir(spec_root: &Path) -> Vec<(String, usize)> {
    let all = collect_all_hrx(spec_root);
    let mut counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for path in &all {
        let rel = path.strip_prefix(spec_root).unwrap_or(path);
        let first = rel
            .components()
            .next()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .unwrap_or_default();
        *counts.entry(first).or_default() += 1;
    }
    counts.into_iter().collect()
}
