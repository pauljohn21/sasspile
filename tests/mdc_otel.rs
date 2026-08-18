//! Material Components Web (MDC-Web) SCSS compilation with real OpenTelemetry SDK.
//!
//! See `otel_test_harness` for the shared test logic.
//!
//! Usage:
//! ```bash
//! RUST_LOG=info cargo test --test mdc_otel -- --nocapture
//! ```

use std::path::PathBuf;

mod otel_test_harness;
use otel_test_harness::ProjectSpec;

fn spec() -> ProjectSpec {
    ProjectSpec {
        label: "mdc",
        project: "mdc_web",
        compile_span: "mdc_compile",
        validate_span: "mdc_validate",
        scss_path: || {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("mdc-web")
                .join("packages")
                .join("material-components-web")
                .join("material-components-web.scss")
        },
    }
}

#[test]
fn test_mdc_otel_trace() {
    otel_test_harness::run_otel_trace_test(&spec());
}

#[test]
fn test_mdc_output_valid() {
    otel_test_harness::run_output_valid_test(&spec());
}
