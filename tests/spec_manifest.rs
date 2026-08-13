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
    // —— scale/adjust/change 中的 3.x 色彩空间文件 ——
    "core_functions/color/scale/a98_rgb",
    "core_functions/color/scale/display_p3",
    "core_functions/color/scale/display_p3_linear",
    "core_functions/color/scale/lab",
    "core_functions/color/scale/lch",
    "core_functions/color/scale/no_space",
    "core_functions/color/scale/oklab",
    "core_functions/color/scale/oklch",
    "core_functions/color/scale/prophoto_rgb",
    "core_functions/color/scale/rec2020",
    "core_functions/color/scale/space",
    "core_functions/color/scale/srgb",
    "core_functions/color/scale/srgb_linear",
    "core_functions/color/scale/xyz_d50",
    "core_functions/color/scale/xyz_d65",
    "core_functions/color/adjust/a98_rgb",
    "core_functions/color/adjust/display_p3",
    "core_functions/color/adjust/display_p3_linear",
    "core_functions/color/adjust/lab",
    "core_functions/color/adjust/lch",
    "core_functions/color/adjust/oklab",
    "core_functions/color/adjust/oklch",
    "core_functions/color/adjust/prophoto_rgb",
    "core_functions/color/adjust/rec2020",
    "core_functions/color/adjust/srgb",
    "core_functions/color/adjust/srgb_linear",
    "core_functions/color/adjust/xyz_d50",
    "core_functions/color/adjust/space",
    "core_functions/color/change/a98_rgb",
    "core_functions/color/change/display_p3",
    "core_functions/color/change/display_p3_linear",
    "core_functions/color/change/lab",
    "core_functions/color/change/lch",
    "core_functions/color/change/no_space",
    "core_functions/color/change/oklab",
    "core_functions/color/change/oklch",
    "core_functions/color/change/prophoto_rgb",
    "core_functions/color/change/rec2020",
    "core_functions/color/change/space",
    "core_functions/color/change/srgb",
    "core_functions/color/change/srgb_linear",
    "core_functions/color/change/xyz",
    "core_functions/color/change/xyz_d50",
    "core_functions/color/to_space",
    "core_functions/color/channel",
    "core_functions/color/to_gamut",
    "core_functions/color/is_powerless",
    "core_functions/color/lab",
    // —— Sass 3.x 新色彩空间目录 ——
    "core_functions/color/lch",
    "core_functions/color/oklab",
    "core_functions/color/oklch",
    // —— Sass 3.x 新色彩 API 单文件 ——
    "core_functions/color/is_in_gamut",
    "core_functions/color/is_legacy",
    "core_functions/color/is_missing",
    "core_functions/color/space",
    "core_functions/color/same",
    "core_functions/color/ie_hex_str",
    // —— Sass 3.x color() 函数 + relative color + color spaces ——
    "core_functions/color/color",
    // —— hsl/rgb 中的 relative color 语法 ——
    "core_functions/color/hsl/one_arg/relative_color",
    "core_functions/color/rgb/one_arg/relative_color",
    // —— mix 中的 3.x 色彩空间方法 ——
    "core_functions/color/mix/mixed_spaces",
    "core_functions/color/mix/missing",
    "core_functions/color/mix/predefined",
    "core_functions/color/mix/hue_interpolation",
    // —— invert 中的 3.x 色彩空间参数 ——
    "core_functions/color/invert/modern",
    "core_functions/color/invert/named",
    // —— 全局函数（30 文件）—— sass:global 模块
    "core_functions/global",
    // —— 模块级函数（16 文件）—— 模块化颜色函数
    "core_functions/modules",
    // —— load-css（23 文件）—— meta.load-css
    "core_functions/meta/load_css",
    // —— 选择器函数已解锁（selector-is-superselector/unify/extend 已实现）——
    // "core_functions/selector/is_superselector",
    // "core_functions/selector/extend",
    // "core_functions/selector/unify",
    // —— calculation 类型（60 文件）—— calc/min/max/clamp 作为值类型
    "values/calculation",
    // —— use/forward with 配置（31 文件）—— 配置参数传递
    "directives/use/with",
    "directives/forward/with",
    // —— use 错误用例（27 文件）—— 错误诊断格式不匹配
    "directives/use/error",
    // —— forward 错误用例（7 文件）
    "directives/forward/error",
    // —— Sass 3.x meta 新 API（未实现）——
    // meta.calc-args() / meta.calc-name()：calculation 内省（Dart Sass 1.8.0+）
    "core_functions/meta/calc_args",
    "core_functions/meta/calc_name",
    // meta.module-variables/mixins/functions()：模块内省（Dart Sass 1.7.0+）
    "core_functions/meta/module_functions",
    "core_functions/meta/module_mixins",
    "core_functions/meta/module_variables",
    // meta.apply()：通过 mixin 引用调用（Dart Sass 1.7.0+）
    "core_functions/meta/apply",
    // meta.get-mixin()：获取 mixin 引用（Dart Sass 1.7.0+）
    "core_functions/meta/get_mixin",
    // meta.accepts-content()：检查 mixin 是否接受 @content（Dart Sass 1.7.0+）
    "core_functions/meta/accepts_content",
    // meta.inspect(mixin)：依赖 get-mixin（Dart Sass 1.7.0+）
    "core_functions/meta/inspect/mixin",
    // —— Sass 3.x .import.scss 迁移特性 ——
    // @import 配置通过 .import.scss 文件转发（3.x 迁移机制）
    "directives/import/configuration",
    // —— CSS calculation 简化（依赖 Calculation 值类型）——
    "css/plain/calculation",
];

/// 检查文件相对 spec_root 的路径是否在跳过列表中。
fn should_skip(rel_path: &str) -> bool {
    SKIP_DIRS
        .iter()
        .any(|skip| rel_path.starts_with(skip) || rel_path == *skip)
}

/// 收集 spec 目录下所有 HRX 文件，跳过 `SKIP_DIRS` 和 >100KB 的文件。
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
                    && meta.len() < 100_000 {
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
