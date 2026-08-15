//! 识别并列出 sass-spec 中 CSS 4.0 颜色相关的测试用例。
//!
//! CSS Color Level 4 引入了以下新特性：
//! - 新颜色函数: oklch(), oklab(), lch(), lab(), color()
//! - 新颜色空间: display-p3, a98-rgb, prophoto-rgb, rec2020, srgb-linear, xyz, xyz-d50, xyz-d65, oklab, oklch
//! - 新功能: color-mix(), to-gamut(), to-space(), channel(), relative-color syntax (from ...), is-in-gamut, is-powerless
//!
//! 用法:
//!   cargo run --example css4_color_skip -- /path/to/sass-spec/spec

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// CSS 4.0 相关的颜色空间和函数关键词
const CSS4_COLOR_KEYWORDS: &[&str] = &[
    // 新颜色函数 (CSS Color 4)
    "oklch(", "oklab(", "lch(", "lab(",
    // color() 函数的新颜色空间
    "color(", "color(from",
    // 新颜色空间名称
    "display-p3", "display_p3", "a98-rgb", "a98_rgb",
    "prophoto-rgb", "prophoto_rgb", "rec2020",
    "srgb-linear", "srgb_linear",
    "xyz-d50", "xyz-d65", "xyz_d50", "xyz_d65",
    // CSS 4 特有功能
    "to-gamut", "to-space", "to_gamut", "to_space",
    "is-in-gamut", "is_powerless", "is-in-gamut", "is-powerless",
    "color-mix", "color_mix",
    "channel(", "local-minde", "local_minde",
    // 相对颜色语法
    "from #", "from rgb", "from hsl", "from hwb", "from lab", "from lch", "from oklab", "from oklch",
];

/// 检测路径是否为 CSS 4.0 颜色相关
fn is_css4_color_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    let components: Vec<&str> = path_str.split('/').collect();

    // 检查是否在 color 或颜色相关目录下
    let in_color_dir = components.iter().any(|c| *c == "color" || *c == "colors");

    if !in_color_dir {
        return false;
    }

    // 检查文件名是否匹配 CSS 4.0 颜色关键词
    if let Some(filename) = path.file_name() {
        let name = filename.to_string_lossy().to_lowercase();
        let css4_spaces = [
            "oklch", "oklab", "lch", "lab",
            "display_p3", "display-p3", "a98_rgb", "a98-rgb",
            "prophoto_rgb", "prophoto-rgb", "rec2020",
            "srgb_linear", "srgb-linear",
            "xyz_d50", "xyz_d65", "xyz-d50", "xyz-d65",
            "to_gamut", "to-gamut", "to_space", "to-space",
            "is_in_gamut", "is-in-gamut", "is_powerless", "is-powerless",
            "relative_color", "relative-color",
        ];
        if css4_spaces.iter().any(|s| name.contains(s)) {
            return true;
        }
    }

    false
}

/// 检测文件内容是否包含 CSS 4.0 颜色功能
fn contains_css4_color_content(content: &str) -> bool {
    let lower = content.to_lowercase();

    // 检查新颜色函数调用
    if lower.contains("oklch(") || lower.contains("oklab(")
        || lower.contains("lch(") || lower.contains("lab(") {
        return true;
    }

    // color() 函数 + 新颜色空间
    if lower.contains("color(") {
        let new_spaces = [
            "display-p3", "display_p3", "a98-rgb", "a98_rgb",
            "prophoto-rgb", "prophoto_rgb", "rec2020",
            "srgb-linear", "srgb_linear",
            "xyz-d50", "xyz-d65", "xyz_d50", "xyz_d65",
            "oklab", "oklch", "lch", "lab",
        ];
        if new_spaces.iter().any(|s| lower.contains(s)) {
            return true;
        }
    }

    // 相对颜色语法
    if lower.contains("from #") || lower.contains("from rgb")
        || lower.contains("from hsl") || lower.contains("from oklab")
        || lower.contains("from oklch") || lower.contains("from lch")
        || lower.contains("from lab") {
        return true;
    }

    // CSS 4 特有功能
    if lower.contains("to-gamut") || lower.contains("to_gamut")
        || lower.contains("to-space") || lower.contains("to_space")
        || lower.contains("is-in-gamut") || lower.contains("is_powerless")
        || lower.contains("color-mix") || lower.contains("color_mix")
        || lower.contains("channel(") {
        return true;
    }

    false
}

/// 递归查找目录中所有 CSS 4.0 颜色相关的测试
fn find_css4_color_tests(dir: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    let mut visited = HashSet::new();

    visit_dir(dir, &mut results, &mut visited);

    results
}

fn visit_dir(dir: &Path, results: &mut Vec<PathBuf>, visited: &mut HashSet<PathBuf>) {
    if !dir.is_dir() {
        return;
    }
    if !visited.insert(dir.to_path_buf()) {
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            warn!(path = %dir.display(), error = %e, "failed to read directory");
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            // 递归处理子目录
            if is_css4_color_path(&path) {
                // 整个目录都是 CSS 4.0 颜色相关，收集所有 .hrx 文件
                collect_all_hrx(&path, results);
                info!(path = %path.display(), "found CSS 4.0 color directory");
            } else {
                visit_dir(&path, results, visited);
            }
        } else if path.extension().map_or(false, |e| e == "hrx") {
            // 检查文件名或内容
            if is_css4_color_path(&path) || hrx_contains_css4_color(&path) {
                results.push(path);
            }
        }
    }
}

/// 收集目录中的所有 HRX 文件
fn collect_all_hrx(dir: &Path, results: &mut Vec<PathBuf>) {
    if !dir.is_dir() {
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_all_hrx(&path, results);
        } else if path.extension().map_or(false, |e| e == "hrx") {
            results.push(path);
        }
    }
}

/// 检查 HRX 文件内容是否包含 CSS 4.0 颜色特性
fn hrx_contains_css4_color(path: &Path) -> bool {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    contains_css4_color_content(&content)
}

/// 分类统计
fn categorize_tests(tests: &[PathBuf]) -> Vec<(&str, Vec<PathBuf>)> {
    let mut categories: Vec<(&str, Vec<PathBuf>)> = vec![
        ("oklch/oklab (新感知颜色空间)", Vec::new()),
        ("lch/lab (CIELAB 颜色空间)", Vec::new()),
        ("display-p3 (广色域显示器)", Vec::new()),
        ("rec2020 (超高清电视)", Vec::new()),
        ("a98-rgb (Adobe RGB)", Vec::new()),
        ("prophoto-rgb (ProPhoto)", Vec::new()),
        ("srgb-linear (线性 sRGB)", Vec::new()),
        ("xyz-d50/d65 (CIE XYZ)", Vec::new()),
        ("to-gamut (色域映射)", Vec::new()),
        ("to-space (颜色空间转换)", Vec::new()),
        ("is-powerless (无力度检测)", Vec::new()),
        ("is-in-gamut (色域内检测)", Vec::new()),
        ("relative-color (相对颜色语法)", Vec::new()),
        ("其他 CSS 4.0 颜色功能", Vec::new()),
    ];

    for test in tests {
        let path_str = test.to_string_lossy().to_lowercase();

        if path_str.contains("oklch") || path_str.contains("oklab") {
            categories[0].1.push(test.clone());
        } else if path_str.contains("lch") || path_str.contains("/lab") || path_str.contains("lab/") {
            categories[1].1.push(test.clone());
        } else if path_str.contains("display_p3") || path_str.contains("display-p3") {
            categories[2].1.push(test.clone());
        } else if path_str.contains("rec2020") {
            categories[3].1.push(test.clone());
        } else if path_str.contains("a98") {
            categories[4].1.push(test.clone());
        } else if path_str.contains("prophoto") {
            categories[5].1.push(test.clone());
        } else if path_str.contains("srgb_linear") || path_str.contains("srgb-linear") {
            categories[6].1.push(test.clone());
        } else if path_str.contains("xyz") {
            categories[7].1.push(test.clone());
        } else if path_str.contains("to_gamut") || path_str.contains("to-gamut") {
            categories[8].1.push(test.clone());
        } else if path_str.contains("to_space") || path_str.contains("to-space") {
            categories[9].1.push(test.clone());
        } else if path_str.contains("is_powerless") || path_str.contains("is-powerless") {
            categories[10].1.push(test.clone());
        } else if path_str.contains("is_in_gamut") || path_str.contains("is-in-gamut") {
            categories[11].1.push(test.clone());
        } else if path_str.contains("relative_color") || path_str.contains("relative-color") {
            categories[12].1.push(test.clone());
        } else {
            categories[13].1.push(test.clone());
        }
    }

    categories
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let args: Vec<String> = std::env::args().collect();
    let spec_dir = args.get(1).map(Path::new).unwrap_or_else(|| {
        Path::new("/Users/pauljohn/rust/sass-spec-main/spec")
    });

    info!(path = %spec_dir.display(), "scanning for CSS 4.0 color tests");

    let tests = find_css4_color_tests(spec_dir);

    // 去重
    let mut unique_tests = tests;
    unique_tests.sort();
    unique_tests.dedup();

    info!(total = unique_tests.len(), "found CSS 4.0 color-related tests");

    // 分类统计
    let categories = categorize_tests(&unique_tests);

    println!("\n{}", "=".repeat(70));
    println!("  CSS 4.0 Color Level 4 - 需要跳过的测试用例");
    println!("{}\n", "=".repeat(70));

    let mut total_shown = 0;
    for (name, tests) in &categories {
        if !tests.is_empty() {
            println!("📦 {} ({} 个文件)", name, tests.len());
            for test in tests {
                let rel = test.strip_prefix(spec_dir).unwrap_or(test);
                println!("   - {}", rel.display());
                total_shown += 1;
            }
            println!();
        }
    }

    println!("{}", "-".repeat(70));
    println!("总计: {} 个 CSS 4.0 颜色相关测试文件需要跳过", total_shown);
    println!("{}", "-".repeat(70));

    // 生成 options.yml 格式的忽略列表
    println!("\n\n// 在 sass-spec 中标记为忽略的示例:");
    println!("---");
    println!(":ignore_for:");
    println!("- rust-sass");
    println!();

    // 输出忽略文件列表（用于批量处理）
    println!("// 需要添加到 :ignore_for 的测试路径列表:");
    for test in &unique_tests {
        let rel = test.strip_prefix(spec_dir).unwrap_or(test);
        let path_str = rel.to_string_lossy();
        // 移除 .hrx 后缀作为测试路径
        let test_path = if path_str.ends_with(".hrx") {
            &path_str[..path_str.len() - 4]
        } else {
            &path_str
        };
        println!("  - {}", test_path);
    }
}
