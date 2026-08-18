//! Spec tests for `sass-spec/spec/core_functions/list/` domain. Uses OTel Metrics + Trace.
#[path = "hrx_parser.rs"] mod hrx_parser;
#[path = "hrx_vfs.rs"] mod hrx_vfs;
#[path = "spec_runner.rs"] mod spec_runner;
#[path = "spec_otel_runner.rs"] mod spec_otel_runner;
mod tracing_init;
use std::path::PathBuf;
fn spec_root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("sass-spec").join("spec") }
#[test] fn test_core_functions_list_otel() {
    let label = "spec_core_functions_list"; tracing_init::init_otel(label); tracing_init::init_metrics(label);
    let dir = spec_root().join("core_functions/list");
    if !dir.exists() { tracing::warn!(stage="spec_test", domain="core_functions_list", "dir not found, skipping"); tracing_init::shutdown_metrics(); tracing_init::shutdown_otel(); return; }
    let mut runner = spec_otel_runner::SpecOtelRunner::new("core_functions_list");
    let hrx_files = hrx_parser::find_hrx_files(&dir);
    for hrx_path in &hrx_files { runner.run_hrx_tests(hrx_path); }
    let stats = runner.finalize();
    tracing_init::shutdown_metrics(); tracing_init::shutdown_otel();
    runner.assert_results(label);
}
