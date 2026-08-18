//! Element Plus SCSS compilation with real OpenTelemetry SDK.
//!
//! See `otel_test_harness` for the shared test logic.
//!
//! Usage:
//! ```bash
//! RUST_LOG=info cargo test --test element_plus_otel -- --nocapture
//! ```

use std::path::PathBuf;

mod otel_test_harness;
use otel_test_harness::ProjectSpec;

fn spec() -> ProjectSpec {
    ProjectSpec {
        label: "element_plus",
        project: "element_plus",
        compile_span: "element_plus_compile",
        validate_span: "element_plus_validate",
        scss_path: || {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("element-plus")
                .join("packages")
                .join("theme-chalk")
                .join("src")
                .join("index.scss")
        },
    }
}

#[test]
fn test_element_plus_otel_trace() {
    otel_test_harness::run_otel_trace_test(&spec());
}

#[test]
fn test_element_plus_output_valid() {
    otel_test_harness::run_output_valid_test(&spec());
}
