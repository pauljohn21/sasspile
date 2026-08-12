//! core_functions 诊断——显示前 N 个失败的摘要。
//! 集成 CSS diff 模块，逐行显示差异。
//! 支持跨文件 @use——写入临时目录后用 compile_file_with_load_paths 编译。
//!
//! 使用 tracing 进行问题追踪，不使用 println!。

mod common;
use common::diff_css;

use std::path::{Path, PathBuf};

/// HRX 测试用例——包含所有文件和期望输出。
struct HrxCase {
    /// 所有文件（路径 → 内容），用于写入临时目录。
    files: Vec<(String, String)>,
    /// input.scss 的路径。
    input_path: String,
    /// 期望输出 CSS。
    expected_output: String,
    /// 是否期望错误。
    expect_error: bool,
}

/// 解析 HRX——提取所有文件和测试用例。
fn parse_hrx(content: &str) -> Vec<HrxCase> {
    let mut files: Vec<(String, String)> = Vec::new();
    let mut current_path = String::new();
    let mut current_content = String::new();

    for line in content.lines() {
        if line.starts_with("<===>") {
            if !current_path.is_empty() {
                files.push((current_path.clone(), current_content));
            }
            current_path = line.trim_start_matches("<===>").trim().to_string();
            current_content = String::new();
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }
    if !current_path.is_empty() {
        files.push((current_path, current_content));
    }

    let mut cases = Vec::new();
    for (path, _input) in &files {
        if path.ends_with("input.scss") {
            let base = path.strip_suffix("input.scss").unwrap_or(path).to_string();
            let output_path = format!("{base}output.css");
            let error_path = format!("{base}error");

            let expected_output = files
                .iter()
                .find(|(p, _)| p == &output_path)
                .map(|(_, c)| c.clone())
                .unwrap_or_default();
            let expect_error = files.iter().any(|(p, _)| p == &error_path);

            // 收集所有 .scss/.css 文件（排除 sass: 内置模块引用）
            let case_files: Vec<(String, String)> = files
                .iter()
                .filter(|(p, _)| {
                    (p.ends_with(".scss") || p.ends_with(".css")) && !p.contains("/sass/")
                })
                .map(|(p, c)| (p.clone(), c.clone()))
                .collect();

            cases.push(HrxCase {
                files: case_files,
                input_path: path.clone(),
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
            } else if path.extension().and_then(|s| s.to_str()) == Some("hrx") {
                if let Ok(meta) = std::fs::metadata(&path) {
                    if meta.len() < 50_000 {
                        files.push(path);
                    }
                }
            }
        }
    }
}

/// 编译单个测试用例——写入临时目录后用 compile_file_with_load_paths 编译。
/// hrx_dir 是 HRX 文件所在目录，hrx_stem 是 HRX 文件名（不含扩展名）。
/// 将 HRX 内容写入 tmp_dir/<hrx_stem>/，同时复制 hrx_dir 下的 .scss 文件到 tmp_dir/，
/// 使 @use '../test-hue' 能正确解析到 tmp_dir/_test-hue.scss。
fn compile_case(case: &HrxCase, spec_root: &Path, hrx_dir: &Path, hrx_stem: &str) -> Result<String, String> {
    let tmp_dir = std::env::temp_dir().join(format!("cf-diag-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).ok();

    // 将 HRX 中的文件写入 tmp_dir/<hrx_stem>/
    let case_subdir = tmp_dir.join(hrx_stem);
    std::fs::create_dir_all(&case_subdir).ok();
    for (path, content) in &case.files {
        // 如果 path 本身就包含 hrx_stem/ 前缀，直接用 tmp_dir；否则用 case_subdir
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

    // 复制 hrx_dir 下的所有 .scss/.css 文件到 tmp_dir/（支持 @use '../xxx' 解析）
    if let Ok(entries) = std::fs::read_dir(hrx_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) == Some("scss")
                || p.extension().and_then(|s| s.to_str()) == Some("css")
            {
                if let Ok(content) = std::fs::read_to_string(&p) {
                    let filename = p.file_name().unwrap().to_string_lossy().to_string();
                    std::fs::write(tmp_dir.join(&filename), content).ok();
                }
            }
        }
    }

    // input_path 在 HRX 中可能只是 "input.scss"，需要映射到 case_subdir
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

fn diag(subdir: &str, max_show: usize) {
    sasspile::init_tracing();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sass-spec-main/spec");
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
                match compile_case(case, &spec_root, file.parent().unwrap_or(Path::new(".")), &stem) {
                    Ok(actual) => {
                        if case.expect_error {
                            // 期望错误但实际成功了
                            shown += 1;
                            *err_types.entry("expected_error_but_ok".to_string()).or_default() += 1;
                            tracing::warn!(test = %format!("{stem}/{name}"), "FAIL: expected_error_but_ok");
                        } else if actual.trim() != case.expected_output.trim() {
                            shown += 1;
                            let diff = diff_css(case.expected_output.trim(), actual.trim());
                            let key = diff.classify();
                            *err_types.entry(key.to_string()).or_default() += 1;
                            tracing::warn!(test = %format!("{stem}/{name}"), kind = %key, n_diffs = diff.lines.len(), "FAIL");
                            for dl in diff.lines.iter().take(3) {
                                match dl {
                                    common::DiffLine::Changed { line, expected, actual } => {
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
                            let key = if err_str.contains("未定义") {
                                "undefined".to_string()
                            } else if err_str.contains("语法错误") {
                                "syntax".to_string()
                            } else if err_str.contains("求值错误") {
                                "eval".to_string()
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

#[test]
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
    diag("directives/import", 15);
}

#[test]
fn diag_use() {
    diag("directives/use", 15);
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
    diag("directives/extend", 15);
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

#[test]
fn diag_values_colors() {
    diag("values/colors", 10);
}

/// 只统计指定子目录的通过/失败/总数。
fn stats_subdir(subdir: &str) {
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sass-spec-main/spec");
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
                match compile_case(case, &spec_root, file.parent().unwrap_or(Path::new(".")), &stem) {
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
