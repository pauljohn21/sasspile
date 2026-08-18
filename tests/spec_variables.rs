//! Spec tests for `sass-spec/spec/variables/` domain. Uses OTel Metrics + Trace.
#[path = "hrx_parser.rs"] mod hrx_parser;
#[path = "hrx_vfs.rs"] mod hrx_vfs;
#[path = "spec_runner.rs"] mod spec_runner;
#[path = "spec_otel_runner.rs"] mod spec_otel_runner;
mod tracing_init;
use std::path::PathBuf;
fn spec_root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("sass-spec").join("spec") }
#[test] fn test_variables_otel() {
    let label = "spec_variables"; tracing_init::init_otel(label); tracing_init::init_metrics(label);
    let dir = spec_root().join("variables");
    if !dir.exists() { tracing::warn!(stage="spec_test", domain="variables", "dir not found, skipping"); tracing_init::shutdown_metrics(); tracing_init::shutdown_otel(); return; }
    let mut runner = spec_otel_runner::SpecOtelRunner::new("variables");
    let hrx_files = hrx_parser::find_hrx_files(&dir);
    for hrx_path in &hrx_files { runner.run_hrx_tests(hrx_path); }
    let stats = runner.finalize();
    tracing_init::shutdown_metrics(); tracing_init::shutdown_otel();
    runner.assert_results(label);
}
