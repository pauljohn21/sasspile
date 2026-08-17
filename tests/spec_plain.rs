//! Spec tests for `sass-spec/spec/css/plain/` domain.
//!
//! These tests parse HRX files from the plain CSS spec domain,
//! compile the input SCSS, and compare to expected output.

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
fn test_plain_boolean_operations() {
    let hrx_path = spec_root().join("css/plain/boolean_operations.hrx");
    if !hrx_path.exists() {
        return; // Skip if sass-spec not available
    }
    let results = spec_runner::run_hrx_tests(&hrx_path);
    for result in &results {
        assert!(
            result.passed,
            "Test '{}' failed: {}",
            result.name,
            result.message.as_deref().unwrap_or("(no message)")
        );
    }
}

#[test]
fn test_plain_slash() {
    let hrx_path = spec_root().join("css/plain/slash.hrx");
    if !hrx_path.exists() {
        return;
    }
    let results = spec_runner::run_hrx_tests(&hrx_path);
    for result in &results {
        assert!(
            result.passed,
            "Test '{}' failed: {}",
            result.name,
            result.message.as_deref().unwrap_or("(no message)")
        );
    }
}

#[test]
fn test_plain_null() {
    let hrx_path = spec_root().join("css/plain/null.hrx");
    if !hrx_path.exists() {
        return;
    }
    let results = spec_runner::run_hrx_tests(&hrx_path);
    for result in &results {
        assert!(
            result.passed,
            "Test '{}' failed: {}",
            result.name,
            result.message.as_deref().unwrap_or("(no message)")
        );
    }
}

#[test]
fn test_plain_calculation() {
    let hrx_path = spec_root().join("css/plain/calculation.hrx");
    if !hrx_path.exists() {
        return;
    }
    let results = spec_runner::run_hrx_tests(&hrx_path);
    for result in &results {
        assert!(
            result.passed,
            "Test '{}' failed: {}",
            result.name,
            result.message.as_deref().unwrap_or("(no message)")
        );
    }
}

#[test]
fn test_plain_if() {
    let hrx_path = spec_root().join("css/plain/if.hrx");
    if !hrx_path.exists() {
        return;
    }
    let results = spec_runner::run_hrx_tests(&hrx_path);
    for result in &results {
        assert!(
            result.passed,
            "Test '{}' failed: {}",
            result.name,
            result.message.as_deref().unwrap_or("(no message)")
        );
    }
}
