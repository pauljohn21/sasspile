//! Bootstrap SCSS compilation with real OpenTelemetry SDK.
//!
//! See `otel_test_harness` for the shared test logic.
//!
//! Usage:
//! ```bash
//! RUST_LOG=info cargo test --test bootstrap_otel -- --nocapture
//! ```

use std::path::PathBuf;

mod otel_test_harness;
use otel_test_harness::ProjectSpec;

fn spec() -> ProjectSpec {
    ProjectSpec {
        label: "bootstrap",
        project: "bootstrap",
        compile_span: "bootstrap_compile",
        validate_span: "bootstrap_validate",
        scss_path: || {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("bootstrap")
                .join("scss")
                .join("bootstrap.scss")
        },
    }
}

#[test]
fn test_bootstrap_otel_trace() {
    otel_test_harness::run_otel_trace_test(&spec());
}

#[test]
fn test_bootstrap_output_valid() {
    otel_test_harness::run_output_valid_test(&spec());
}
