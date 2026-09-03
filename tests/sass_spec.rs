//! sass-spec 合规测试框架 v2——正确解析 HRX 格式。
//!
//! HRX 解析使用 `hrx_auditor` crate（VFS + parser），正确支持 `===` 多层嵌套。
//! 内存优化：逐文件处理，限制测试数量。

#![allow(clippy::cast_precision_loss)]

mod hrx_support;

use hrx_support::{HrxArchive, HrxEntry, Vfs, parse_hrx as hrx_parse};
use std::path::Path;

/// HRX 测试用例。
#[derive(Debug)]
struct HrxCase {
    name: String,
    input: String,
    expected_output: String,
    expect_error: bool,
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

        for (dir_path, files) in &dirs {
            let input_file = files.iter().find(|(f, _)| f == "input.scss");
            if input_file.is_none() {
                continue;
            }
            let (input_name, input_content) = input_file.unwrap();
            let expected_output = files
                .iter()
                .find(|(f, _)| f == "output.css")
                .map(|(_, c)| c.clone())
                .unwrap_or_default();
            let expect_error = files.iter().any(|(f, _)| f == "error");
            let name = if dir_path == "." {
                input_name.clone()
            } else {
                dir_path.clone()
            };

            cases.push(HrxCase {
                name,
                input: input_content.clone(),
                expected_output,
                expect_error,
            });
        }
    }
    cases
}

/// 运行单个测试用例——限制输入大小防止内存爆炸。
fn run_case(case: &HrxCase) -> Result<(), String> {
    if case.input.len() > 10000 {
        return Err(format!("输入过大跳过 [{}]", case.name));
    }

    let result = sasspile::compile_expanded(&case.input);

    if case.expect_error {
        match result {
            Ok(_) => Err(format!("期望失败但成功 [{}]", case.name)),
            Err(_) => Ok(()),
        }
    } else if case.expected_output.is_empty() {
        Ok(())
    } else {
        let actual = result.map_err(|e| format!("编译失败 [{}]: {e}", case.name))?;
        let actual_trimmed = actual.trim();
        let expected_trimmed = case.expected_output.trim();

        if actual_trimmed == expected_trimmed {
            Ok(())
        } else {
            Err(format!(
                "不匹配 [{}]: 期望 {} 字节, 实际 {} 字节",
                case.name,
                expected_trimmed.len(),
                actual_trimmed.len()
            ))
        }
    }
}

/// 递归运行目录——限制最大测试数。
fn run_dir(dir: &Path, max_tests: usize) -> (usize, usize, usize) {
    let mut passed = 0;
    let mut failed = 0;
    let mut total = 0;

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if total >= max_tests {
                break;
            }
            let path = entry.path();
            if path.is_dir() {
                let (p, f, t) = run_dir(&path, max_tests - total);
                passed += p;
                failed += f;
                total += t;
            } else if path.extension().and_then(|s| s.to_str()) == Some("hrx") {
                if let Ok(meta) = std::fs::metadata(&path)
                    && meta.len() > 100_000
                {
                    continue;
                }
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let cases = parse_hrx(&content);
                    for case in &cases {
                        if total >= max_tests {
                            break;
                        }
                        total += 1;
                        match run_case(case) {
                            Ok(()) => passed += 1,
                            Err(_) => failed += 1,
                        }
                    }
                }
            }
        }
    }
    (passed, failed, total)
}

// —— 子目录测试（限制 50 个）——

#[test]
fn test_operators() {
    sasspile::init_tracing();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec/operators");
    let (p, f, t) = run_dir(&dir, 50);
    tracing::info!("operators: {p}/{t} 通过, {f} 失败");
    assert!(t > 0, "无测试用例");
}

#[test]
fn test_css_basic() {
    sasspile::init_tracing();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec/css");
    let (p, f, t) = run_dir(&dir, 50);
    tracing::info!("css: {p}/{t} 通过, {f} 失败");
    assert!(t > 0, "无测试用例");
}

#[test]
fn test_directives_if() {
    sasspile::init_tracing();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec/directives/if");
    let (p, f, t) = run_dir(&dir, 50);
    tracing::info!("@if: {p}/{t} 通过, {f} 失败");
    assert!(t > 0, "无测试用例");
}

/// 诊断——显示测试失败的详细对比。
#[test]
fn test_css_diagnostic() {
    sasspile::init_tracing();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec/css");
    let mut shown = 0;
    diag_dir(&dir, &mut shown);
}

fn diag_dir(dir: &Path, shown: &mut usize) {
    if *shown >= 10 {
        return;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if *shown >= 10 {
                return;
            }
            let path = entry.path();
            if path.is_dir() {
                diag_dir(&path, shown);
            } else if path.extension().and_then(|s| s.to_str()) == Some("hrx") {
                if let Ok(meta) = std::fs::metadata(&path)
                    && meta.len() > 50000
                {
                    continue;
                }
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let cases = parse_hrx(&content);
                    let stem = path.file_stem().unwrap().to_string_lossy();
                    for case in &cases {
                        if *shown >= 10 {
                            return;
                        }
                        if case.expected_output.is_empty() {
                            continue;
                        }
                        match sasspile::compile_expanded(&case.input) {
                            Ok(actual) => {
                                let a = actual.trim();
                                let e = case.expected_output.trim();
                                if a != e {
                                    *shown += 1;
                                    tracing::warn!(test = %format!("{stem}/{}", case.name), input = %case.input.trim(), expected = %e, actual = %a, "FAIL");
                                }
                            }
                            Err(err) => {
                                *shown += 1;
                                tracing::warn!(test = %format!("{stem}/{}", case.name), input = %case.input.trim(), error = %err, "ERROR");
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 全量 sass-spec 合规快报——默认不运行，用 --ignored 手动触发。
#[test]
#[ignore = "sass-spec 全量合规快报需手动 --ignored 触发"]
fn test_sass_spec_summary() {
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let (passed, failed, total) = run_dir(&spec_root, 50);
    let compliance = if total > 0 {
        (passed as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    tracing::info!(
        passed = passed,
        failed = failed,
        total = total,
        compliance = format!("{compliance:.1}%"),
        "sass-spec 合规快报（前 {total} 个）"
    );
}
