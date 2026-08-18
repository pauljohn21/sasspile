//! Shared macro for generating spec domain test boilerplate.
//!
//! Eliminates duplication across 17 `spec_*.rs` test files.
//! Each test file now only needs one line: `spec_domain_test!(label, "path", "domain");`
//!
//! Usage:
//! ```rust,ignore
//! #[path = "spec_macro.rs"]
//! mod spec_macro;
//! spec_macro::spec_domain_test!(test_operators_otel, "operators", "operators");
//! ```

/// Generate a complete `#[test]` function for a spec domain.
///
/// Parameters:
/// - `$test_fn`: The test function name (e.g. `test_operators_otel`)
/// - `$spec_path`: The path under `sass-spec/spec/` (e.g. `"operators"`, `"css/plain"`)
/// - `$domain`: The domain label for OTel metrics (e.g. `"operators"`, `"css_plain"`)
macro_rules! spec_domain_test {
    ($test_fn:ident, $spec_path:expr, $domain:expr) => {
        #[path = "hrx_parser.rs"]
        mod hrx_parser;

        #[path = "hrx_vfs.rs"]
        mod hrx_vfs;

        #[path = "spec_runner.rs"]
        mod spec_runner;

        #[path = "spec_otel_runner.rs"]
        mod spec_otel_runner;

        mod tracing_init;

        use std::path::PathBuf;

        fn spec_root() -> PathBuf {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("sass-spec")
                .join("spec")
        }

        #[test]
        fn $test_fn() {
            let label = stringify!($test_fn);
            let domain_label: &str = $domain;
            tracing_init::init_otel(label);
            tracing_init::init_metrics(label);

            let dir = spec_root().join($spec_path);
            if !dir.exists() {
                tracing::warn!(
                    stage = "spec_test",
                    domain = %domain_label,
                    path = %$spec_path,
                    "directory not found, skipping"
                );
                tracing_init::shutdown_metrics();
                tracing_init::shutdown_otel();
                return;
            }

            let span = tracing::info_span!(
                "spec_domain",
                stage = "spec_test",
                domain = %domain_label,
                path = %$spec_path,
            );
            let _enter = span.enter();

            let mut runner = spec_otel_runner::SpecOtelRunner::new(domain_label);
            let hrx_files = hrx_parser::find_hrx_files(&dir);

            tracing::info!(
                stage = "spec_test",
                domain = %domain_label,
                hrx_count = hrx_files.len(),
                "starting domain"
            );

            for hrx_path in &hrx_files {
                runner.run_hrx_tests(hrx_path);
            }

            let stats = runner.finalize();

            tracing::info!(
                stage = "spec_test",
                domain = %domain_label,
                total = stats.total,
                passed = stats.passed,
                failed = stats.failed,
                pass_rate = format!("{:.3}", stats.pass_rate()),
                "domain complete"
            );

            tracing_init::shutdown_metrics();
            tracing_init::shutdown_otel();

            runner.assert_results(label);
        }
    };
}
