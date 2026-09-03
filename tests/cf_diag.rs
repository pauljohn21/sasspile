//! `core_functions` 诊断——显示前 N 个失败的摘要。
//! 集成 CSS diff 模块，逐行显示差异。
//! 支持跨文件 @use——写入临时目录后用 `compile_file_with_load_paths` 编译。
//!
//! HRX 解析使用 `hrx_auditor` crate（VFS + parser），正确支持 `===` 多层嵌套。

mod common;
use common::diff_css;

mod hrx_support;

use hrx_support::{HrxArchive, HrxEntry, Vfs, parse_hrx as hrx_parse};
use std::path::{Path, PathBuf};

/// HRX 测试用例——包含所有文件和期望输出。
struct HrxCase {
    files: Vec<(String, String)>,
    input_path: String,
    expected_output: String,
    expect_error: bool,
}

/// 按 `===` 分隔符将 HRX entries 分成独立组，每组构建自己的 VFS。
fn parse_hrx(content: &str) -> Vec<HrxCase> {
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

/// 编译单个测试用例——写入临时目录后用 `compile_file_with_load_paths` 编译。
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
            if (p.extension().and_then(|s| s.to_str()) == Some("scss")
                || p.extension().and_then(|s| s.to_str()) == Some("css"))
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
    if let Ok(ref css) = result {
        tracing::trace!(input = %case.input_path, css = %css, "compile_case_ok");
    }
    let _ = std::fs::remove_dir_all(&tmp_dir);
    result.map_err(|e| format!("{e}"))
}

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

#[test]
fn diag_list() {
    diag("core_functions/list", 15);
}

#[test]
fn diag_selector() {
    diag("core_functions/selector", 15);
}

/// 颜色诊断——已跳过（颜色测试需手动 --ignored 触发）。
#[test]
#[ignore]
fn diag_color() {
    diag("core_functions/color", 15);
}

#[test]
fn diag_math() {
    diag("core_functions/math", 15);
}

#[test]
fn diag_expressions() {
    diag("expressions", 15);
}

#[test]
fn diag_meta() {
    diag("core_functions/meta", 15);
}

#[test]
fn diag_import() {
    diag("directives/import", 50);
}

#[test]
fn diag_use() {
    diag("directives/use", 50);
}

#[test]
fn diag_css() {
    diag("css", 20);
}

#[test]
fn diag_non_conformant() {
    diag("non_conformant", 20);
}

#[test]
fn diag_function() {
    diag("directives/function", 15);
}

#[test]
fn diag_extend() {
    diag("directives/extend", 50);
}

#[test]
fn diag_forward() {
    diag("directives/forward", 50);
}

#[test]
fn diag_numbers() {
    diag("values/numbers", 20);
}

#[test]
fn diag_libsass_closed() {
    diag("libsass-closed-issues", 20);
}

#[test]
fn diag_string() {
    diag("core_functions/string", 20);
}

#[test]
fn diag_map() {
    diag("core_functions/map", 15);
}

#[test]
fn diag_for() {
    diag("directives/for", 15);
}

#[test]
fn diag_each() {
    diag("directives/each", 15);
}

#[test]
fn diag_while() {
    diag("directives/while", 15);
}

#[test]
fn diag_media() {
    diag("directives/media", 15);
}

#[test]
fn diag_values_maps() {
    diag("values/maps", 10);
}

/// 颜色值诊断——已跳过（颜色测试需手动 --ignored 触发）。
#[test]
#[ignore]
fn diag_values_colors() {
    diag("values/colors", 10);
}

/// 只统计指定子目录的通过/失败/总数。
fn stats_subdir(subdir: &str) {
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
    let pct = if cases > 0 { pass * 100 / cases } else { 0 };
    tracing::info!(subdir = %subdir, pass = pass, total = cases, pct = pct, fail = fail, "子目录统计");
}

#[test]
fn stats_list() {
    stats_subdir("core_functions/list");
}

#[test]
fn stats_math() {
    stats_subdir("core_functions/math");
}
