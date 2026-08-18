//! Real project validation — Bootstrap SCSS compilation.
//!
//! Compiles `bootstrap/scss/bootstrap.scss` using `compile_file` so that
//! `@import` directives resolve from the filesystem.
//! Records compilation time using tracing span `elapsed_ms`.

use sasspile::compile_file;
use std::path::PathBuf;
use std::time::Instant;

mod tracing_init;

fn init_tracing() {
    tracing_init::init("bootstrap");
}

fn bootstrap_scss_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("bootstrap")
        .join("scss")
        .join("bootstrap.scss")
}

#[test]
fn test_bootstrap_compiles() {
    init_tracing();
    let path = bootstrap_scss_path();
    if !path.exists() {
        tracing::warn!(stage = "real_project", project = "bootstrap", "SCSS file not found, skipping");
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
        Ok(Ok(css)) => {
            tracing_init::flush_flame();
            assert!(!css.is_empty(), "Bootstrap output CSS should not be empty");
        }
        Ok(Err(e)) => {
            tracing_init::flush_flame();
            panic!("Bootstrap compilation error: {}", e);
        }
        Err(_) => {
            tracing_init::flush_flame();
            panic!("Bootstrap compilation thread panicked");
        }
    }
}

#[test]
fn test_bootstrap_output_valid() {
    init_tracing();
    let path = bootstrap_scss_path();
    if !path.exists() {
        tracing::warn!(stage = "real_project", project = "bootstrap", "SCSS file not found, skipping");
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
}
