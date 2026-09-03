//! css 目录失败详情——逐个用例输出 expected vs actual 差异。
//!
//! HRX 解析使用 `hrx_auditor` crate（VFS + parser），正确支持 `===` 多层嵌套。

#![allow(clippy::case_sensitive_file_extension_comparisons)]

mod spec_manifest;

mod hrx_support;

use hrx_support::{HrxArchive, HrxEntry, Vfs, parse_hrx as hrx_parse};
use spec_manifest::collect_hrx_files;
use std::path::{Path, PathBuf};

struct HrxCase {
    files: Vec<(String, String)>,
    input_path: String,
    expected_output: String,
    expect_error: bool,
    name: String,
}

/// 按 `===` 分隔符将 HRX entries 分成独立组，每组构建自己的 VFS。
fn parse_hrx(content: &str) -> Vec<HrxCase> {
    let Ok(archive) = hrx_parse(content) else {
        return Vec::new();
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
            let name = if dir_path == "." {
                String::new()
            } else {
                dir_path.clone()
            };

            cases.push(HrxCase {
                files: all_files.clone(),
                input_path,
                expected_output,
                expect_error,
                name,
            });
        }
    }
    cases
}

fn run_case(case: &HrxCase, load_paths: &[PathBuf]) -> Option<String> {
    if case.expected_output.is_empty() && !case.expect_error {
        return None;
    }
    let total_size: usize = case.files.iter().map(|(_, c)| c.len()).sum();
    if total_size > 50_000 {
        return Some("TOO_LARGE".to_string());
    }

    let tmp_dir = std::env::temp_dir().join(format!("css-diag-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).ok();
    for (path, content) in &case.files {
        let file_path = tmp_dir.join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&file_path, content).ok();
    }

    let input_file = tmp_dir.join(&case.input_path);
    let result = sasspile::compile_file_with_load_paths(
        &input_file,
        sasspile::OutputStyle::Expanded,
        load_paths.to_vec(),
    );
    let _ = std::fs::remove_dir_all(&tmp_dir);

    match result {
        Ok(actual) => {
            if actual.trim() == case.expected_output.trim() {
                None
            } else {
                Some(format!(
                    "--- FAIL: {} ---\nEXPECTED:\n{}\nACTUAL:\n{}\n",
                    case.name,
                    case.expected_output.trim(),
                    actual.trim()
                ))
            }
        }
        Err(e) => {
            if case.expect_error {
                None
            } else {
                Some(format!(
                    "--- FAIL: {} ---\nEXPECTED:\n{}\nERROR:\n{}\n",
                    case.name,
                    case.expected_output.trim(),
                    e
                ))
            }
        }
    }
}

#[test]
fn css_fail_details() {
    sasspile::init_tracing();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let css_dir = spec_root.join("css");

    let (files, _) = collect_hrx_files(&css_dir, &spec_root);

    let mut fail_count = 0;
    let mut shown = 0;
    for file in &files {
        if let Ok(content) = std::fs::read_to_string(file) {
            for case in &parse_hrx(&content) {
                if let Some(diff) = run_case(case, std::slice::from_ref(&spec_root)) {
                    fail_count += 1;
                    if shown < 200 {
                        tracing::info!("\n{diff}");
                        shown += 1;
                    }
                }
            }
        }
    }
    tracing::info!(total_fails = fail_count, shown = shown, "css fail summary");
}
