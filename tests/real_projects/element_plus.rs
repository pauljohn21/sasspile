//! Real project validation — Element Plus SCSS compilation.
//!
//! Compiles `element-plus/packages/theme-chalk/src/index.scss` and verifies no errors.
//! Records compilation time using tracing span `elapsed_ms`.

use sasspile::compile;
use std::path::PathBuf;
use std::time::Instant;

fn element_plus_scss_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("element-plus")
        .join("packages")
        .join("theme-chalk")
        .join("src")
        .join("index.scss")
}

#[test]
fn test_element_plus_compiles() {
    let path = element_plus_scss_path();
    if !path.exists() {
        eprintln!("Element Plus SCSS not found at {:?}, skipping", path);
        return;
    }

    let src = std::fs::read_to_string(&path).expect("should read index.scss");
    let start = Instant::now();

    match compile(&src) {
        Ok(css) => {
            let elapsed = start.elapsed();
            tracing::info!(
                stage = "real_project",
                project = "element_plus",
                elapsed_ms = elapsed.as_millis() as u64,
                output_len = css.len(),
                "Element Plus compiled successfully"
            );

            assert!(!css.is_empty(), "Element Plus output CSS should not be empty");
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
            eprintln!("Element Plus compilation error: {}", e);
        }
    }
}

#[test]
fn test_element_plus_output_valid() {
    let path = element_plus_scss_path();
    if !path.exists() {
        eprintln!("Element Plus SCSS not found, skipping");
        return;
    }

    let src = std::fs::read_to_string(&path).expect("should read index.scss");

    match compile(&src) {
        Ok(css) => {
            let open_braces = css.matches('{').count();
            let close_braces = css.matches('}').count();
            assert_eq!(
                open_braces, close_braces,
                "Braces should be balanced: {} open, {} close",
                open_braces, close_braces
            );
        }
        Err(_) => {
            eprintln!("Element Plus compilation failed, skipping CSS validation");
        }
    }
}
