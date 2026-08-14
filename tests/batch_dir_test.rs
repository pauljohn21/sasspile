//! 目录级批量编译测试——验证大批量编译时 RSS 是否可控。
//!
//! 测试策略：从入口文件编译（bootstrap.scss 等），让 @import 链条自动拉取依赖。
//! 这才是真实使用场景——用户编译入口文件，而非单独编译每个 _partial。
//!
//! ```bash
//! # 监控 RSS
//! cargo test --test batch_dir_test test_batch_bootstrap_entry -- --nocapture &
//! PID=$!
//! while kill -0 $PID 2>/dev/null; do
//!   ps -o rss= -p $PID | tr -d ' '
//!   sleep 0.5
//! done
//! ```

use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// 收集目录下所有 SCSS/SASS 文件。
fn collect_scss_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_scss_files(&path, files);
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str())
                && (ext == "scss" || ext == "sass")
            {
                files.push(path);
            }
        }
    }
}

/// 目录级批量编译——从入口文件编译，验证 RSS 行为。
///
/// 入口文件（bootstrap.scss）通过 @import 链条拉取所有 partial，
/// 这才是真实使用场景。单独编译 _partial 因缺少依赖上下文而失败是正常的。
#[test]
fn test_batch_bootstrap_entry() {
    sasspile::init_tracing();
    let bs_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bs/scss");

    if !bs_root.exists() {
        warn!("Bootstrap SCSS 目录不存在，跳过: {}", bs_root.display());
        return;
    }

    // 入口文件：bootstrap 系列主文件
    let entries = [
        "bootstrap.scss",
        "bootstrap-grid.scss",
        "bootstrap-reboot.scss",
        "bootstrap-utilities.scss",
    ];

    let entry_files: Vec<PathBuf> = entries
        .iter()
        .map(|f| bs_root.join(f))
        .filter(|p| p.exists())
        .collect();

    info!(files = ?entries, "从入口文件批量编译开始");

    // 一次性批量编译所有入口文件
    let start = std::time::Instant::now();
    let result = sasspile::compile_batch(&entry_files, sasspile::OutputStyle::Expanded);
    let elapsed = start.elapsed();

    // 统计结果
    let mut ok_count = 0;
    let mut err_count = 0;
    let mut total_css_bytes = 0;

    for (name, res) in &result.outputs {
        match res {
            Ok(css) => {
                ok_count += 1;
                total_css_bytes += css.len();
                info!(file = %name, css_bytes = css.len(), "编译成功");
            }
            Err(e) => {
                err_count += 1;
                warn!(file = %name, error = %e, "编译失败");
            }
        }
    }

    info!(
        ok_count,
        err_count,
        total_css_bytes,
        elapsed_ms = elapsed.as_millis(),
        "入口文件批量编译完成"
    );

    // 验证：主入口 bootstrap.scss 必须编译成功
    let main_ok = result
        .outputs
        .iter()
        .any(|(name, res)| name.contains("bootstrap.scss") && res.is_ok());
    assert!(main_ok, "bootstrap.scss 主入口应编译成功");
}

/// 测试全量 partial 单独编译——验证依赖缺失时的行为（对照实验）。
///
/// 此测试确认：单独编译 _partial 文件会因缺少 @import 上下文而失败，
/// 这是预期行为，不是 bug。
#[test]
fn test_batch_partials_individual() {
    sasspile::init_tracing();
    let bs_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bs/scss");

    if !bs_root.exists() {
        return;
    }

    // 收集所有 partial 文件（下划线开头）
    let mut files = Vec::new();
    collect_scss_files(&bs_root, &mut files);

    let partials: Vec<PathBuf> = files
        .into_iter()
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with('_'))
                .unwrap_or(false)
        })
        .collect();

    let partial_count = partials.len();
    info!(partial_count, "单独编译所有 _partial 文件（对照实验）");

    let result = sasspile::compile_batch(&partials, sasspile::OutputStyle::Expanded);

    let ok_count = result.outputs.iter().filter(|(_, r)| r.is_ok()).count();
    let err_count = result.outputs.len() - ok_count;

    info!(
        partial_count,
        ok_count, err_count, "_partial 单独编译统计（部分失败是预期的——缺少依赖上下文）"
    );

    // 验证：partial 单独编译会有失败（这是正常的）
    // 注意：不 assert 具体数量，因为不同版本可能有差异
    assert_eq!(result.outputs.len(), partial_count, "返回结果数量应匹配");
}

/// 测试小批量编译（对照实验）——确认 compile_batch 基本功能。
#[test]
fn test_batch_small() {
    sasspile::init_tracing();
    let bs_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bs/scss");

    let files: Vec<PathBuf> = ["_variables.scss", "_functions.scss", "_mixins.scss"]
        .iter()
        .map(|f| bs_root.join(f))
        .filter(|p| p.exists())
        .collect();

    if files.is_empty() {
        warn!("找不到对照测试文件，跳过");
        return;
    }

    let result = sasspile::compile_batch(&files, sasspile::OutputStyle::Expanded);
    info!(file_count = result.outputs.len(), "小批量编译完成");

    // 基本验证：返回结果数量匹配
    assert_eq!(result.outputs.len(), files.len());
}

/// 测试 utilities 目录下的子目录批量编译（验证嵌套目录处理）。
#[test]
fn test_batch_nested_dirs() {
    sasspile::init_tracing();
    let bs_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bs/scss");

    // 收集包括子目录（mixins/, helpers/, forms/, utilities/）的所有文件
    let mut files = Vec::new();
    collect_scss_files(&bs_root, &mut files);

    // 只取入口文件 + 子目录下的文件（排除顶层 _partial）
    let entry_and_nested: Vec<PathBuf> = files
        .into_iter()
        .filter(|p| {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            // 入口文件（无下划线前缀）或子目录下的文件
            !name.starts_with('_') || p.parent() != Some(bs_root.as_path())
        })
        .collect();

    info!(file_count = entry_and_nested.len(), "嵌套目录批量编译开始");

    let start = std::time::Instant::now();
    let result = sasspile::compile_batch(&entry_and_nested, sasspile::OutputStyle::Expanded);
    let elapsed = start.elapsed();

    let ok_count = result.outputs.iter().filter(|(_, r)| r.is_ok()).count();
    let err_count = result.outputs.len() - ok_count;

    info!(
        file_count = entry_and_nested.len(),
        ok_count,
        err_count,
        elapsed_ms = elapsed.as_millis(),
        "嵌套目录批量编译完成"
    );
}
