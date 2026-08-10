//! sass-spec 全量统计——使用 manifest 跳过不支持的目录。
//!
//! manifest 在 `tests/spec_manifest.rs` 中定义 `SKIP_DIRS` 跳过列表。
//! 支持新功能后，从 `SKIP_DIRS` 移除对应条目即可。

mod spec_manifest;

use spec_manifest::collect_hrx_files;
use std::path::Path;
use tracing::info;

/// HRX 测试用例——包含所有文件和期望输出。
struct HrxCase {
    files: Vec<(String, String)>,
    input_path: String,
    expected_output: String,
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

/// 运行单个测试用例——写入临时目录并用 compile_file 编译。
fn run_case(case: &HrxCase) -> bool {
    if case.expected_output.is_empty() && !case.expect_error {
        return true;
    }

    let total_size: usize = case.files.iter().map(|(_, c)| c.len()).sum();
    if total_size > 50_000 {
        return false;
    }

    let tmp_dir = std::env::temp_dir().join(format!("sass-spec-{}", std::process::id()));
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
    let result = sasspile::compile_file(&input_file, sasspile::OutputStyle::Expanded);
    let _ = std::fs::remove_dir_all(&tmp_dir);

    if case.expect_error {
        result.is_err()
    } else {
        match result {
            Ok(actual) => actual.trim() == case.expected_output.trim(),
            Err(_) => false,
        }
    }
}

/// 按 spec 一级目录运行并统计。
fn run_spec_dir(spec_root: &Path, dir_name: &str) -> (usize, usize, usize, usize) {
    let dir = spec_root.join(dir_name);
    if !dir.exists() {
        return (0, 0, 0, 0);
    }

    // 使用 manifest 的 collect_hrx_files（自动跳过 SKIP_DIRS）
    let (files, skipped) = collect_hrx_files(&dir);

    let (mut pass, mut fail, mut skip, mut cases) = (0, 0, 0, 0);
    for file in &files {
        if let Ok(content) = std::fs::read_to_string(file) {
            for case in &parse_hrx(&content) {
                cases += 1;
                if case.expected_output.is_empty() && !case.expect_error {
                    skip += 1;
                    continue;
                }
                if run_case(case) {
                    pass += 1;
                } else {
                    fail += 1;
                }
            }
        }
    }

    let evaluated = cases - skip;
    let pct = if evaluated > 0 {
        pass * 100 / evaluated
    } else {
        0
    };
    info!(
        dir = dir_name,
        pass = pass,
        fail = fail,
        skip = skip,
        skipped_dirs = skipped,
        total = cases,
        pct = pct,
        "sass-spec 目录"
    );
    (pass, fail, skip, cases)
}

#[test]
fn test_import_use_forward() {
    sasspile::init_tracing();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sass-spec-main/spec");
    for subdir in &["directives/import", "directives/use", "directives/forward"] {
        let (pass, fail, skip, cases) = run_spec_dir(&spec_root, subdir);
        let evaluated = cases - skip;
        let pct = if evaluated > 0 {
            pass * 100 / evaluated
        } else {
            0
        };
        info!(
            subdir = subdir,
            pass = pass,
            fail = fail,
            skip = skip,
            total = cases,
            pct = pct,
            "import/use/forward"
        );
    }
}

#[test]
fn test_sass_spec_full_stats() {
    sasspile::init_tracing();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sass-spec-main/spec");

    // 所有 spec 一级目录（manifest 自动跳过不支持的功能）
    let dirs = [
        "variables",
        "values",
        "css",
        "operators",
        "expressions",
        "directives",
        "core_functions",
        "parser",
        "callable",
    ];

    let (mut total_pass, mut total_fail, mut total_skip, mut total_cases) = (0, 0, 0, 0);

    for dir in &dirs {
        let (pass, fail, skip, cases) = run_spec_dir(&spec_root, dir);
        total_pass += pass;
        total_fail += fail;
        total_skip += skip;
        total_cases += cases;
    }

    let evaluated = total_cases - total_skip;
    let overall_pct = if evaluated > 0 {
        total_pass * 100 / evaluated
    } else {
        0
    };
    info!(
        pass = total_pass,
        fail = total_fail,
        skip = total_skip,
        total = total_cases,
        evaluated = evaluated,
        pct = overall_pct,
        "sass-spec 全量统计（已跳过不支持的目录）"
    );
}
