//! Spec test runner with OTel Metrics + Trace integration.
//!
//! Wraps `spec_runner::run_spec_test` with:
//! - **Counter** (spec_tests_total) by domain + result
//! - **Histogram** (spec_test_duration_ms) by domain
//! - **ObservableGauge** (spec_pass_rate) registered at finalize
//! - `tracing::error!` on failure (no panic until final assert)
//! - VFS support for multi-file tests

#![allow(dead_code)]

#[path = "hrx_parser.rs"]
mod hrx_parser;

#[path = "hrx_vfs.rs"]
mod hrx_vfs;

#[path = "spec_runner.rs"]
mod spec_runner;

use std::path::Path;
use std::time::Instant;

use opentelemetry::KeyValue;
use opentelemetry::metrics::{Counter, Histogram, Meter};
use sasspile::compile_with_resolver;

/// Per-domain accumulated statistics.
#[derive(Debug, Clone, Default)]
pub struct DomainStats {
    pub total: u64,
    pub passed: u64,
    pub failed: u64,
    pub skipped: u64,
}

impl DomainStats {
    pub fn pass_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.passed as f64 / self.total as f64
        }
    }
}

/// Accumulated results for a domain.
#[derive(Debug, Clone, Default)]
pub struct DomainAccumulator {
    pub stats: DomainStats,
    /// Failed test names + mismatch details (for reporting).
    pub failures: Vec<(String, String)>,
}

/// The OTel-instrumented spec test runner.
pub struct SpecOtelRunner {
    /// Domain label (e.g. "css_plain").
    domain: String,
    /// OTel meter for recording metrics.
    meter: Meter,
    /// Counter for test results.
    counter: Counter<u64>,
    /// Histogram for test duration.
    duration_hist: Histogram<f64>,
    /// Accumulated stats.
    stats: DomainAccumulator,
}

impl SpecOtelRunner {
    /// Create a new runner for the given domain.
    pub fn new(domain: &str) -> Self {
        // tracing_init is included by the calling test file via #[path]
        // We obtain the global meter from OTel global API
        let meter = opentelemetry::global::meter_provider()
            .meter("sasspile-spec");
        let counter = meter
            .u64_counter("spec_tests_total")
            .with_description("Total spec test cases by domain and result")
            .build();
        let duration_hist = meter
            .f64_histogram("spec_test_duration_ms")
            .with_description("Spec test compilation duration in ms by domain")
            .with_unit("ms")
            .build();

        Self {
            domain: domain.to_string(),
            meter,
            counter,
            duration_hist,
            stats: DomainAccumulator::default(),
        }
    }

    /// Run a single spec test case with metrics + trace.
    pub fn run_spec_test(
        &mut self,
        name: &str,
        input: &str,
        expected: Option<&str>,
        expected_error: Option<&str>,
    ) -> bool {
        let span = tracing::info_span!(
            "spec_test",
            stage = "spec_test",
            domain = %self.domain,
            test_name = %name,
            result = tracing::field::Empty,
        );
        let _enter = span.enter();

        let start = Instant::now();
        let result = spec_runner::run_spec_test(name, input, expected, expected_error);
        let elapsed_ms = start.elapsed().as_millis() as f64;

        self.stats.stats.total += 1;

        let result_label = if result.passed {
            // Check if it was skipped
            if result
                .message
                .as_ref()
                .map(|m| m.starts_with("SKIPPED"))
                .unwrap_or(false)
            {
                "skip"
            } else {
                "pass"
            }
        } else {
            "fail"
        };

        match result_label {
            "pass" => self.stats.stats.passed += 1,
            "fail" => {
                self.stats.stats.failed += 1;
                self.stats.failures.push((
                    name.to_string(),
                    result.message.clone().unwrap_or_default(),
                ));
                // error! macro — does NOT panic, continues to next test
                tracing::error!(
                    stage = "spec_test",
                    domain = %self.domain,
                    test_name = %name,
                    msg = %result.message.as_deref().unwrap_or(""),
                    "FAIL"
                );
            }
            "skip" => self.stats.stats.skipped += 1,
            _ => {}
        }

        // Record metrics
        self.counter
            .add(1, &[KeyValue::new("domain", self.domain.clone()),
                       KeyValue::new("result", result_label)]);
        self.duration_hist
            .record(elapsed_ms, &[KeyValue::new("domain", self.domain.clone())]);

        tracing::info!(
            stage = "spec_test",
            domain = %self.domain,
            test_name = %name,
            result = result_label,
            elapsed_ms = elapsed_ms as u64,
            "test case complete"
        );

        result.passed
    }

    /// Run all test cases from an HRX file.
    /// Single-file tests use `compile()`, multi-file tests use VFS + `compile_with_resolver()`.
    pub fn run_hrx_tests(&mut self, hrx_path: &Path) {
        let hrx_name = hrx_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let span = tracing::info_span!(
            "spec_hrx",
            stage = "spec_test",
            domain = %self.domain,
            hrx_file = %hrx_name,
            case_count = tracing::field::Empty,
        );
        let _enter = span.enter();

        let content = std::fs::read_to_string(hrx_path).unwrap_or_default();
        let files = hrx_parser::parse_hrx(&content);
        let cases = hrx_parser::extract_test_cases(hrx_path);

        tracing::info!(
            stage = "spec_test",
            domain = %self.domain,
            hrx_file = %hrx_name,
            case_count = cases.len(),
            "running HRX file"
        );

        for case in &cases {
            // Check if this is a multi-file test
            let dir = case.base_path.clone().unwrap_or_default();
            let has_extra_files = files.iter().any(|(path, _)| {
                if path.ends_with("input.scss")
                    || path.ends_with("output.css")
                    || path == "error"
                    || path == "options"
                {
                    return false;
                }
                let file_dir = if let Some(idx) = path.rfind('/') {
                    &path[..idx]
                } else {
                    ""
                };
                file_dir == dir
            });

            let input = case.input.clone().unwrap_or_default();
            let output = case.output.clone();
            let error = case.error.clone();
            let case_name = case.name.clone();
            let files_clone = files.clone();
            let domain = self.domain.clone();

            // Wrap each test in catch_unwind to prevent compiler panics
            let test_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if has_extra_files {
                    self.run_multi_file_test(
                        &case_name,
                        &files_clone,
                        &dir,
                        &input,
                        output.as_deref(),
                        error.as_deref(),
                    );
                } else {
                    self.run_spec_test(
                        &case_name,
                        &input,
                        output.as_deref(),
                        error.as_deref(),
                    );
                }
            }));

            if let Err(panic_val) = test_result {
                self.stats.stats.total += 1;
                self.stats.stats.failed += 1;
                self.stats.failures.push((
                    case_name.clone(),
                    "COMPILER PANIC".to_string(),
                ));
                tracing::error!(
                    stage = "spec_test",
                    domain = %domain,
                    test_name = %case_name,
                    panic = %panic_val.downcast_ref::<String>()
                        .map(|s| s.as_str())
                        .unwrap_or("(non-string panic)"),
                    "COMPILER PANIC"
                );
                self.counter.add(
                    1,
                    &[
                        KeyValue::new("domain", domain.clone()),
                        KeyValue::new("result", "panic"),
                    ],
                );
            }
        }
    }

    /// Run a multi-file test case using VFS.
    fn run_multi_file_test(
        &mut self,
        name: &str,
        files: &[(String, String)],
        dir: &str,
        input: &str,
        expected: Option<&str>,
        expected_error: Option<&str>,
    ) {
        let span = tracing::info_span!(
            "spec_test",
            stage = "spec_test",
            domain = %self.domain,
            test_name = %name,
            mode = "vfs",
            result = tracing::field::Empty,
        );
        let _enter = span.enter();

        let start = Instant::now();

        // Build VFS from HRX files
        let vfs = hrx_vfs::build_vfs(files, dir);
        let mut resolver = hrx_vfs::VfsResolver::new(vfs);

        let compile_result = compile_with_resolver(input, &mut resolver);
        let elapsed_ms = start.elapsed().as_millis() as f64;

        self.stats.stats.total += 1;

        let (passed, result_label, message) = match (expected, expected_error, compile_result) {
            (Some(expected_output), None, Ok(actual_output)) => {
                let expected_norm = spec_runner::normalize_css(expected_output);
                let actual_norm = spec_runner::normalize_css(&actual_output);
                if expected_norm == actual_norm {
                    (true, "pass", None)
                } else {
                    (
                        false,
                        "fail",
                        Some(format!(
                            "Output mismatch:\n--- expected ---\n{}\n--- actual ---\n{}\n",
                            expected_norm, actual_norm
                        )),
                    )
                }
            }
            (None, Some(expected_err), Err(actual_err)) => {
                let actual = actual_err.to_string();
                if actual.contains(expected_err) || expected_err.contains(&actual) {
                    (true, "pass", None)
                } else {
                    (
                        false,
                        "fail",
                        Some(format!(
                            "Error mismatch:\n--- expected ---\n{}\n--- actual ---\n{}\n",
                            expected_err, actual
                        )),
                    )
                }
            }
            (_, _, Ok(_)) if expected.is_none() && expected_error.is_none() => {
                (false, "fail", Some("No expected output or error".to_string()))
            }
            (Some(_), None, Err(e)) => (
                false,
                "fail",
                Some(format!("Expected success but got error: {}", e)),
            ),
            (None, Some(_), Ok(_)) => (
                false,
                "fail",
                Some("Expected error but compilation succeeded".to_string()),
            ),
            _ => (
                false,
                "fail",
                Some("Test case has neither expected output nor expected error".to_string()),
            ),
        };

        if passed {
            self.stats.stats.passed += 1;
        } else {
            self.stats.stats.failed += 1;
            self.stats
                .failures
                .push((name.to_string(), message.clone().unwrap_or_default()));
            tracing::error!(
                stage = "spec_test",
                domain = %self.domain,
                test_name = %name,
                mode = "vfs",
                msg = %message.as_deref().unwrap_or(""),
                "FAIL"
            );
        }

        self.counter.add(
            1,
            &[
                KeyValue::new("domain", self.domain.clone()),
                KeyValue::new("result", result_label),
            ],
        );
        self.duration_hist
            .record(elapsed_ms, &[KeyValue::new("domain", self.domain.clone())]);
    }

    /// Finalize: register ObservableGauge for pass rate, return stats.
    pub fn finalize(&self) -> DomainStats {
        let stats = self.stats.stats.clone();
        let domain = self.domain.clone();
        let stats_for_cb = stats.clone();

        // Register ObservableGauge
        let _gauge = self
            .meter
            .f64_observable_gauge("spec_pass_rate")
            .with_description("Spec test pass rate by domain")
            .with_callback(move |observer| {
                observer.observe(
                    stats_for_cb.pass_rate(),
                    &[KeyValue::new("domain", domain.clone())],
                );
            })
            .build();

        tracing::info!(
            stage = "spec_test",
            domain = %self.domain,
            total = stats.total,
            passed = stats.passed,
            failed = stats.failed,
            skipped = stats.skipped,
            pass_rate = format!("{:.3}", stats.pass_rate()),
            "domain complete"
        );

        stats
    }

    /// Get accumulated stats (without registering gauge).
    pub fn stats(&self) -> &DomainAccumulator {
        &self.stats
    }

    /// Assert all tests passed. Call after shutdown_otel/metrics.
    pub fn assert_results(&self, label: &str) {
        if self.stats.stats.failed > 0 {
            panic!(
                "{} / {} tests failed in domain '{}'. \
                 See otel-trace-{}.jsonl and otel-metrics-{}.jsonl for details. \
                 First failure: {}",
                self.stats.stats.failed,
                self.stats.stats.total,
                self.domain,
                label,
                label,
                self.stats
                    .failures
                    .first()
                    .map(|(n, m)| format!("{}: {}", n, m))
                    .unwrap_or_default()
            );
        }
    }
}
