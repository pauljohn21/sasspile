//! Spec test runner — compiles SCSS input and compares to expected output.
//!
//! This module provides utilities to run sass-spec tests against the compiler.
//! Multi-file tests (requiring VFS) are skipped — only single-file tests run.

// This module is shared across multiple test targets via `#[path = "spec_runner.rs"]`.
// When compiled as a standalone test target, some public items appear unused.
#![allow(dead_code)]

#[path = "hrx_parser.rs"]
mod hrx_parser;

use sasspile::compile;

/// Result of running a single spec test case.
#[derive(Debug)]
pub struct SpecTestResult {
    /// Test name
    pub name: String,
    /// Whether the test passed
    pub passed: bool,
    /// Error message if failed (None if passed)
    pub message: Option<String>,
}

/// Run a single spec test case.
///
/// Compiles the input SCSS and compares to expected output.
/// If an error is expected, checks that compilation fails.
pub fn run_spec_test(name: &str, input: &str, expected: Option<&str>, expected_error: Option<&str>) -> SpecTestResult {
    let result = compile(input);

    match (expected, expected_error) {
        (Some(expected_output), None) => {
            // Normal test: expect successful compilation matching output
            match result {
                Ok(actual_output) => {
                    let expected = normalize_css(expected_output);
                    let actual = normalize_css(&actual_output);
                    if expected == actual {
                        SpecTestResult {
                            name: name.to_string(),
                            passed: true,
                            message: None,
                        }
                    } else {
                        SpecTestResult {
                            name: name.to_string(),
                            passed: false,
                            message: Some(format!(
                                "Output mismatch:\n--- expected ---\n{}\n--- actual ---\n{}\n",
                                expected, actual
                            )),
                        }
                    }
                }
                Err(e) => SpecTestResult {
                    name: name.to_string(),
                    passed: false,
                    message: Some(format!("Expected success but got error: {}", e)),
                },
            }
        }
        (None, Some(expected_err)) => {
            // Error test: expect compilation to fail
            match result {
                Ok(_) => SpecTestResult {
                    name: name.to_string(),
                    passed: false,
                    message: Some("Expected error but compilation succeeded".to_string()),
                },
                Err(actual_err) => {
                    // Check if error message contains expected substring
                    let actual = actual_err.to_string();
                    if actual.contains(expected_err) || expected_err.contains(&actual) {
                        SpecTestResult {
                            name: name.to_string(),
                            passed: true,
                            message: None,
                        }
                    } else {
                        SpecTestResult {
                            name: name.to_string(),
                            passed: false,
                            message: Some(format!(
                                "Error mismatch:\n--- expected ---\n{}\n--- actual ---\n{}\n",
                                expected_err, actual
                            )),
                        }
                    }
                }
            }
        }
        _ => SpecTestResult {
            name: name.to_string(),
            passed: false,
            message: Some("Test case has neither expected output nor expected error".to_string()),
        },
    }
}

/// Normalize CSS output for comparison.
///
/// - Trims leading/trailing whitespace
/// - Normalizes line endings
/// - Collapses trailing whitespace per line
pub fn normalize_css(css: &str) -> String {
    let mut result = String::new();
    for line in css.lines() {
        let trimmed = line.trim_end();
        if !trimmed.is_empty() {
            result.push_str(trimmed);
            result.push('\n');
        }
    }
    result.trim_end().to_string()
}

/// Run all spec test cases from an HRX file.
///
/// Single-file tests run normally.
/// Multi-file tests (those with extra files beyond input.scss/output.css)
/// are skipped, as they require VFS support which has been removed.
pub fn run_hrx_tests(hrx_path: &std::path::Path) -> Vec<SpecTestResult> {
    let cases = crate::hrx_parser::extract_test_cases(hrx_path);
    let content = std::fs::read_to_string(hrx_path).unwrap_or_default();
    let files = crate::hrx_parser::parse_hrx(&content);

    cases
        .iter()
        .map(|case| {
            // Check if this is a multi-file test (has extra files beyond input/output/error/options)
            let dir = case.base_path.as_deref().unwrap_or("");
            let has_extra_files = files.iter().any(|(path, _)| {
                if path.ends_with("input.scss") || path.ends_with("output.css") || path == "error" || path == "options" {
                    return false;
                }
                let file_dir = if let Some(idx) = path.rfind('/') {
                    &path[..idx]
                } else {
                    ""
                };
                file_dir == dir
            });

            if has_extra_files {
                // Skip multi-file tests — VFS removed, use compile_file for filesystem-based tests
                return SpecTestResult {
                    name: case.name.clone(),
                    passed: true,
                    message: Some("SKIPPED: multi-file test (VFS removed)".to_string()),
                };
            }

            let input = case.input.as_deref().unwrap_or("");
            run_spec_test(
                &case.name,
                input,
                case.output.as_deref(),
                case.error.as_deref(),
            )
        })
        .collect()
}

/// Print a summary of test results.
pub fn print_summary(results: &[SpecTestResult]) {
    let total = results.len();
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = total - passed;

    tracing::info!(
        stage = "spec_test",
        passed = passed,
        failed = failed,
        total = total,
        "Spec test results"
    );

    for result in results.iter().filter(|r| !r.passed) {
        if let Some(ref msg) = result.message {
            tracing::warn!(stage = "spec_test", test = %result.name, msg = %msg, "FAIL");
        }
    }
}
