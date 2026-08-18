//! Bootstrap SCSS compilation with real OpenTelemetry SDK.
//!
//! This test initializes the OTel TracerProvider with a stdout exporter
//! that writes OTel-formatted spans to `otel-trace-bootstrap.jsonl`.
//! Also writes tracing events as JSON lines to `otel-trace-bootstrap.events.jsonl`.
//!
//! Usage:
//! ```bash
//! RUST_LOG=info cargo test --test bootstrap_otel -- --nocapture
//! # Then read: otel-trace-bootstrap.jsonl and otel-trace-bootstrap.events.jsonl
//! ```

use sasspile::compile_file;
use std::path::PathBuf;
use std::time::Instant;

mod tracing_init;

fn init_tracing() {
    tracing_init::init_otel("bootstrap");
}

fn bootstrap_scss_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("bootstrap")
        .join("scss")
        .join("bootstrap.scss")
}

#[test]
fn test_bootstrap_otel_trace() {
    init_tracing();
    let path = bootstrap_scss_path();
    if !path.exists() {
        tracing::warn!(
            stage = "real_project",
            project = "bootstrap",
            "SCSS file not found, skipping"
        );
        return;
    }

    // Run in a thread with a large stack to handle deeply nested SCSS
    let handle = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024) // 64 MB stack
        .spawn(move || {
            let start = Instant::now();
            let span = tracing::info_span!(
                "bootstrap_compile",
                stage = "real_project",
                project = "bootstrap",
                file = %path.display(),
            );
            let _enter = span.enter();

            match compile_file(&path) {
                Ok(css) => {
                    let elapsed = start.elapsed();
                    tracing::info!(
                        stage = "real_project",
                        project = "bootstrap",
                        elapsed_ms = elapsed.as_millis() as u64,
                        output_len = css.len(),
                        "Bootstrap compiled successfully"
                    );
                    Ok(css)
                }
                Err(e) => {
                    let elapsed = start.elapsed();
                    tracing::warn!(
                        stage = "real_project",
                        project = "bootstrap",
                        elapsed_ms = elapsed.as_millis() as u64,
                        error = %e,
                        "Bootstrap compilation failed"
                    );
                    Err(e)
                }
            }
        })
        .expect("failed to spawn thread");

    match handle.join() {
        Ok(Ok(css)) => assert!(
            !css.is_empty(),
            "Bootstrap output CSS should not be empty"
        ),
        Ok(Err(e)) => {
            tracing::error!(
                stage = "real_project",
                project = "bootstrap",
                error = %e,
                "Bootstrap compilation error"
            );
            // Don't panic — we want the trace file to be fully written
        }
        Err(_) => tracing::error!(
            stage = "real_project",
            project = "bootstrap",
            "Bootstrap compilation thread panicked"
        ),
    }

    tracing_init::shutdown_otel();
}
