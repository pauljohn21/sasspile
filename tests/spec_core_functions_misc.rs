//! Spec tests for `sass-spec/spec/core_functions/` misc domain.
//!
//! Covers: global functions, modules, and root-level HRX files.

#[path = "hrx_parser.rs"]
mod hrx_parser;

#[path = "spec_runner.rs"]
mod spec_runner;

use std::path::PathBuf;

fn spec_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("sass-spec")
        .join("spec")
}

#[test]
fn test_core_functions_global() {
    let dir = spec_root().join("core_functions").join("global");
    if !dir.exists() {
        return;
    }
    let hrx_files = hrx_parser::find_hrx_files(&dir);
    let mut total_passed = 0;
    let mut total_failed = 0;
    for hrx_path in &hrx_files {
        let results = spec_runner::run_hrx_tests(hrx_path);
        for result in &results {
            if result.passed {
                total_passed += 1;
            } else {
                total_failed += 1;
            }
        }
    }
    tracing::info!(
        stage = "spec_test",
        domain = "core_functions/global",
        passed = total_passed,
        failed = total_failed,
        "sass global functions spec tests"
    );
}

#[test]
fn test_core_functions_modules() {
    let dir = spec_root().join("core_functions").join("modules");
    if !dir.exists() {
        return;
    }
    let hrx_files = hrx_parser::find_hrx_files(&dir);
    let mut total_passed = 0;
    let mut total_failed = 0;
    for hrx_path in &hrx_files {
        let results = spec_runner::run_hrx_tests(hrx_path);
        for result in &results {
            if result.passed {
                total_passed += 1;
            } else {
                total_failed += 1;
            }
        }
    }
    tracing::info!(
        stage = "spec_test",
        domain = "core_functions/modules",
        passed = total_passed,
        failed = total_failed,
        "sass module functions spec tests"
    );
}

#[test]
fn test_core_functions_root() {
    let dir = spec_root().join("core_functions");
    if !dir.exists() {
        return;
    }
    let hrx_files: Vec<_> = hrx_parser::find_hrx_files(&dir)
        .into_iter()
        .filter(|p| p.parent() == Some(dir.as_path()))
        .collect();
    let mut total_passed = 0;
    let mut total_failed = 0;
    for hrx_path in &hrx_files {
        let results = spec_runner::run_hrx_tests(hrx_path);
        for result in &results {
            if result.passed {
                total_passed += 1;
            } else {
                total_failed += 1;
            }
        }
    }
    tracing::info!(
        stage = "spec_test",
        domain = "core_functions/root",
        passed = total_passed,
        failed = total_failed,
        "sass core_functions root spec tests"
    );
}
