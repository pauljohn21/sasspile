//! sass-spec 全量统计——使用 manifest 跳过不支持的目录。
//!
//! manifest 在 `tests/spec_manifest.rs` 中定义 `SKIP_DIRS` 跳过列表。
//! 支持新功能后，从 `SKIP_DIRS` 移除对应条目即可。
//!
//! 流式处理：逐文件读取 → 解析 → 编译 → drop，内存 O(1)。
//! 使用 jemalloc + 每文件 purge 释放物理内存，防止 macOS OOM。

mod spec_manifest;

use spec_manifest::collect_hrx_files;
use std::path::{Path, PathBuf};
use tracing::info;

// —— jemalloc 全局分配器 + 显式 purge ——
#[cfg(feature = "jemalloc")]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

/// 触发内存 pressure relief（macOS 专用 FFI）。
fn purge_memory() {
    #[cfg(target_os = "macos")]
    unsafe {
        unsafe extern "C" {
            fn malloc_zone_pressure_relief(zone: *mut libc::c_void, goal: libc::size_t) -> libc::size_t;
        }
        malloc_zone_pressure_relief(std::ptr::null_mut(), 0);
    }
}

/// HRX 测试用例。
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

/// 流式处理单个 HRX 文件——编译所有 case 后立即释放内存。
/// 关键：content 和 cases 在函数返回时 drop，不累积。
fn process_hrx_file(
    file_path: &Path,
    spec_root: &Path,
    pass: &mut usize,
    fail: &mut usize,
    skip: &mut usize,
    cases: &mut usize,
) {
    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let file_cases = parse_hrx(&content);
    drop(content); // 释放原始 HRX 内容

    for case in &file_cases {
        *cases += 1;
        if case.expected_output.is_empty() && !case.expect_error {
            *skip += 1;
            continue;
        }
        if run_case(case, &[spec_root.to_path_buf()]) {
            *pass += 1;
        } else {
            *fail += 1;
        }
    }
    // file_cases 在这里 drop，释放所有 case 数据
}

/// 按 spec 一级目录运行并统计（流式处理，O(1) 内存）。
fn run_spec_dir(spec_root: &Path, dir_name: &str) -> (usize, usize, usize, usize) {
    let dir = spec_root.join(dir_name);
    if !dir.exists() {
        return (0, 0, 0, 0);
    }

    let (files, skipped) = collect_hrx_files(&dir, spec_root);

    let (mut pass, mut fail, mut skip, mut cases) = (0, 0, 0, 0);
    for file in &files {
        process_hrx_file(file, spec_root, &mut pass, &mut fail, &mut skip, &mut cases);
        // 每个文件处理完，相关内存已释放
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

/// 处理单个目录——每个 HRX 文件编译后显式 drop。
fn run_spec_dir_chunked(spec_root: &Path, dir_name: &str) -> (usize, usize, usize, usize) {
    let dir = spec_root.join(dir_name);
    if !dir.exists() {
        return (0, 0, 0, 0);
    }

    let (files, skipped) = collect_hrx_files(&dir, spec_root);

    let (mut pass, mut fail, mut skip, mut cases) = (0, 0, 0, 0);
    for file in &files {
        process_hrx_file(file, spec_root, &mut pass, &mut fail, &mut skip, &mut cases);
        purge_memory(); // 每文件释放物理内存给 OS
    }
    // 释放文件列表
    drop(files);

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

/// 运行指定目录列表并汇总统计。
fn run_dirs(spec_root: &Path, dirs: &[&str]) -> (usize, usize, usize, usize) {
    let (mut total_pass, mut total_fail, mut total_skip, mut total_cases) = (0, 0, 0, 0);
    for dir in dirs {
        let (pass, fail, skip, cases) = run_spec_dir_chunked(spec_root, dir);
        total_pass += pass;
        total_fail += fail;
        total_skip += skip;
        total_cases += cases;
    }
    (total_pass, total_fail, total_skip, total_cases)
}

#[test]
#[ignore]
fn test_sass_spec_full_stats() {
    sasspile::init_tracing();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sass-spec-main/spec");

    // 基础目录（已通过，无 OOM）
    let (pass, fail, skip, cases) = run_dirs(
        &spec_root,
        &[
            "variables",
            "values",
            "css",
            "operators",
            "expressions",
            "directives",
        ],
    );

    let evaluated = cases - skip;
    let pct = pass * 100 / evaluated.max(1);
    info!(
        pass = pass,
        fail = fail,
        skip = skip,
        total = cases,
        evaluated = evaluated,
        pct = pct,
        "sass-spec 基础目录（无 OOM）"
    );
}

// core_functions 子目录逐个独立运行（完全隔离，避免累积 OOM）
macro_rules! core_fn_test {
    ($name:ident, $subdir:expr) => {
        #[test]
        #[ignore]
        fn $name() {
            sasspile::init_tracing();
            let spec_root =
                Path::new(env!("CARGO_MANIFEST_DIR")).join("../sass-spec-main/spec");
            let (pass, fail, skip, cases) =
                run_spec_dir_chunked(&spec_root, &format!("core_functions/{}", $subdir));
            let evaluated = cases - skip;
            let pct = pass * 100 / evaluated.max(1);
            info!(
                pass = pass,
                fail = fail,
                skip = skip,
                total = cases,
                evaluated = evaluated,
                pct = pct,
                subdir = $subdir,
                "core_functions 子目录"
            );
        }
    };
}

core_fn_test!(cf_general, "general.hrx");
core_fn_test!(cf_newlines, "newlines.hrx");
core_fn_test!(cf_list, "list");
core_fn_test!(cf_map, "map");
core_fn_test!(cf_math, "math");
core_fn_test!(cf_meta, "meta");
// selector 拆分为独立子目录（避免累积 OOM）
core_fn_test!(cf_selector_append, "selector/append.hrx");
core_fn_test!(cf_selector_nest, "selector/nest");
core_fn_test!(cf_selector_parse, "selector/parse");
core_fn_test!(cf_selector_replace, "selector/replace.hrx");
core_fn_test!(cf_selector_extend, "selector/extend");
core_fn_test!(cf_selector_unify, "selector/unify");
core_fn_test!(cf_selector_is_superselector, "selector/is_superselector");
core_fn_test!(cf_string, "string");

#[test]
#[ignore]
fn test_sass_spec_parser_callable() {
    sasspile::init_tracing();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sass-spec-main/spec");

    // parser + callable 单独运行
    let (pass, fail, skip, cases) =
        run_dirs(&spec_root, &["parser", "callable"]);

    let evaluated = cases - skip;
    let pct = pass * 100 / evaluated.max(1);
    info!(
        pass = pass,
        fail = fail,
        skip = skip,
        total = cases,
        evaluated = evaluated,
        pct = pct,
        "sass-spec parser + callable"
    );
}
