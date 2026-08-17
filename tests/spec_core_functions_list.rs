//! Spec tests for `sass-spec/spec/core_functions/list/` domain.
//!
//! Covers: sass:list module functions (append, index, length, nth, etc.)

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
fn test_core_functions_list() {
    let dir = spec_root().join("core_functions").join("list");
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
        domain = "core_functions/list",
        passed = total_passed,
        failed = total_failed,
        "sass:list spec tests"
    );
}
