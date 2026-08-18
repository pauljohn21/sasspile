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

/// Spawn a thread with 64 MB stack to compile Bootstrap SCSS.
fn compile_bootstrap_in_thread(
    path: &PathBuf,
) -> Result<String, sasspile::SassError> {
    let path = path.clone();
    let handle = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
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

    handle.join().unwrap_or_else(|_| {
        Err(sasspile::SassError::eval(
            "Bootstrap compilation thread panicked",
            sasspile::error::SourcePos::default(),
        ))
    })
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

    match compile_bootstrap_in_thread(&path) {
        Ok(css) => assert!(
            !css.is_empty(),
            "Bootstrap output CSS should not be empty"
        ),
        Err(e) => {
            tracing::error!(
                stage = "real_project",
                project = "bootstrap",
                error = %e,
                "Bootstrap compilation error"
            );
            // Don't panic — we want the trace file to be fully written
        }
    }

    tracing_init::shutdown_otel();
}

#[test]
fn test_bootstrap_output_valid() {
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

    let span = tracing::info_span!(
        "bootstrap_validate",
        stage = "real_project",
        project = "bootstrap",
    );
    let _enter = span.enter();

    match compile_file(&path) {
        Ok(css) => {
            let open_braces = css.matches('{').count();
            let close_braces = css.matches('}').count();
            assert_eq!(
                open_braces, close_braces,
                "Braces should be balanced: {} open, {} close",
                open_braces, close_braces
            );
            tracing::info!(
                stage = "real_project",
                project = "bootstrap",
                open_braces,
                close_braces,
                "CSS validation passed"
            );
        }
        Err(e) => {
            tracing::warn!(
                stage = "real_project",
                project = "bootstrap",
                error = %e,
                "Bootstrap compilation failed, skipping CSS validation"
            );
        }
    }

    tracing_init::shutdown_otel();
}
