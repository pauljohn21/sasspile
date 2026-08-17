//! Spec test runner — compiles SCSS input and compares to expected output.
//!
//! This module provides utilities to run sass-spec tests against the compiler.

#[path = "hrx_parser.rs"]
mod hrx_parser;

use sasspile::{compile, compile_with_files};

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

/// Run a spec test case with a virtual file system.
///
/// The VFS maps module names to file content, enabling multi-file tests.
pub fn run_spec_test_with_vfs(
    name: &str,
    input: &str,
    vfs: &std::collections::HashMap<String, String>,
    expected: Option<&str>,
    expected_error: Option<&str>,
) -> SpecTestResult {
    let result = compile_with_files(input, vfs);

    match (expected, expected_error) {
        (Some(expected_output), None) => {
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
            match result {
                Ok(_) => SpecTestResult {
                    name: name.to_string(),
                    passed: false,
                    message: Some("Expected error but compilation succeeded".to_string()),
                },
                Err(actual_err) => {
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
fn normalize_css(css: &str) -> String {
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
/// Supports multi-file HRX tests by collecting all files (e.g. `plain.css`)
/// and passing them as a virtual file system to the compiler.
pub fn run_hrx_tests(hrx_path: &std::path::Path) -> Vec<SpecTestResult> {
    let cases = crate::hrx_parser::extract_test_cases(hrx_path);
    let content = std::fs::read_to_string(hrx_path).unwrap_or_default();
    let files = crate::hrx_parser::parse_hrx(&content);

    cases
        .iter()
        .map(|case| {
            // Build VFS from all files in the same directory as the test case
            let mut vfs: std::collections::HashMap<String, String> = std::collections::HashMap::new();
            let dir = case.base_path.as_deref().unwrap_or("");
            for (path, content) in &files {
                // Skip input.scss and output.css
                if path.ends_with("input.scss") || path.ends_with("output.css") || path == "error" || path == "options" {
                    continue;
                }
                // Extract base filename (with extension)
                let base_name = path.rsplit('/').next().unwrap_or(path);
                // Only include files in the same directory
                let file_dir = if let Some(idx) = path.rfind('/') {
                    &path[..idx]
                } else {
                    ""
                };
                if file_dir == dir {
                    vfs.insert(base_name.to_string(), content.clone());
                }
            }

            let input = case.input.as_deref().unwrap_or("");
            if vfs.is_empty() {
                run_spec_test(
                    &case.name,
                    input,
                    case.output.as_deref(),
                    case.error.as_deref(),
                )
            } else {
                run_spec_test_with_vfs(
                    &case.name,
                    input,
                    &vfs,
                    case.output.as_deref(),
                    case.error.as_deref(),
                )
            }
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
