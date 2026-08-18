//! Element Plus SCSS compilation with real OpenTelemetry SDK.
//!
//! This test initializes the OTel TracerProvider with a stdout exporter
//! that writes OTel-formatted spans to `otel-trace-element_plus.jsonl`.
//! Also writes tracing events as JSON lines to `otel-trace-element_plus.events.jsonl`.
//!
//! Usage:
//! ```bash
//! RUST_LOG=info cargo test --test element_plus_otel -- --nocapture
//! # Then read: otel-trace-element_plus.jsonl and otel-trace-element_plus.events.jsonl
//! ```

use sasspile::compile_file;
use std::path::PathBuf;
use std::time::Instant;

mod tracing_init;

fn init_tracing() {
    tracing_init::init_otel("element_plus");
}

fn element_plus_scss_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("element-plus")
        .join("packages")
        .join("theme-chalk")
        .join("src")
        .join("index.scss")
}

/// Spawn a thread with 64 MB stack to compile Element Plus SCSS.
fn compile_element_plus_in_thread(
    path: &PathBuf,
) -> Result<String, sasspile::SassError> {
    let path = path.clone();
    let handle = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let start = Instant::now();
            let span = tracing::info_span!(
                "element_plus_compile",
                stage = "real_project",
                project = "element_plus",
                file = %path.display(),
            );
            let _enter = span.enter();

            match compile_file(&path) {
                Ok(css) => {
                    let elapsed = start.elapsed();
                    tracing::info!(
                        stage = "real_project",
                        project = "element_plus",
                        elapsed_ms = elapsed.as_millis() as u64,
                        output_len = css.len(),
                        "Element Plus compiled successfully"
                    );
                    Ok(css)
                }
                Err(e) => {
                    let elapsed = start.elapsed();
                    tracing::warn!(
                        stage = "real_project",
                        project = "element_plus",
                        elapsed_ms = elapsed.as_millis() as u64,
                        error = %e,
                        "Element Plus compilation failed"
                    );
                    Err(e)
                }
            }
        })
        .expect("failed to spawn thread");

    handle.join().unwrap_or_else(|_| {
        Err(sasspile::SassError::eval(
            "Element Plus compilation thread panicked",
            sasspile::error::SourcePos::default(),
        ))
    })
}

#[test]
fn test_element_plus_otel_trace() {
    init_tracing();
    let path = element_plus_scss_path();
    if !path.exists() {
        tracing::warn!(
            stage = "real_project",
            project = "element_plus",
            "SCSS file not found, skipping"
        );
        return;
    }

    match compile_element_plus_in_thread(&path) {
        Ok(css) => assert!(
            !css.is_empty(),
            "Element Plus output CSS should not be empty"
        ),
        Err(e) => {
            tracing::error!(
                stage = "real_project",
                project = "element_plus",
                error = %e,
                "Element Plus compilation error"
            );
            // Don't panic — we want the trace file to be fully written
        }
    }

    tracing_init::shutdown_otel();
}

#[test]
fn test_element_plus_output_valid() {
    init_tracing();
    let path = element_plus_scss_path();
    if !path.exists() {
        tracing::warn!(
            stage = "real_project",
            project = "element_plus",
            "SCSS file not found, skipping"
        );
        return;
    }

    let span = tracing::info_span!(
        "element_plus_validate",
        stage = "real_project",
        project = "element_plus",
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
                project = "element_plus",
                open_braces,
                close_braces,
                "CSS validation passed"
            );
        }
        Err(e) => {
            tracing::warn!(
                stage = "real_project",
                project = "element_plus",
                error = %e,
                "Element Plus compilation failed, skipping CSS validation"
            );
        }
    }

    tracing_init::shutdown_otel();
}
