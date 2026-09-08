//! cf-noncolor-boost 统一诊断——收集 selector/meta/list/math/modules 失败模式。
//!
//! 用法：
//!   RUST_LOG=info cargo test --test tmp_cf_diag -- --nocapture

#![allow(clippy::case_sensitive_file_extension_comparisons)]

mod common;
use common::diff_css;

mod hrx_support;

use hrx_support::{HrxArchive, HrxEntry, Vfs, parse_hrx as hrx_parse};
use std::path::{Path, PathBuf};

/// HRX 测试用例。
struct HrxCase {
    files: Vec<(String, String)>,
    input_path: String,
    expected_output: String,
    expect_error: bool,
}

/// 按 `===` 分隔符将 HRX entries 分成独立组。
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
            .filter(|(p, _)| {
                (p.ends_with(".scss") || p.ends_with(".css") || p.ends_with(".sass"))
                    && !p.contains("/sass/")
            })
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

            cases.push(HrxCase {
                files: all_files.clone(),
                input_path,
                expected_output,
                expect_error,
            });
        }
    }
    cases
}

fn collect_hrx(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_hrx(&path, files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("hrx")
                && let Ok(meta) = std::fs::metadata(&path)
                && meta.len() < 50_000
            {
                files.push(path);
            }
        }
    }
}

/// 编译单个测试用例。
fn compile_case(
    case: &HrxCase,
    spec_root: &Path,
    hrx_dir: &Path,
    hrx_stem: &str,
) -> Result<String, String> {
    let tmp_dir = std::env::temp_dir().join(format!("cf-diag-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).ok();

    let case_subdir = tmp_dir.join(hrx_stem);
    std::fs::create_dir_all(&case_subdir).ok();
    for (path, content) in &case.files {
        let target = if path.starts_with(&format!("{hrx_stem}/")) {
            tmp_dir.join(path)
        } else {
            case_subdir.join(path)
        };
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&target, content).ok();
    }

    if let Ok(entries) = std::fs::read_dir(hrx_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if (p.extension().and_then(|s| s.to_str()) == Some(".scss")
                || p.extension().and_then(|s| s.to_str()) == Some(".css"))
                && let Ok(content) = std::fs::read_to_string(&p)
            {
                let filename = p.file_name().unwrap().to_string_lossy().to_string();
                std::fs::write(tmp_dir.join(&filename), content).ok();
            }
        }
    }

    let input_file = if case.input_path.starts_with(&format!("{hrx_stem}/")) {
        tmp_dir.join(&case.input_path)
    } else {
        case_subdir.join(&case.input_path)
    };

    let result = sasspile::compile_file_with_load_paths(
        &input_file,
        sasspile::OutputStyle::Expanded,
        vec![spec_root.to_path_buf()],
    );
    let _ = std::fs::remove_dir_all(&tmp_dir);
    result.map_err(|e| format!("{e}"))
}

/// 诊断指定子目录——显示前 N 个失败。
fn diag(subdir: &str, max_show: usize) {
    sasspile::init_tracing();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let dir = spec_root.join(subdir);
    let mut files = Vec::new();
    collect_hrx(&dir, &mut files);

    let mut shown = 0;
    let mut err_types: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for file in &files {
        if shown >= max_show {
            break;
        }
        if let Ok(content) = std::fs::read_to_string(file) {
            let stem = file.file_stem().unwrap().to_string_lossy().to_string();
            for case in &parse_hrx(&content) {
                if shown >= max_show {
                    break;
                }
                if case.expected_output.is_empty() && !case.expect_error {
                    continue;
                }
                let name = case
                    .input_path
                    .strip_suffix("input.scss")
                    .unwrap_or(&case.input_path)
                    .trim_end_matches('/')
                    .to_string();
                match compile_case(
                    case,
                    &spec_root,
                    file.parent().unwrap_or(Path::new(".")),
                    &stem,
                ) {
                    Ok(actual) => {
                        if case.expect_error {
                            shown += 1;
                            *err_types
                                .entry("expected_error_but_ok".to_string())
                                .or_default() += 1;
                            tracing::warn!(test = %format!("{stem}/{name}"), "FAIL: expected_error_but_ok");
                        } else if actual.trim() != case.expected_output.trim() {
                            shown += 1;
                            let diff = diff_css(case.expected_output.trim(), actual.trim());
                            let key = diff.classify();
                            *err_types.entry(key.to_string()).or_default() += 1;
                            tracing::warn!(test = %format!("{stem}/{name}"), kind = %key, n_diffs = diff.lines.len(), "FAIL");
                            for dl in diff.lines.iter().take(3) {
                                match dl {
                                    common::DiffLine::Changed {
                                        line,
                                        expected,
                                        actual,
                                    } => {
                                        tracing::debug!(line = line, expected = %expected, actual = %actual, "diff: changed");
                                    }
                                    common::DiffLine::ExtraExpected { line, content } => {
                                        tracing::debug!(line = line, expected = %content, actual = "(missing)", "diff: extra_expected");
                                    }
                                    common::DiffLine::ExtraActual { line, content } => {
                                        tracing::debug!(line = line, expected = "(missing)", actual = %content, "diff: extra_actual");
                                    }
                                }
                            }
                        }
                    }
                    Err(err_str) => {
                        if case.expect_error {
                            // 期望错误且确实出错了——通过
                        } else {
                            shown += 1;
                            let key =
                                if err_str.contains("Undefined") || err_str.contains("undefined") {
                                    "undefined".to_string()
                                } else if err_str.contains("Parse error")
                                    || err_str.contains("parse error")
                                    || err_str.contains("Syntax")
                                    || err_str.contains("syntax")
                                {
                                    "syntax".to_string()
                                } else if err_str.contains("Eval")
                                    || err_str.contains("eval")
                                    || err_str.contains("type")
                                    || err_str.contains("Type")
                                {
                                    "eval".to_string()
                                } else if err_str.contains("Module")
                                    || err_str.contains("module")
                                    || err_str.contains("Cannot")
                                    || err_str.contains("cannot")
                                {
                                    "module".to_string()
                                } else {
                                    "other_err".to_string()
                                };
                            *err_types.entry(key.clone()).or_default() += 1;
                            tracing::warn!(test = %format!("{stem}/{name}"), kind = %key, error = %err_str, "ERROR");
                        }
                    }
                }
            }
        }
    }

    tracing::info!(subdir = %subdir, "错误类型统计");
    for (k, v) in &err_types {
        tracing::info!(error_type = %k, count = *v, "错误类型");
    }
}

/// 统计指定子目录的通过/失败/总数。
fn stats_subdir_inner(subdir: &str) -> (usize, usize, usize) {
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let dir = spec_root.join(subdir);
    let mut files = Vec::new();
    collect_hrx(&dir, &mut files);
    let mut pass = 0;
    let mut fail = 0;
    let mut cases = 0;
    for file in &files {
        if let Ok(content) = std::fs::read_to_string(file) {
            let stem = file.file_stem().unwrap().to_string_lossy().to_string();
            for case in &parse_hrx(&content) {
                cases += 1;
                if case.expected_output.is_empty() && !case.expect_error {
                    continue;
                }
                match compile_case(
                    case,
                    &spec_root,
                    file.parent().unwrap_or(Path::new(".")),
                    &stem,
                ) {
                    Ok(actual) => {
                        if case.expect_error {
                            fail += 1;
                        } else if actual.trim() == case.expected_output.trim() {
                            pass += 1;
                        } else {
                            fail += 1;
                        }
                    }
                    Err(_) => {
                        if case.expect_error {
                            pass += 1;
                        } else {
                            fail += 1;
                        }
                    }
                }
            }
        }
    }
    (pass, fail, cases)
}

/// 统计指定子目录的通过/失败/总数（保留供外部调用）。
#[allow(dead_code)]
fn stats_subdir(subdir: &str) {
    let (pass, fail, cases) = stats_subdir_inner(subdir);
    let pct = if cases > 0 { pass * 100 / cases } else { 0 };
    tracing::info!(subdir = %subdir, pass = pass, total = cases, pct = pct, fail = fail, "子目录统计");
}

#[test]
fn diag_selector() {
    diag("core_functions/selector", 30);
}

#[test]
fn diag_meta() {
    diag("core_functions/meta", 30);
}

#[test]
fn diag_list() {
    diag("core_functions/list", 30);
}

#[test]
fn diag_math() {
    diag("core_functions/math", 30);
}

#[test]
fn diag_modules() {
    diag("core_functions/modules", 50);
}

#[test]
fn stats_selector() {
    sasspile::init_tracing();
    let (pass, fail, cases) = stats_subdir_inner("core_functions/selector");
    let pct = if cases > 0 { pass * 100 / cases } else { 0 };
    tracing::info!(pass = pass, total = cases, pct = pct, fail = fail, "selector 统计");
}

#[test]
fn stats_meta() {
    sasspile::init_tracing();
    let (pass, fail, cases) = stats_subdir_inner("core_functions/meta");
    let pct = if cases > 0 { pass * 100 / cases } else { 0 };
    tracing::info!(pass = pass, total = cases, pct = pct, fail = fail, "meta 统计");
}

#[test]
fn stats_list() {
    sasspile::init_tracing();
    let (pass, fail, cases) = stats_subdir_inner("core_functions/list");
    let pct = if cases > 0 { pass * 100 / cases } else { 0 };
    tracing::info!(pass = pass, total = cases, pct = pct, fail = fail, "list 统计");
}

#[test]
fn stats_math() {
    sasspile::init_tracing();
    let (pass, fail, cases) = stats_subdir_inner("core_functions/math");
    let pct = if cases > 0 { pass * 100 / cases } else { 0 };
    tracing::info!(pass = pass, total = cases, pct = pct, fail = fail, "math 统计");
}

#[test]
fn stats_modules() {
    sasspile::init_tracing();
    let (pass, fail, cases) = stats_subdir_inner("core_functions/modules");
    let pct = if cases > 0 { pass * 100 / cases } else { 0 };
    tracing::info!(pass = pass, total = cases, pct = pct, fail = fail, "modules 统计");
}
