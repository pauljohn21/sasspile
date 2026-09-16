//! sasspile-rx: rxrust-first SCSS compiler.
//!
//! # Usage
//!
//! ```rust
//! use sasspile_rx::compile;
//!
//! fn main() -> Result<(), sasspile_rx::CompileError> {
//!     let css = compile("a { color: red; }")?;
//!     println!("{css}");
//!     Ok(())
//! }
//! ```

pub mod ast;
mod error;
mod evaluate_dst;
pub mod parse_dst;
mod pipeline;
mod serialize_dst;
mod shared;
mod tokenize_dst;

pub use error::CompileError;

use std::sync::Once;
use tracing_subscriber::FmtSubscriber;

static TRACING_INIT: Once = Once::new();

fn ensure_tracing() {
    TRACING_INIT.call_once(|| {
        let filter = std::env::var("RUST_LOG")
            .ok()
            .and_then(|s| s.parse::<tracing::Level>().ok())
            .unwrap_or(tracing::Level::WARN);
        let _ = FmtSubscriber::builder()
            .with_max_level(filter)
            .try_init();
    });
}

use std::path::Path;
use std::rc::Rc;
use std::cell::RefCell;

use rxrust::prelude::*;

/// Compile SCSS source to CSS string — the single public API entry.
///
/// Wraps the internal rxrust 4-stage pipeline
/// (tokenize → parse → evaluate → serialize)
/// into a clean `Result<String, CompileError>`.
///
/// # Example
/// ```
/// use sasspile_rx::compile;
///
/// let css = compile("a { color: red; }").unwrap();
/// assert!(css.contains("color"));
/// assert!(css.contains("red"));
/// ```
pub fn compile(input: &str) -> Result<String, CompileError> {
    let _root = tracing::info_span!("sasspile.compile", input_len = input.len()).entered();

    ensure_tracing();

    let result = Rc::new(RefCell::new(Vec::<char>::new()));
    let r = result.clone();

    pipeline::build(input).subscribe(move |ch| {
        r.borrow_mut().push(ch);
    });

    let css: String = result.borrow().iter().collect();

    tracing::info!(
        parent: tracing::Span::current(),
        output_len = css.len(),
        "compile complete"
    );

    if css.contains("COMPILE ERROR:") {
        return Err(CompileError::InvalidInput {
            message: "compilation error — see output".into(),
        });
    }
    Ok(css)
}

/// Compile a SCSS file at `path`, resolving its @import/@use directives
/// against the file's parent directory.
pub fn compile_file(path: &Path) -> Result<String, CompileError> {
    let src = std::fs::read_to_string(path).map_err(|e| CompileError::ModuleLoadFailure {
        path: path.display().to_string(),
        reason: e.to_string(),
    })?;
    let base = path.parent().unwrap_or_else(|| Path::new("."));
    compile_at(&src, base)
}

/// Compile with a base path — used internally by @import expansion.
pub fn compile_at(input: &str, base_path: &Path) -> Result<String, CompileError> {
    let _root = tracing::info_span!("sasspile.compile_at", base = %base_path.display(), input_len = input.len()).entered();

    let result = Rc::new(RefCell::new(Vec::<char>::new()));
    let r = result.clone();

    pipeline::build_with_base(input, Some(base_path)).subscribe(move |ch| {
        r.borrow_mut().push(ch);
    });

    let css: String = result.borrow().iter().collect();
    if css.contains("COMPILE ERROR:") {
        return Err(CompileError::InvalidInput {
            message: "compilation error — see output".into(),
        });
    }
    Ok(css)
}
