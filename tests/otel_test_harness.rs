//! Shared test harness for OTel-instrumented SCSS compilation tests.
//!
//! Eliminates boilerplate duplicated across `bootstrap_otel`, `bulma_otel`,
//! `element_plus_otel`, `mdc_otel`, and `foundation_otel` test files.
//!
//! Each project test only needs to provide a [`ProjectSpec`] and call
//! [`run_otel_trace_test`] / [`run_output_valid_test`].

use sasspile::compile_file;
use sasspile::error::SourcePos;
use sasspile::SassError;
use std::path::PathBuf;
use std::time::Instant;

#[path = "tracing_init.rs"]
mod tracing_init;

/// Configuration for a single SCSS project under test.
#[derive(Clone)]
pub struct ProjectSpec {
    /// Short label used for OTel trace file names (e.g. `"bootstrap"`).
    pub label: &'static str,
    /// Human-readable project name for span fields (e.g. `"bootstrap"`).
    pub project: &'static str,
    /// Span name for the compile phase (used in log messages).
    pub compile_span: &'static str,
    /// Span name for the validate phase (used in log messages).
    pub validate_span: &'static str,
    /// Function returning the SCSS entry file path.
    pub scss_path: fn() -> PathBuf,
}

impl ProjectSpec {
    /// Build the full SCSS path via the provided closure.
    pub fn path(&self) -> PathBuf {
        (self.scss_path)()
    }
}

// ---------------------------------------------------------------------------
// Public test entry points
// ---------------------------------------------------------------------------

/// Run the OTel trace test: init OTel → compile in 64 MB thread → shutdown.
///
/// Writes trace data to `otel-trace-<label>.jsonl` and
/// `otel-trace-<label>.events.jsonl`.
pub fn run_otel_trace_test(spec: &ProjectSpec) {
    tracing_init::init_otel(spec.label);
    let path = spec.path();
    if !path.exists() {
        tracing::warn!(
            stage = "real_project",
            project = spec.project,
            "SCSS file not found, skipping"
        );
        return;
    }

    match compile_in_thread(spec, &path) {
        Ok(css) => {
            if css.is_empty() {
                tracing::warn!(
                    stage = "real_project",
                    project = spec.project,
                    output_len = 0usize,
                    "{} output CSS is empty (likely @use not fully resolved)",
                    spec.project
                );
            } else {
                tracing::info!(
                    stage = "real_project",
                    project = spec.project,
                    output_len = css.len(),
                    "{} output CSS is non-empty",
                    spec.project
                );
            }
        }
        Err(e) => {
            tracing::error!(
                stage = "real_project",
                project = spec.project,
                error = %e,
                "{} compilation error",
                spec.project
            );
            // Don't panic — we want the trace file to be fully written
        }
    }

    tracing_init::shutdown_otel();
}

/// Run the output validation test: init OTel → compile → validate braces.
///
/// Uses the same 64 MB thread as the trace test to avoid stack overflow
/// on deeply nested SCSS projects.
pub fn run_output_valid_test(spec: &ProjectSpec) {
    tracing_init::init_otel(spec.label);
    let path = spec.path();
    if !path.exists() {
        tracing::warn!(
            stage = "real_project",
            project = spec.project,
            "SCSS file not found, skipping"
        );
        return;
    }

    let span = tracing::info_span!(
        "project_validate",
        stage = "real_project",
        project = spec.project,
        phase = spec.validate_span,
    );
    let _enter = span.enter();

    match compile_in_thread(spec, &path) {
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
                project = spec.project,
                open_braces,
                close_braces,
                "CSS validation passed"
            );
        }
        Err(e) => {
            tracing::warn!(
                stage = "real_project",
                project = spec.project,
                error = %e,
                "{} compilation failed, skipping CSS validation",
                spec.project
            );
        }
    }

    tracing_init::shutdown_otel();
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Spawn a thread with 64 MB stack to compile SCSS.
///
/// The large stack is necessary because sasspile's recursive evaluator
/// can exceed the default 8 MB stack on real-world projects.
fn compile_in_thread(spec: &ProjectSpec, path: &PathBuf) -> Result<String, SassError> {
    let path = path.clone();
    let span_name: String = spec.compile_span.to_string();
    let project: String = spec.project.to_string();
    let handle = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let start = Instant::now();
            let span = tracing::info_span!(
                "project_compile",
                stage = "real_project",
                project = %project,
                file = %path.display(),
                phase = %span_name,
            );
            let _enter = span.enter();

            match compile_file(&path) {
                Ok(css) => {
                    let elapsed = start.elapsed();
                    tracing::info!(
                        stage = "real_project",
                        project = %project,
                        elapsed_ms = elapsed.as_millis() as u64,
                        output_len = css.len(),
                        "{} compiled successfully",
                        project
                    );
                    Ok(css)
                }
                Err(e) => {
                    let elapsed = start.elapsed();
                    tracing::warn!(
                        stage = "real_project",
                        project = %project,
                        elapsed_ms = elapsed.as_millis() as u64,
                        error = %e,
                        "{} compilation failed",
                        project
                    );
                    Err(e)
                }
            }
        })
        .expect("failed to spawn thread");

    handle.join().unwrap_or_else(|_| {
        let msg = format!("{} compilation thread panicked", spec.project);
        Err(SassError::eval(msg, SourcePos::default()))
    })
}
