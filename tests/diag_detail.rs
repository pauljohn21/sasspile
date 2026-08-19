//! 精准诊断——输出剩余失败的详细分类。
//!
//! HRX 解析使用 `hrx_auditor` crate（VFS + parser），正确支持 `===` 多层嵌套。

mod spec_manifest;

use hrx_auditor::parser::{parse_hrx as hrx_parse, HrxArchive, HrxEntry};
use hrx_auditor::vfs::Vfs;
use spec_manifest::collect_hrx_files;
use std::path::{Path, PathBuf};
use tracing::info;

type ParsedHrx = Vec<(Vec<(String, String)>, String, String, bool)>;

/// 按 `===` 分隔符将 HRX entries 分成独立组，每组构建自己的 VFS。
fn parse_hrx(content: &str) -> ParsedHrx {
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

        let all_files: Vec<(String, String)> = dirs
            .iter()
            .flat_map(|(dir_path, files)| {
                files.iter().map(move |(f, c)| {
                    if dir_path == "." {
                        (f.clone(), c.clone())
                    } else {
                        (format!("{dir_path}/{f}"), c.clone())
                    }
                })
            })
            .filter(|(p, _)| p.ends_with(".scss") || p.ends_with(".css"))
            .collect();

        for (dir_path, files) in &dirs {
            let input_file = files.iter().find(|(f, _)| f == "input.scss");
            if input_file.is_none() {
                continue;
            }
            let (input_name, _) = input_file.unwrap();
            let input_path = if dir_path == "." {
                input_name.clone()
            } else {
                format!("{dir_path}/{input_name}")
            };
            let expected_output = files
                .iter()
                .find(|(f, _)| f == "output.css")
                .map(|(_, c)| c.clone())
                .unwrap_or_default();
            let expect_error = files.iter().any(|(f, _)| f == "error");
            cases.push((all_files.clone(), input_path, expected_output, expect_error));
        }
    }
    cases
}

fn run_case(
    case: &(Vec<(String, String)>, String, String, bool),
    load_paths: &[PathBuf],
) -> Result<String, String> {
    let (files, input_path, _, expect_error) = case;

    let total_size: usize = files.iter().map(|(_, c)| c.len()).sum();
    if total_size > 50_000 {
        return Err("too_large".to_string());
    }

    let tmp_dir = std::env::temp_dir().join(format!("sass-diag-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).ok();
    for (path, content) in files {
        let file_path = tmp_dir.join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&file_path, content).ok();
    }

    let input_file = tmp_dir.join(input_path);
    let result = sasspile::compile_file_with_load_paths(
        &input_file,
        sasspile::OutputStyle::Expanded,
        load_paths.to_vec(),
    );
    let _ = std::fs::remove_dir_all(&tmp_dir);

    if *expect_error {
        match result {
            Ok(_) => Err("expected_error_but_got_ok".to_string()),
            Err(_) => Ok(String::new()),
        }
    } else {
        match result {
            Ok(actual) => {
                let (_, _, expected, _) = case;
                if actual.trim() == expected.trim() {
                    Ok(String::new())
                } else {
                    let a = actual.trim();
                    let e = expected.trim();
                    let diff_start = a.chars().zip(e.chars()).position(|(a, e)| a != e);
                    let ctx = if let Some(pos) = diff_start {
                        let a_ctx: String = a.chars().skip(pos.saturating_sub(20)).take(60).collect();
                        let e_ctx: String = e.chars().skip(pos.saturating_sub(20)).take(60).collect();
                        format!("actual_near=|{a_ctx}| expected_near=|{e_ctx}|")
                    } else if a.len() < e.len() {
                        format!("actual_shorter, actual_len={} expected_len={}", a.len(), e.len())
                    } else {
                        format!("expected_shorter, actual_len={} expected_len={}", a.len(), e.len())
                    };
                    Err(format!("output_mismatch: {ctx}"))
                }
            }
            Err(e) => {
                let msg = format!("{e}");
                if msg.contains("UndefinedFunction") {
                    Err("undef_function".to_string())
                } else if msg.contains("求值错误") {
                    Err(format!("eval: {}", msg.split(':').nth(1).unwrap_or("").trim().chars().take(100).collect::<String>()))
                } else if msg.contains("解析错误") || msg.contains("ParseError") {
                    Err("parse_error".to_string())
                } else {
                    Err(format!("other: {}", msg.chars().take(100).collect::<String>()))
                }
            }
        }
    }
}

#[test]
fn diag_output_mismatch() {
    sasspile::init_tracing();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");

    let dirs = [
        "core_functions",
        "values",
        "css",
        "directives",
        "expressions",
    ];

    let mut patterns: std::collections::BTreeMap<String, Vec<(String, String)>> = std::collections::BTreeMap::new();
    let mut error_counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();

    for dir_name in &dirs {
        let dir = spec_root.join(dir_name);
        if !dir.exists() {
            continue;
        }
        let (files, _skipped) = collect_hrx_files(&dir, &spec_root);

        for file in &files {
            if let Ok(content) = std::fs::read_to_string(file) {
                for case in &parse_hrx(&content) {
                    let (_, _, expected, expect_error) = case;
                    if expected.is_empty() && !expect_error {
                        continue;
                    }
                    let rel_file = file
                        .strip_prefix(&spec_root)
                        .unwrap_or(file)
                        .to_string_lossy()
                        .to_string();

                    match run_case(case, std::slice::from_ref(&spec_root)) {
                        Ok(_) => {}
                        Err(err) => {
                            *error_counts.entry(err.clone()).or_insert(0) += 1;
                            if err.starts_with("output_mismatch") || err.starts_with("eval: hsl") || err.starts_with("eval: hwb") || err.starts_with("eval: rgba") || err.starts_with("eval: alpha") || err.starts_with("eval: adjust") || err.starts_with("eval: scale") {
                                patterns.entry(err).or_default().push((rel_file, case.2.clone()));
                            }
                        }
                    }
                }
            }
        }
    }

    let mut sorted_errors: Vec<_> = error_counts.iter().collect();
    sorted_errors.sort_by(|a, b| b.1.cmp(a.1));
    info!("=== Top 30 错误类型 ===");
    for (i, (err, count)) in sorted_errors.iter().take(30).enumerate() {
        info!(rank = i + 1, error = err.as_str(), count, "错误");
    }

    let mut sorted_patterns: Vec<_> = patterns.iter().collect();
    sorted_patterns.sort_by_key(|b| std::cmp::Reverse(b.1.len()));
    info!("=== Top 15 失败模式详情 ===");
    for (i, (pattern, files)) in sorted_patterns.iter().take(15).enumerate() {
        info!(rank = i + 1, pattern = pattern.as_str(), count = files.len(), "模式");
        for (f, expected) in files.iter().take(2) {
            let exp_preview: String = expected.lines().take(5).collect::<Vec<_>>().join(" | ");
            info!(file = f.as_str(), expected = exp_preview.as_str(), "  详情");
        }
    }
}
