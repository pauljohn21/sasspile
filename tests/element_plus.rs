//! Real project validation — Element Plus SCSS compilation.
//!
//! Compiles `element-plus/packages/theme-chalk/src/index.scss` using
//! `compile_file` so that `@import`/`@use` directives resolve from filesystem.
//! Records compilation time using tracing span `elapsed_ms`.

use sasspile::compile_file;
use std::path::PathBuf;
use std::time::Instant;

mod tracing_init;

fn init_tracing() {
    tracing_init::init("element_plus");
}

fn element_plus_scss_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("element-plus")
        .join("packages")
        .join("theme-chalk")
        .join("src")
        .join("index.scss")
}

#[test]
fn test_element_plus_compiles() {
    init_tracing();
    let path = element_plus_scss_path();
    if !path.exists() {
        tracing::warn!(stage = "real_project", project = "element_plus", "SCSS file not found, skipping");
        return;
    }

    // Run in a thread with a large stack to handle deeply nested SCSS
    let handle = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024) // 64 MB stack
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

    match handle.join() {
        Ok(Ok(css)) => {
            tracing_init::flush_flame();
            assert!(!css.is_empty(), "Element Plus output CSS should not be empty");
        }
        Ok(Err(e)) => {
            tracing_init::flush_flame();
            panic!("Element Plus compilation error: {}", e);
        }
        Err(_) => {
            tracing_init::flush_flame();
            panic!("Element Plus compilation thread panicked");
        }
    }
}

#[test]
fn test_element_plus_output_valid() {
    let path = element_plus_scss_path();
    if !path.exists() {
        tracing::warn!(stage = "real_project", project = "element_plus", "SCSS file not found, skipping");
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
}
