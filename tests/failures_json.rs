//! sass-spec 全量失败原因导出为 JSON。
//!
//! 运行全部 sass-spec case，将每个失败的完整 expected/actual/error 写入
//! `tests/sass-spec-failures.json`。
//!
//! 用法：
//!   cargo test --test failures_json -- --nocapture

mod hrx_support;
mod spec_manifest;

use hrx_support::parse_hrx_to_cases;
use spec_manifest::SKIP_DIRS;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, info_span};

// ─── 数据结构 ─────────────────────────────────────────────────────────────

/// 单条失败记录。
#[derive(serde::Serialize)]
struct FailureRecord {
    id: String,
    dir: String,
    #[serde(rename = "type")]
    failure_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    actual: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// 单目录聚合统计。
#[derive(serde::Serialize, Default)]
struct DirAgg {
    diff: u32,
    err: u32,
    err_exp_ok: u32,
}

/// 顶层 JSON 输出结构。
#[derive(serde::Serialize)]
struct FailuresOutput {
    metadata: Metadata,
    failures: Vec<FailureRecord>,
    by_dir: BTreeMap<String, DirAgg>,
    by_type: BTreeMap<String, u32>,
}

#[derive(serde::Serialize)]
struct Metadata {
    timestamp: String,
    total_cases: u32,
    pass: u32,
    fail: u32,
    skip: u32,
}

// ─── 文件收集 ─────────────────────────────────────────────────────────────

fn collect_hrx_files_with_manifest(dir: &Path, spec_root: &Path) -> (Vec<PathBuf>, usize) {
    let all = collect_recursive(dir);
    let mut kept = Vec::new();
    let mut skipped = 0;
    for path in all {
        let rel = path
            .strip_prefix(spec_root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
        if SKIP_DIRS.iter().any(|s| rel.starts_with(s) || rel == *s) {
            skipped += 1;
            continue;
        }
        kept.push(path);
    }
    (kept, skipped)
}

fn collect_recursive(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_recursive_into(&path, &mut files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("hrx")
                && let Ok(meta) = std::fs::metadata(&path)
                && meta.len() < 100_000
            {
                files.push(path);
            }
        }
    }
    files
}

fn collect_recursive_into(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_recursive_into(&path, files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("hrx")
                && let Ok(meta) = std::fs::metadata(&path)
                && meta.len() < 100_000
            {
                files.push(path);
            }
        }
    }
}

// ─── 编译辅助 ─────────────────────────────────────────────────────────────

fn compile_case(files: &[(String, String)], input_path: &str) -> Result<String, String> {
    let tmp_dir = std::env::temp_dir().join(format!("fj-{}", std::process::id()));
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
    let load_paths = vec![tmp_dir.clone()];
    let result = sasspile::compile_file_with_load_paths(
        &input_file,
        sasspile::OutputStyle::Expanded,
        load_paths,
    );
    let _ = std::fs::remove_dir_all(&tmp_dir);
    result.map_err(|e| format!("{e}"))
}

// ─── 主逻辑 ───────────────────────────────────────────────────────────────

const OUTPUT_DIRS: &[&str] = &[
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

fn run_all_dirs(spec_root: &Path, output_path: &Path) {
    let span = info_span!("failures_json_export");
    let _enter = span.enter();

    let mut all_failures: Vec<FailureRecord> = Vec::new();
    let mut total_pass = 0u32;
    let mut total_fail = 0u32;
    let mut total_skip = 0u32;
    let mut total_cases = 0u32;

    for dir_name in OUTPUT_DIRS {
        let (pass, fail, skip, cases) = process_dir(spec_root, dir_name, &mut all_failures);
        info!(dir = dir_name, pass, fail, skip, cases, "目录处理完成");
        total_pass += pass;
        total_fail += fail;
        total_skip += skip;
        total_cases += cases;
    }

    // 聚合统计
    let by_dir = aggregate_by_dir(&all_failures);
    let by_type = aggregate_by_type(&all_failures);

    let output = FailuresOutput {
        metadata: Metadata {
            timestamp: chrono_now(),
            total_cases,
            pass: total_pass,
            fail: total_fail,
            skip: total_skip,
        },
        failures: all_failures,
        by_dir,
        by_type,
    };

    let json = serde_json::to_string_pretty(&output).expect("JSON 序列化失败");
    fs::write(output_path, json).expect("写入失败");
    info!(path = %output_path.display(), pass = total_pass, fail = total_fail, total = total_cases, "JSON 已写入");
}

fn process_dir(
    spec_root: &Path,
    dir_name: &str,
    failures: &mut Vec<FailureRecord>,
) -> (u32, u32, u32, u32) {
    let span = info_span!("process_dir", dir = dir_name);
    let _enter = span.enter();

    let dir = spec_root.join(dir_name);
    if !dir.exists() {
        return (0, 0, 0, 0);
    }

    let (files, _) = collect_hrx_files_with_manifest(&dir, spec_root);
    let mut pass = 0u32;
    let mut fail = 0u32;
    let mut skip = 0u32;
    let mut cases = 0u32;

    for file in &files {
        let Ok(content) = std::fs::read_to_string(file) else { continue; };
        let rel_path = file
            .strip_prefix(spec_root)
            .unwrap_or(file)
            .to_string_lossy()
            .to_string();

        for case in &parse_hrx_to_cases(&content, &rel_path) {
            // 跳过 .sass 缩进式语法
            if case.input_path.ends_with("input.sass") {
                continue;
            }
            // 跳过无期望输出的 case
            if case.expected_output.is_empty() && !case.expect_error {
                skip += 1;
                continue;
            }
            cases += 1;

            let dir_key = dir_name.to_string();
            let id = case.input_path.clone();

            match compile_case(&case.files, &case.input_path) {
                Ok(actual) => {
                    if case.expect_error {
                        // 期望错误但编译成功
                        fail += 1;
                        failures.push(FailureRecord {
                            id,
                            dir: dir_key,
                            failure_type: "ERR_EXP_OK".to_string(),
                            expected: None,
                            actual: Some(actual.trim().to_string()),
                            error: None,
                        });
                    } else if actual.trim() == case.expected_output.trim() {
                        pass += 1;
                    } else {
                        // DIFF
                        fail += 1;
                        failures.push(FailureRecord {
                            id,
                            dir: dir_key,
                            failure_type: "DIFF".to_string(),
                            expected: Some(case.expected_output.clone()),
                            actual: Some(actual.trim().to_string()),
                            error: None,
                        });
                    }
                }
                Err(err_str) => {
                    if case.expect_error {
                        pass += 1;
                    } else {
                        // 编译错误
                        fail += 1;
                        failures.push(FailureRecord {
                            id,
                            dir: dir_key,
                            failure_type: "ERR".to_string(),
                            expected: Some(case.expected_output.clone()),
                            actual: None,
                            error: Some(err_str),
                        });
                    }
                }
            }
        }
    }

    (pass, fail, skip, cases)
}

fn aggregate_by_dir(failures: &[FailureRecord]) -> BTreeMap<String, DirAgg> {
    let mut map: BTreeMap<String, DirAgg> = BTreeMap::new();
    for f in failures {
        let entry = map.entry(f.dir.clone()).or_default();
        match f.failure_type.as_str() {
            "DIFF" => entry.diff += 1,
            "ERR" => entry.err += 1,
            "ERR_EXP_OK" => entry.err_exp_ok += 1,
            _ => {}
        }
    }
    map
}

fn aggregate_by_type(failures: &[FailureRecord]) -> BTreeMap<String, u32> {
    let mut map: BTreeMap<String, u32> = BTreeMap::new();
    for f in failures {
        *map.entry(f.failure_type.clone()).or_default() += 1;
    }
    map
}

fn chrono_now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default()
}

// ─── 测试入口 ─────────────────────────────────────────────────────────────

#[test]
fn export_failures_json() {
    sasspile::init_tracing();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let output_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/sass-spec-failures.json");
    run_all_dirs(&spec_root, &output_path);
}
