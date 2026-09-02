//! sass-spec manifest——目录索引 + 跳过列表。
//!
//! 跳过已弃用的目录（libsass 系列、non_conformant）和颜色相关目录。
//! 颜色目录跳过以防止在非颜色任务中反复触发颜色测试失败导致无限修复循环。
//! 如需专门测试颜色功能，使用 `--ignored` 手动触发颜色相关测试。

use std::path::{Path, PathBuf};

/// 跳过的 spec 目录（已弃用/非标准/颜色）。
///
/// - `libsass` 系列：LibSass 实现的旧测试，已被 SCSS 规范弃用
/// - `non_conformant`：不符合规范的旧测试
/// - `core_functions/color`：颜色函数（adjust/change/scale/channel/mix/hsl/hwb/rgb/invert/
///   is_powerless/lab/lch/oklab/oklch/color/to_space/to_gamut + adjust_color/adjust_hue 等）
/// - `values/colors`：颜色值测试（alpha_hex, equality）
///
/// **注意**：颜色相关测试已标记 `#[ignore]`，需要用 `--ignored` 手动触发。
pub const SKIP_DIRS: &[&str] = &[
    // —— LibSass 弃用目录 ——
    "libsass",
    "libsass-closed-issues",
    "libsass-todo-issues",
    "libsass-todo-tests",
    // —— 不符合规范的旧测试 ——
    "non_conformant",
    // —— 颜色相关目录（防止无限修复循环） ——
    // core_functions/color 下所有子目录：
    // adjust, adjust_color, adjust_hue, change, channel, color,
    // hsl, hwb, invert, is_powerless, lab, lch, mix, oklab, oklch,
    // rgb, scale, to_gamut, to_space + 顶层 .hrx 文件
    "core_functions/color",
    // values/colors（alpha_hex, equality）
    "values/colors",
];

/// 检查文件相对 spec_root 的路径是否在跳过列表中。
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
/// 返回 (files, skipped_count)。
#[allow(dead_code)]
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
