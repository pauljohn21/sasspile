//! core_functions/color 诊断——显示错误模式统计。
//!
//! HRX 解析使用 `hrx_auditor` crate（VFS + parser），正确支持 `===` 多层嵌套。
//! 使用 tracing 进行问题追踪，不使用 println!。

use hrx_auditor::parser::{parse_hrx as hrx_parse, HrxArchive, HrxEntry};
use hrx_auditor::vfs::Vfs;
use std::collections::HashMap;
use std::path::Path;

/// HRX 解析——按 `===` 分组，提取所有 (name, input, expected) 三元组。
fn parse_hrx(content: &str) -> Vec<(String, String, String)> {
    let archive = match hrx_parse(content) {
        Ok(a) => a,
        Err(_) => return Vec::new(),
    };

    let groups: Vec<Vec<HrxEntry>> = {
        let mut groups: Vec<Vec<HrxEntry>> = Vec::new();
        let mut current: Vec<HrxEntry> = Vec::new();
        for entry in archive.entries {
            if entry.path.is_empty() {
                if !current.is_empty() {
                    groups.push(std::mem::take(&mut current));
                }
            } else {
                current.push(entry);
            }
        }
        if !current.is_empty() {
            groups.push(current);
        }
        groups
    };

    let mut cases = Vec::new();
    for group_entries in &groups {
        let group_archive = HrxArchive {
            entries: group_entries.clone(),
        };
        let vfs = Vfs::from_archive(&group_archive);
        let dirs = vfs.walk();

        for (dir_path, files) in &dirs {
            let input_file = files.iter().find(|(f, _)| f == "input.scss");
            if input_file.is_none() {
                continue;
            }
            let (_, input_content) = input_file.unwrap();
            let output = files
                .iter()
                .find(|(f, _)| f == "output.css")
                .map(|(_, c)| c.clone())
                .unwrap_or_default();
            let has_error = files.iter().any(|(f, _)| f == "error");
            if !has_error && !output.is_empty() {
                let name = if dir_path == "." {
                    String::new()
                } else {
                    dir_path.clone()
                };
                cases.push((name, input_content.clone(), output));
            }
        }
    }
    cases
}

fn collect_hrx(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_hrx(&path, files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("hrx")
                && let Ok(meta) = std::fs::metadata(&path)
                    && meta.len() < 100_000 {
                        files.push(path);
                    }
        }
    }
}

#[test]
fn color_error_patterns() {
    let dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec/core_functions/color");
    let mut files = Vec::new();
    collect_hrx(&dir, &mut files);

    let mut pass = 0;
    let mut fail = 0;
    let mut patterns: HashMap<String, usize> = HashMap::new();

    for file in &files {
        if let Ok(content) = std::fs::read_to_string(file) {
            let stem = file.file_stem().unwrap().to_string_lossy().to_string();
            for (_name, input, expected) in &parse_hrx(&content) {
                match sasspile::compile_expanded(input) {
                    Ok(actual) => {
                        if actual.trim() == expected.trim() {
                            pass += 1;
                        } else {
                            fail += 1;
                            let a = actual.trim();
                            let e = expected.trim();
                            let _key = if a.is_empty() {
                                "empty".to_string()
                            } else if a.lines().next() != e.lines().next() {
                                "first_line".to_string()
                            } else {
                                "other".to_string()
                            };
                            *patterns.entry(format!("diff/{stem}")).or_default() += 1;
                        }
                    }
                    Err(err) => {
                        fail += 1;
                        let err_str = format!("{err}");
                        let prefix = if err_str.contains("未定义") {
                            let func = err_str.split("未定义函数: ").nth(1).unwrap_or("?");
                            format!("undef/{func}")
                        } else if err_str.contains("语法错误") {
                            "syntax".to_string()
                        } else if err_str.contains("求值错误") {
                            let msg = err_str.split("求值错误: ").nth(1).unwrap_or("?");
                            format!("eval/{msg}")
                        } else {
                            "other_err".to_string()
                        };
                        *patterns.entry(prefix).or_default() += 1;
                    }
                }
            }
        }
    }

    sasspile::init_tracing();
    tracing::info!(pass = pass, fail = fail, "color 诊断");
    tracing::info!("错误模式 (top 20):");
    let mut sorted: Vec<_> = patterns.into_iter().collect();
    sorted.sort_by_key(|b| std::cmp::Reverse(b.1));
    for (k, v) in sorted.iter().take(20) {
        tracing::info!(count = *v, pattern = %k, "错误模式");
    }
}
