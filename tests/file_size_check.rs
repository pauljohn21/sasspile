//! 文件行数检测——确保所有 src/**/*.rs 文件 ≤ 500 行。
//!
//! AGENTS.md 规则：单文件 ≤ 500 行（源码和测试分别计算）。
//! 超限文件必须拆分后再编写。

use std::fs;
use std::path::Path;
use tracing;

/// 遍历 src/**/*.rs 文件，找出所有 >500 行的文件。
/// 返回 Vec<(文件路径, 行数)> — 空表示全部通过。
fn find_overlimit_files() -> Vec<(String, usize)> {
    let src_dir = Path::new("src");
    let mut overlimit = Vec::new();
    collect_files(src_dir, &mut overlimit);
    overlimit.sort_by(|a, b| b.1.cmp(&a.1)); // 按行数降序（最严重的排最前）
    overlimit
}

fn collect_files(dir: &Path, acc: &mut Vec<(String, usize)>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, acc);
        } else if path.extension().is_some_and(|e| e == "rs") {
            let Ok(content) = fs::read_to_string(&path) else { continue };
            let line_count = content.lines().count();
            if line_count > 500 {
                let rel = path.strip_prefix("src/").unwrap_or(&path);
                acc.push((rel.to_string_lossy().to_string(), line_count));
            }
        }
    }
}

#[test]
fn check_file_size_limits() {
    let overlimit = find_overlimit_files();
    if overlimit.is_empty() {
        return;
    }
    let mut msg = String::from("文件行数超限（上限 500 行）：\n");
    for (file, lines) in &overlimit {
        let excess = lines - 500;
        msg.push_str(&format!("  - {file}: {lines} 行，超出 {excess} 行\n"));
    }
    panic!("{}", msg);
}

#[test]
fn check_file_size_within_grace() {
    // 预警：列出 400-500 行的文件（接近上限，需关注）
    let src_dir = Path::new("src");
    let mut near_limit = Vec::new();
    collect_near_limit(src_dir, &mut near_limit);
    near_limit.sort_by(|a, b| b.1.cmp(&a.1));
    
    if !near_limit.is_empty() {
        tracing::error!("\n📏 文件行数预警（400-500 行，接近上限）：");
        for (file, lines) in &near_limit {
            tracing::error!("  - {file}: {lines} 行（距离上限 {} 行）", 500 - lines);
        }
    }
}

fn collect_near_limit(dir: &Path, acc: &mut Vec<(String, usize)>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_near_limit(&path, acc);
        } else if path.extension().is_some_and(|e| e == "rs") {
            let Ok(content) = fs::read_to_string(&path) else { continue };
            let line_count = content.lines().count();
            if line_count >= 400 && line_count <= 500 {
                let rel = path.strip_prefix("src/").unwrap_or(&path);
                acc.push((rel.to_string_lossy().to_string(), line_count));
            }
        }
    }
}
