//! Spec runner — executes sass-spec test cases and compares output.
//!
//! Iterates over loaded spec cases, runs the compiler on each input,
//! and checks if the output matches the expected CSS.

#[path = "hrx_loader.rs"]
mod hrx_loader;

use tracing::{info, info_span, instrument, warn};

use hrx_loader::{load_hrx_file, load_hrx_dir, SpecCase};

/// Result of a single spec test run.
#[derive(Debug, Clone)]
pub struct SpecResult {
    /// Test name.
    pub name: String,
    /// Whether the test passed.
    pub passed: bool,
    /// Actual output (if compilation succeeded).
    pub actual: Option<String>,
    /// Expected output.
    pub expected: String,
    /// Error message (if compilation failed).
    pub error: Option<String>,
}

/// Run a single spec case through the compiler.
#[instrument(skip(case))]
pub fn run_case(case: &SpecCase) -> SpecResult {
    let span = info_span!("spec_run", name = %case.name);
    let _enter = span.enter();

    // Compile the input.
    let actual = match compile_input(&case.input) {
        Ok(css) => css,
        Err(e) => {
            return SpecResult {
                name: case.name.clone(),
                passed: false,
                actual: None,
                expected: case.expected.clone(),
                error: Some(e),
            };
        }
    };

    // Normalize and compare.
    let actual_norm = normalize_css(&actual);
    let expected_norm = normalize_css(&case.expected);
    let passed = actual_norm == expected_norm;

    if passed {
        info!("spec case passed");
    } else {
        warn!(name = %case.name, "spec case failed");
    }

    SpecResult {
        name: case.name.clone(),
        passed,
        actual: Some(actual),
        expected: case.expected.clone(),
        error: None,
    }
}

/// Compile SCSS input to CSS.
fn compile_input(input: &str) -> Result<String, String> {
    let (stylesheet, _) = sasspile::parser::parse(input);
    sasspile::css::generate(&stylesheet, sasspile::css::OutputStyle::Expanded)
        .map_err(|e| format!("compile error: {e}"))
}

/// Normalize CSS for comparison (trim whitespace, normalize newlines).
fn normalize_css(css: &str) -> String {
    css.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Run all HRX files in a directory.
pub fn run_spec_dir(dir: impl AsRef<std::path::Path>) -> Vec<SpecResult> {
    let cases = load_hrx_dir(dir);
    let mut results = Vec::new();

    for case in cases {
        match case {
            Ok(c) => results.push(run_case(&c)),
            Err(e) => {
                warn!(error = %e, "failed to load spec case");
                results.push(SpecResult {
                    name: "load_error".to_string(),
                    passed: false,
                    actual: None,
                    expected: String::new(),
                    error: Some(e),
                });
            }
        }
    }

    results
}

/// Summarize spec results.
pub fn summarize(results: &[SpecResult]) -> (usize, usize) {
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    (passed, failed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_simple_case() {
        // Use a hex color to avoid named-color/$variable ambiguity in current parser.
        let case = SpecCase {
            name: "simple".to_string(),
            input: ".foo { color: #ff0000; }".to_string(),
            expected: ".foo {\n  color: #ff0000;\n}\n".to_string(),
            style: None,
        };
        let result = run_case(&case);
        assert!(result.passed, "simple case should pass: {:?}, actual: {:?}", result.error, result.actual);
    }

    #[test]
    fn run_failing_case() {
        let case = SpecCase {
            name: "wrong".to_string(),
            input: ".foo { color: red; }".to_string(),
            expected: ".foo { color: blue; }".to_string(),
            style: None,
        };
        let result = run_case(&case);
        assert!(!result.passed);
    }

    #[test]
    fn normalize_css_whitespace() {
        let a = ".foo {\n  color: red;\n}\n";
        let b = ".foo { color: red; }";
        assert_eq!(normalize_css(a), normalize_css(b));
    }

    #[test]
    fn summarize_results() {
        let results = vec![
            SpecResult {
                name: "a".to_string(),
                passed: true,
                actual: None,
                expected: String::new(),
                error: None,
            },
            SpecResult {
                name: "b".to_string(),
                passed: false,
                actual: None,
                expected: String::new(),
                error: Some("err".to_string()),
            },
            SpecResult {
                name: "c".to_string(),
                passed: true,
                actual: None,
                expected: String::new(),
                error: None,
            },
        ];
        let (passed, failed) = summarize(&results);
        assert_eq!(passed, 2);
        assert_eq!(failed, 1);
    }
}
