//! sass-spec manifest——目录索引。
//!
//! CSS4 色彩空间、未来特性文件级跳过。
//! 支持新功能后，从 SKIP_DIRS 或 CSS4_COLOR_PATTERNS 移除对应条目即可。

use std::path::{Path, PathBuf};

/// 跳过的 spec 子目录——整个目录都是 CSS4/未来特性/libsass 专用。
pub const SKIP_DIRS: &[&str] = &[
    // libsass 专用（对 sasspile 无用）
    "libsass",
    "libsass-closed-issues",
    "libsass-todo-issues",
    "libsass-todo-tests",
    "non_conformant",
    // CSS calc() 数学化简（需实现算术化简引擎，暂跳过）
    "values/calculation",
    // CSS4 色彩空间（整个目录都是）
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
    "core_functions/color/color",
    "core_functions/color/adjust_color",
    "core_functions/color/mixed_spaces",
    "core_functions/color/channel",
];

/// CSS4 色彩空间文件名模式——出现在 adjust/change/scale 等目录中。
/// 匹配文件名（不含路径）如果包含这些模式则跳过。
const CSS4_COLOR_PATTERNS: &[&str] = &[
    // CSS4 色彩空间
    "a98_rgb",
    "a98-rgb",
    "display_p3",
    "display-p3",
    "display_p3_linear",
    "prophoto_rgb",
    "prophoto-rgb",
    "rec2020",
    "srgb_linear",
    "srgb-linear",
    "xyz_d50",
    "xyz-d50",
    "xyz_d65",
    "xyz",
    // CSS4 色彩空间函数（hwb/lab/lch/oklab/oklch 在 adjust/change/scale 中）
    "hwb",
    "lab",
    "lch",
    "oklab",
    "oklch",
];

/// 检查文件相对 spec_root 的路径是否在跳过列表中。
fn should_skip(rel_path: &str) -> bool {
    // 目录级跳过
    if SKIP_DIRS
        .iter()
        .any(|skip| rel_path.starts_with(skip) || rel_path == *skip)
    {
        return true;
    }

    // 文件名级 CSS4 色彩空间模式匹配
    // 只对 core_functions/color 目录下的非 hsl/rgb/srgb 文件生效
    if rel_path.starts_with("core_functions/color/") {
        let file_name = Path::new(rel_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        // 跳过包含 CSS4 色彩空间模式的文件
        if CSS4_COLOR_PATTERNS
            .iter()
            .any(|pat| file_name.contains(pat))
        {
            // 但保留 hsl/rgb/srgb 相关的标准文件
            let is_standard = file_name == "hsl"
                || file_name == "rgb"
                || file_name == "srgb"
                || file_name == "alpha"
                || file_name == "hue"
                || file_name == "saturation"
                || file_name == "lightness"
                || file_name == "blue"
                || file_name == "green"
                || file_name == "red"
                || file_name == "complement"
                || file_name == "grayscale"
                || file_name == "invert"
                || file_name == "mix"
                || file_name == "scale"
                || file_name == "adjust"
                || file_name == "change"
                || file_name == "darken"
                || file_name == "lighten"
                || file_name == "saturate"
                || file_name == "desaturate"
                || file_name == "fade_in"
                || file_name == "fade_out"
                || file_name == "opacify"
                || file_name == "transparentize"
                || file_name == "adjust_hue";

            if !is_standard {
                return true;
            }
        }
    }

    false
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
                    && meta.len() < 50_000
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
