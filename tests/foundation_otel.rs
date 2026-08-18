//! Foundation SCSS compilation with real OpenTelemetry SDK.
//!
//! See `otel_test_harness` for the shared test logic.
//!
//! Usage:
//! ```bash
//! RUST_LOG=info cargo test --test foundation_otel -- --nocapture
//! ```

use std::path::PathBuf;

mod otel_test_harness;
use otel_test_harness::ProjectSpec;

fn spec() -> ProjectSpec {
    ProjectSpec {
        label: "foundation",
        project: "foundation",
        compile_span: "foundation_compile",
        validate_span: "foundation_validate",
        scss_path: || {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("foundation")
                .join("scss")
                .join("foundation.scss")
        },
    }
}

#[test]
fn test_foundation_otel_trace() {
    otel_test_harness::run_otel_trace_test(&spec());
}

#[test]
fn test_foundation_output_valid() {
    otel_test_harness::run_output_valid_test(&spec());
}
