//! sass-spec manifest——目录索引。
//!
//! 注意：所有 3.x/CSS4 色彩空间、模块系统、calculation 值类型等文件已物理删除。
//! sasspile 直接运行剩余 spec，无跳过列表。

use std::path::{Path, PathBuf};

/// 跳过的 spec 子目录——CSS4 色彩空间等 sasspile 暂不支持的功能。
pub const SKIP_DIRS: &[&str] = &[
    // CSS4 色彩空间（sasspile 暂不支持）
    "core_functions/color/hwb",
    "core_functions/color/lab",
    "core_functions/color/lch",
    "core_functions/color/oklab",
    "core_functions/color/oklch",
    "core_functions/color/to_gamut",
    "core_functions/color/to_space",
    "core_functions/color/is_in_gamut",
    "core_functions/color/is_legacy",
    "core_functions/color/is_missing",
    "core_functions/color/is_powerless",
    "core_functions/color/space",
    "core_functions/color/blackness",
    "core_functions/color/whiteness",
];

/// 检查文件相对 spec_root 的路径是否在跳过列表中。
fn should_skip(rel_path: &str) -> bool {
    SKIP_DIRS
        .iter()
        .any(|skip| rel_path.starts_with(skip) || rel_path == *skip)
}

/// 收集 spec 目录下所有 HRX 文件，跳过 `SKIP_DIRS` 和 >50KB 的文件。
///
/// 参数：`dir` 要扫描的目录，`spec_root` spec 根目录（用于计算相对路径）。
/// 返回 (files, skipped_count)。
pub fn collect_hrx_files(dir: &Path, spec_root: &Path) -> (Vec<PathBuf>, usize) {
    let mut files = Vec::new();
    let mut skipped = 0;
    collect_recursive(dir, spec_root, &mut files, &mut skipped);
    (files, skipped)
}

fn collect_recursive(
    dir: &Path,
    spec_root: &Path,
    files: &mut Vec<PathBuf>,
    skipped: &mut usize,
) {
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
                    && meta.len() < 50_000 {
                        files.push(path);
                    }
            }
        }
    }
}

/// 递归收集所有 HRX 文件（含跳过的），用于 manifest 统计。
#[allow(dead_code)]
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
                    && meta.len() < 100_000 {
                        files.push(path);
                    }
        }
    }
}

/// 按一级目录统计 HRX 文件分布。
#[allow(dead_code)]
pub fn stats_by_dir(spec_root: &Path) -> Vec<(String, usize)> {
    let all = collect_all_hrx(spec_root);
    let mut counts: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
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
