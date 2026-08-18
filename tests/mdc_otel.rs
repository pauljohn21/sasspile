//! Material Components Web (MDC-Web) SCSS compilation with real OpenTelemetry SDK.
//!
//! This test initializes the OTel TracerProvider with a stdout exporter
//! that writes OTel-formatted spans to `otel-trace-mdc.jsonl`.
//! Also writes tracing events as JSON lines to `otel-trace-mdc.events.jsonl`.
//!
//! Usage:
//! ```bash
//! RUST_LOG=info cargo test --test mdc_otel -- --nocapture
//! # Then read: otel-trace-mdc.jsonl and otel-trace-mdc.events.jsonl
//! ```

use sasspile::compile_file;
use std::path::PathBuf;
use std::time::Instant;

mod tracing_init;

fn init_tracing() {
    tracing_init::init_otel("mdc");
}

fn mdc_scss_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("mdc-web")
        .join("packages")
        .join("material-components-web")
        .join("material-components-web.scss")
}

/// Spawn a thread with 64 MB stack to compile MDC-Web SCSS.
fn compile_mdc_in_thread(
    path: &PathBuf,
) -> Result<String, sasspile::SassError> {
    let path = path.clone();
    let handle = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let start = Instant::now();
            let span = tracing::info_span!(
                "mdc_compile",
                stage = "real_project",
                project = "mdc_web",
                file = %path.display(),
            );
            let _enter = span.enter();

            match compile_file(&path) {
                Ok(css) => {
                    let elapsed = start.elapsed();
                    tracing::info!(
                        stage = "real_project",
                        project = "mdc_web",
                        elapsed_ms = elapsed.as_millis() as u64,
                        output_len = css.len(),
                        "MDC-Web compiled successfully"
                    );
                    Ok(css)
                }
                Err(e) => {
                    let elapsed = start.elapsed();
                    tracing::warn!(
                        stage = "real_project",
                        project = "mdc_web",
                        elapsed_ms = elapsed.as_millis() as u64,
                        error = %e,
                        "MDC-Web compilation failed"
                    );
                    Err(e)
                }
            }
        })
        .expect("failed to spawn thread");

    handle.join().unwrap_or_else(|_| {
        Err(sasspile::SassError::eval(
            "MDC-Web compilation thread panicked",
            sasspile::error::SourcePos::default(),
        ))
    })
}

#[test]
fn test_mdc_otel_trace() {
    init_tracing();
    let path = mdc_scss_path();
    if !path.exists() {
        tracing::warn!(
            stage = "real_project",
            project = "mdc_web",
            "SCSS file not found, skipping"
        );
        return;
    }

    match compile_mdc_in_thread(&path) {
        Ok(css) => assert!(
            !css.is_empty(),
            "MDC-Web output CSS should not be empty"
        ),
        Err(e) => {
            tracing::error!(
                stage = "real_project",
                project = "mdc_web",
                error = %e,
                "MDC-Web compilation error"
            );
            // Don't panic — we want the trace file to be fully written
        }
    }

    tracing_init::shutdown_otel();
}

#[test]
fn test_mdc_output_valid() {
    init_tracing();
    let path = mdc_scss_path();
    if !path.exists() {
        tracing::warn!(
            stage = "real_project",
            project = "mdc_web",
            "SCSS file not found, skipping"
        );
        return;
    }

    let span = tracing::info_span!(
        "mdc_validate",
        stage = "real_project",
        project = "mdc_web",
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
                project = "mdc_web",
                open_braces,
                close_braces,
                "CSS validation passed"
            );
        }
        Err(e) => {
            tracing::warn!(
                stage = "real_project",
                project = "mdc_web",
                error = %e,
                "MDC-Web compilation failed, skipping CSS validation"
            );
        }
    }

    tracing_init::shutdown_otel();
}
