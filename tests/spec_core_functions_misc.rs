//! Spec tests for `sass-spec/spec/core_functions/` misc domain. Uses OTel Metrics + Trace.
#[path = "hrx_parser.rs"] mod hrx_parser;
#[path = "hrx_vfs.rs"] mod hrx_vfs;
#[path = "spec_runner.rs"] mod spec_runner;
#[path = "spec_otel_runner.rs"] mod spec_otel_runner;
mod tracing_init;
use std::path::PathBuf;
fn spec_root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("sass-spec").join("spec") }
#[test] fn test_core_functions_misc_otel() {
    let label = "spec_core_functions_misc"; tracing_init::init_otel(label); tracing_init::init_metrics(label);
    let dir = spec_root().join("core_functions");
    if !dir.exists() { tracing::warn!(stage="spec_test", domain="core_functions_misc", "dir not found, skipping"); tracing_init::shutdown_metrics(); tracing_init::shutdown_otel(); return; }
    let mut runner = spec_otel_runner::SpecOtelRunner::new("core_functions_misc");
    let hrx_files = hrx_parser::find_hrx_files(&dir);
    for hrx_path in &hrx_files {
        // Skip subdirectories already covered by dedicated test files
        let path_str = hrx_path.to_string_lossy();
        if path_str.contains("/color/") || path_str.contains("/list/") || path_str.contains("/map/")
            || path_str.contains("/math/") || path_str.contains("/meta/")
            || path_str.contains("/string/") || path_str.contains("/selector/") {
            continue;
        }
        runner.run_hrx_tests(hrx_path);
    }
    let stats = runner.finalize();
    tracing_init::shutdown_metrics(); tracing_init::shutdown_otel();
    runner.assert_results(label);
}
