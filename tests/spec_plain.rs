//! Spec tests for `sass-spec/spec/css/plain/` domain.
//!
//! Uses OTel Metrics + Trace for quantified pass-rate tracking.

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
fn test_plain_otel() {
    let label = "spec_plain";
    tracing_init::init_otel(label);
    tracing_init::init_metrics(label);

    let dir = spec_root().join("css/plain");
    if !dir.exists() {
        tracing::warn!(stage = "spec_test", domain = "css_plain", "css/plain directory not found, skipping");
        tracing_init::shutdown_metrics();
        tracing_init::shutdown_otel();
        return;
    }

    let hrx_files = hrx_parser::find_hrx_files(&dir);
    let mut runner = spec_otel_runner::SpecOtelRunner::new("css_plain");

    for hrx_path in &hrx_files {
        runner.run_hrx_tests(hrx_path);
    }

    let stats = runner.finalize();

    tracing_init::shutdown_metrics();
    tracing_init::shutdown_otel();

    runner.assert_results(label);
}
