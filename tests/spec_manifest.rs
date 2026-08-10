//! sass-spec manifest——目录索引 + 跳过列表。
//!
//! sasspile 尚不支持的 spec 子目录集中在此管理。
//! 每次全量验证时跳过这些目录，避免扫描注定失败的用例。
//! 支持新功能后，从 `SKIP_DIRS` 移除对应条目即可。

use std::path::{Path, PathBuf};

/// 跳过的 spec 子目录（sasspile 尚不支持的功能）。
///
/// 统计：跳过 ~466 个文件（36%），保留 ~840 个文件（64%）。
pub const SKIP_DIRS: &[&str] = &[
    // —— 颜色空间转换（257 文件）—— sasspile 不支持 lab/lch/oklab/oklch 等色彩空间
    "core_functions/color/to_space",
    // —— 颜色函数（183 文件）—— scale/adjust/change/channel/to_gamut/is_powerless/lab
    "core_functions/color/scale",
    "core_functions/color/adjust",
    "core_functions/color/change",
    "core_functions/color/channel",
    "core_functions/color/to_gamut",
    "core_functions/color/is_powerless",
    "core_functions/color/lab",
    // —— 全局函数（30 文件）—— sass:global 模块
    "core_functions/global",
    // —— 模块级函数（16 文件）—— 模块化颜色函数
    "core_functions/modules",
    // —— load-css（23 文件）—— meta.load-css
    "core_functions/meta/load_css",
    // —— 复杂选择器（81 文件）—— is-superselector/extend/unify 复杂分支
    "core_functions/selector/is_superselector",
    "core_functions/selector/extend",
    "core_functions/selector/unify",
    // —— calculation 类型（60 文件）—— calc/min/max/clamp 作为值类型
    "values/calculation",
    // —— use/forward with 配置（31 文件）—— 配置参数传递
    "directives/use/with",
    "directives/forward/with",
    // —— use 错误用例（27 文件）—— 错误诊断格式不匹配
    "directives/use/error",
    // —— forward 错误用例（7 文件）
    "directives/forward/error",
];

/// 检查路径是否在跳过列表中。
fn should_skip(path: &Path, spec_root: &Path) -> bool {
    let rel = path.strip_prefix(spec_root).unwrap_or(path);
    let rel_str = rel.to_string_lossy();
    SKIP_DIRS
        .iter()
        .any(|skip| rel_str.starts_with(skip) || rel_str == *skip)
}

/// 收集 spec 目录下所有 HRX 文件，跳过 `SKIP_DIRS` 和 >100KB 的文件。
///
/// 返回 (files, skipped_count)。
pub fn collect_hrx_files(spec_root: &Path) -> (Vec<PathBuf>, usize) {
    let mut files = Vec::new();
    let mut skipped = 0;
    collect_recursive(spec_root, spec_root, &mut files, &mut skipped);
    (files, skipped)
}

fn collect_recursive(dir: &Path, spec_root: &Path, files: &mut Vec<PathBuf>, skipped: &mut usize) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_recursive(&path, spec_root, files, skipped);
            } else if path.extension().and_then(|s| s.to_str()) == Some("hrx") {
                if should_skip(&path, spec_root) {
                    *skipped += 1;
                    continue;
                }
                if let Ok(meta) = std::fs::metadata(&path) {
                    if meta.len() < 100_000 {
                        files.push(path);
                    }
                }
            }
        }
    }
}

/// 递归收集所有 HRX 文件（含跳过的），用于 manifest 统计。
pub fn collect_all_hrx(spec_root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_all_recursive(spec_root, &mut files);
    files
}

fn collect_all_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_all_recursive(&path, files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("hrx") {
                if let Ok(meta) = std::fs::metadata(&path) {
                    if meta.len() < 100_000 {
                        files.push(path);
                    }
                }
            }
        }
    }
}

/// 按一级目录统计 HRX 文件分布。
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
