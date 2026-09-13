//! 文件行数检测——确保所有测试文件 ≤ 500 行。
//!
//! 从 compile_test.rs 拆出的文件检测函数，保持 compile_test.rs ≤ 500 行。

use std::fs;
use std::path::Path;

fn find_overlimit_files() -> Vec<(String, usize)> {
    let tests_dir = Path::new("tests");
    let mut overlimit = Vec::new();
    collect_files_overlimit(tests_dir, &mut overlimit);
    overlimit.sort_by(|a, b| b.1.cmp(&a.1));
    overlimit
}

fn collect_files_overlimit(dir: &Path, acc: &mut Vec<(String, usize)>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files_overlimit(&path, acc);
        } else if path.extension().is_some_and(|e| e == "rs") {
            let Ok(content) = fs::read_to_string(&path) else { continue };
            let line_count = content.lines().count();
            if line_count > 500 {
                let rel = path.strip_prefix("tests/").unwrap_or(&path);
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
    let tests_dir = Path::new("tests");
    let mut near_limit = Vec::new();
    collect_near_limit(tests_dir, &mut near_limit);
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
            if (400..=500).contains(&line_count) {
                let rel = path.strip_prefix("tests/").unwrap_or(&path);
                acc.push((rel.to_string_lossy().to_string(), line_count));
            }
        }
    }
}
