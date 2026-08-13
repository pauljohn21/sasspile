//! element-plus 全量编译验证测试。
//!
//! 使用 compile_batch API 批量编译所有 EP SCSS 文件。

use sasspile::*;
use std::path::PathBuf;

/// 批量编译 element-plus 全部 SCSS 文件。
#[test]
fn test_ep_full_stats() {
    init_tracing();
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("ep")
        .join("packages")
        .join("theme-chalk")
        .join("src");

    // 收集目录下所有 SCSS 文件
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("无法读取 element-plus 目录")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "scss"))
        .collect();
    files.sort();

    let file_count = files.len();
    let result = compile_batch(&files, OutputStyle::Expanded);

    let mut ok = 0;
    let mut fail = 0;
    let mut categories: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

    for (name, res) in &result.outputs {
        match res {
            Ok(css) => {
                ok += 1;
                tracing::info!(file = %name, bytes = css.len(), "OK");
            }
            Err(e) => {
                fail += 1;
                let msg = format!("{e}");
                let cat = if msg.contains("未定义函数") {
                    "未定义函数"
                } else if msg.contains("未定义") || msg.contains("Undefined") {
                    "未定义"
                } else if msg.contains("不是") || msg.contains("is not") {
                    "类型错误"
                } else if msg.contains("参数") || msg.contains("argument") {
                    "参数错误"
                } else if msg.contains("解析") || msg.contains("Parse") {
                    "解析错误"
                } else if msg.contains("求值") || msg.contains("eval") {
                    "求值错误"
                } else {
                    "其他"
                };
                categories.entry(cat.to_string()).or_default().push(name.clone());
                tracing::warn!(file = %name, error = %msg, "FAIL");
            }
        }
    }

    tracing::info!("=== 统计 ===");
    tracing::info!(ok = ok, total = ok + fail, fail = fail, "通过/失败");

    tracing::info!("=== 错误分类 ===");
    for (cat, cat_files) in &categories {
        tracing::info!(category = %cat, count = cat_files.len(), files = %cat_files.join(", "), "错误分类");
    }

    tracing::info!(ok = ok, total = file_count, "EP 通过");
    assert!(ok > 0, "EP 应有文件编译成功");
}
