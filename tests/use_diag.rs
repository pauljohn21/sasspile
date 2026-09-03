//! directives/use 专用诊断测试——按子目录统计 pass/fail，输出失败案例路径。
//!
//! 用法：
//! ```bash
//! cargo test --test use_diag -- --nocapture
//! SHOW_FAILS=1 cargo test --test use_diag -- --nocapture  # 显示失败路径
//! ```

#![allow(dead_code)]

mod hrx_support;

use hrx_support::{parse_hrx_to_cases, run_case};
use std::path::{Path, PathBuf};
use tracing::{info, info_span};

// ─── 文件收集 ─────────────────────────────────────────────────────────────

/// 递归收集目录下所有 .hrx 文件（<100KB）。
fn collect_hrx_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_recursive(dir, &mut files);
    files
}

fn collect_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_recursive(&path, files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("hrx")
                && let Ok(meta) = std::fs::metadata(&path)
                && meta.len() < 100_000
            {
                files.push(path);
            }
        }
    }
}

// ─── 子目录发现 ───────────────────────────────────────────────────────────

/// 收集 `directives/use` 下的子目录列表（如 extend, with, error, css, member, ...）。
fn collect_subdirs(spec_root: &Path) -> Vec<String> {
    let use_dir = spec_root.join("directives/use");
    let mut subdirs = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&use_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir()
                && let Some(name) = path.file_name().and_then(|s| s.to_str())
            {
                subdirs.push(name.to_string());
            }
        }
    }
    subdirs.sort();
    subdirs
}

// ─── 统计运行 ─────────────────────────────────────────────────────────────

/// 单个子目录的统计结果。
struct SubdirStats {
    name: String,
    pass: usize,
    fail: usize,
    skip: usize,
    total: usize,
    failed_cases: Vec<String>,
}

/// 运行一个子目录下的所有 HRX 测试用例。
fn run_subdir(spec_root: &Path, subdir: &str) -> SubdirStats {
    let span = info_span!("run_subdir", subdir = %subdir);
    let _enter = span.enter();

    let dir = spec_root.join("directives/use").join(subdir);
    let files = collect_hrx_files(&dir);

    let (mut pass, mut fail, mut skip, mut total) = (0, 0, 0, 0);
    let mut failed_cases = Vec::new();

    for file in &files {
        if let Ok(content) = std::fs::read_to_string(file) {
            let rel_path = file
                .strip_prefix(spec_root)
                .unwrap_or(file)
                .to_string_lossy()
                .to_string();
            for case in &parse_hrx_to_cases(&content, &rel_path) {
                total += 1;
                if case.expected_output.is_empty() && !case.expect_error {
                    skip += 1;
                    continue;
                }
                if run_case(case) {
                    pass += 1;
                } else {
                    fail += 1;
                    failed_cases.push(case.input_path.clone());
                }
            }
        }
    }

    let evaluated = total - skip;
    let pct = pass * 100 / evaluated.max(1);
    info!(
        subdir = %subdir,
        pass, fail, skip, total, evaluated, pct,
        "子目录统计"
    );

    SubdirStats {
        name: subdir.to_string(),
        pass,
        fail,
        skip,
        total,
        failed_cases,
    }
}

// ─── 测试入口 ─────────────────────────────────────────────────────────────

#[test]
fn test_use_subdirs() {
    sasspile::init_tracing_otel();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let use_dir = spec_root.join("directives/use");

    if !use_dir.exists() {
        info!("directives/use 目录不存在，跳过");
        return;
    }

    let subdirs = collect_subdirs(&spec_root);
    info!(subdirs = ?subdirs, "发现的子目录");

    let (mut tp, mut tf, mut ts, mut tc) = (0, 0, 0, 0);
    let mut all_results: Vec<SubdirStats> = Vec::new();

    for subdir in &subdirs {
        let stats = run_subdir(&spec_root, subdir);
        tp += stats.pass;
        tf += stats.fail;
        ts += stats.skip;
        tc += stats.total;
        all_results.push(stats);
    }

    // 汇总
    let evaluated = tc - ts;
    let pct = tp * 100 / evaluated.max(1);
    info!(
        pass = tp,
        fail = tf,
        skip = ts,
        total = tc,
        evaluated,
        pct,
        "directives/use 汇总"
    );

    // 输出失败案例
    let show_fails = std::env::var("SHOW_FAILS").is_ok();
    for stats in &all_results {
        if stats.fail > 0 {
            info!(
                subdir = %stats.name,
                fail = stats.fail,
                pass = stats.pass,
                pct = stats.pass * 100 / (stats.total - stats.skip).max(1),
                "失败子目录"
            );
            if show_fails {
                for case_path in &stats.failed_cases {
                    info!(subdir = %stats.name, case = %case_path, "FAIL");
                }
            }
        }
    }
}

/// 顶层 HRX 文件统计（directives/use 下的 .hrx 文件）。
#[test]
fn test_use_top_level_hrx() {
    sasspile::init_tracing_otel();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let use_dir = spec_root.join("directives/use");

    if !use_dir.exists() {
        info!("directives/use 目录不存在，跳过");
        return;
    }

    // 收集顶层 .hrx 文件
    let mut hrx_files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&use_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("hrx")
                && let Ok(meta) = std::fs::metadata(&path)
                && meta.len() < 100_000
            {
                hrx_files.push(path);
            }
        }
    }

    hrx_files.sort();

    let (mut tp, mut tf, mut ts, mut tc) = (0, 0, 0, 0);

    for hrx_file in &hrx_files {
        let hrx_name = hrx_file
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let content = match std::fs::read_to_string(hrx_file) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let rel_path = hrx_file
            .strip_prefix(&spec_root)
            .unwrap_or(hrx_file)
            .to_string_lossy()
            .to_string();

        let (mut hp, mut hf, mut hs, mut hc) = (0, 0, 0, 0);
        for case in &parse_hrx_to_cases(&content, &rel_path) {
            hc += 1;
            if case.expected_output.is_empty() && !case.expect_error {
                hs += 1;
                continue;
            }
            if run_case(case) {
                hp += 1;
            } else {
                hf += 1;
                if std::env::var("SHOW_FAILS").is_ok() {
                    info!(hrx = %hrx_name, case = %case.input_path, "FAIL");
                }
            }
        }

        let heval = hc - hs;
        let hpct = hp * 100 / heval.max(1);
        info!(
            hrx = %hrx_name,
            pass = hp, fail = hf, skip = hs, total = hc,
            evaluated = heval, pct = hpct,
            "顶层HRX"
        );

        tp += hp;
        tf += hf;
        ts += hs;
        tc += hc;
    }

    let evaluated = tc - ts;
    let pct = tp * 100 / evaluated.max(1);
    info!(
        pass = tp,
        fail = tf,
        skip = ts,
        total = tc,
        evaluated,
        pct,
        "顶层HRX汇总"
    );
}
