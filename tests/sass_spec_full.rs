//! sass-spec 全量统计——使用 manifest 跳过不支持的目录。
//!
//! manifest 在 `tests/spec_manifest.rs` 中定义 `SKIP_DIRS` 跳过列表。
//! 支持新功能后，从 `SKIP_DIRS` 移除对应条目即可。

mod spec_manifest;

use spec_manifest::collect_hrx_files;
use std::path::{Path, PathBuf};
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
                .filter(|(p, _)| p.ends_with(".scss") || p.ends_with(".css"))
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
fn run_case(case: &HrxCase, load_paths: &[PathBuf]) -> bool {
    if case.expected_output.is_empty() && !case.expect_error {
        return true;
    }

    let total_size: usize = case.files.iter().map(|(_, c)| c.len()).sum();
    if total_size > 20_000 {
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
    let result = sasspile::compile_file_with_load_paths(
        &input_file,
        sasspile::OutputStyle::Expanded,
        load_paths.to_vec(),
    );
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
/// `max_files` 限制每个目录最多处理的文件数，防止内存爆炸。
fn run_spec_dir(spec_root: &Path, dir_name: &str) -> (usize, usize, usize, usize) {
    run_spec_dir_limited(spec_root, dir_name, usize::MAX)
}

fn run_spec_dir_limited(
    spec_root: &Path,
    dir_name: &str,
    max_files: usize,
) -> (usize, usize, usize, usize) {
    let dir = spec_root.join(dir_name);
    if !dir.exists() {
        return (0, 0, 0, 0);
    }

    // 使用 manifest 的 collect_hrx_files（自动跳过 SKIP_DIRS）
    let (files, skipped) = collect_hrx_files(&dir, spec_root);
    let files_to_process = files.len().min(max_files);
    let truncated = files.len().saturating_sub(max_files);

    let (mut pass, mut fail, mut skip, mut cases) = (0, 0, 0, 0);
    for file in &files[..files_to_process] {
        if let Ok(content) = std::fs::read_to_string(file) {
            for case in &parse_hrx(&content) {
                cases += 1;
                if case.expected_output.is_empty() && !case.expect_error {
                    skip += 1;
                    continue;
                }
                if run_case(case, &[spec_root.to_path_buf()]) {
                    pass += 1;
                } else {
                    fail += 1;
                }
            }
        }
    }

    let evaluated = cases - skip;
    let pct = pass * 100 / evaluated.max(1);
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
        let pct = pass * 100 / evaluated.max(1);
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
#[ignore]
fn test_sass_spec_full_stats() {
    sasspile::init_tracing();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sass-spec-main/spec");

    // 所有 spec 一级目录（manifest 自动跳过不支持的功能）
    // ⚠️ 仅前 3 个目录——全量运行内存消耗过大（>16GB）
    let dirs = [
        "variables",
        "values",
        "css",
        // "operators",
        // "expressions",
        // "directives",
        // "core_functions",
        // "parser",
        // "callable",
    ];

    // 每个目录最多处理的文件数——防止内存爆炸
    const MAX_FILES_PER_DIR: usize = 20;

    let (mut total_pass, mut total_fail, mut total_skip, mut total_cases) = (0, 0, 0, 0);

    for dir in &dirs {
        let (pass, fail, skip, cases) =
            run_spec_dir_limited(&spec_root, dir, MAX_FILES_PER_DIR);
        total_pass += pass;
        total_fail += fail;
        total_skip += skip;
        total_cases += cases;
    }

    let evaluated = total_cases - total_skip;
    let overall_pct = total_pass * 100 / evaluated.max(1);
    info!(
        pass = total_pass,
        fail = total_fail,
        skip = total_skip,
        total = total_cases,
        evaluated = evaluated,
        pct = overall_pct,
        max_files_per_dir = MAX_FILES_PER_DIR,
        "sass-spec 全量统计（每目录最多20文件）"
    );
}
