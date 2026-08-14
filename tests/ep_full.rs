//! element-plus 全量编译验证测试。
//!
//! 使用 compile_batch API 批量编译所有 EP SCSS 文件。

use sasspile::*;
use std::path::PathBuf;

// 内存监控 — tracing 事件 + 超限 panic
fn get_rss_mb() -> usize {
    let pid = std::process::id();
    let output = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &pid.to_string()])
        .output();
    match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout)
            .trim()
            .parse::<usize>()
            .unwrap_or(0),
        Err(_) => 0,
    }
}

fn start_ep_memory_monitor() {
    std::thread::spawn(|| {
        let mut warned = false;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(2));
            let rss = get_rss_mb();
            if rss > 4 * 1024 * 1024 {
                tracing::error!(rss_mb = rss, "💥 EP MEMORY OOM — auto-aborting");
                panic!("💥 EP OOM: RSS={rss} MB");
            } else if rss > 2 * 1024 * 1024 && !warned {
                tracing::warn!(rss_mb = rss, "⚠️ EP 内存增长中");
                warned = true;
            } else if rss <= 2 * 1024 * 1024 {
                warned = false;
            }
        }
    });
}

/// 批量编译 element-plus 全部 SCSS 文件。
#[test]
fn test_ep_full_stats() {
    init_tracing();
    start_ep_memory_monitor();
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
    let mut categories: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

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
                categories
                    .entry(cat.to_string())
                    .or_default()
                    .push(name.clone());
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
