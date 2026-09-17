//! sasspile-rx: rxrust-first SCSS compiler.
//!
//! endpoint 用 collect::<String>() 消费 char 流,无 Rc<RefCell>/subscribe+push.

pub mod ast;
mod error;
pub mod evaluate_dst;
pub mod parse_dst;
mod pipeline;
pub mod serialize_dst;
mod shared;
pub mod tokenize_dst;

pub use error::CompileError;

use std::path::Path;
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

use rxrust::prelude::*;

pub fn compile(input: &str) -> Result<String, CompileError> {
    let _root =
        tracing::info_span!("sasspile.compile", input_len = input.len()).entered();

    ensure_tracing();

    // collect::<String>() 消费 char 流 → String(FromIterator<char>)
    // subscribe 接收 owned String,无 Rc<RefCell>/push
    let (tx, rx) = std::sync::mpsc::channel();

    pipeline::build(input)
        .tap(|ch| tracing::trace!(char = %ch, stage = "serialize"))
        .collect::<String>()
        .last()
        .subscribe(move |css| {
            let _ = tx.send(css);
        });

    let css = rx.recv().unwrap_or_default();

    if css.contains("COMPILE ERROR:") {
        return Err(CompileError::InvalidInput {
            message: "compilation error — see output".into(),
        });
    }

    tracing::info!(output_len = css.len(), "compile complete");
    Ok(css)
}

pub fn compile_file(path: &Path) -> Result<String, CompileError> {
    let src = std::fs::read_to_string(path).map_err(|e| {
        CompileError::ModuleLoadFailure {
            path: path.display().to_string(),
            reason: e.to_string(),
        }
    })?;
    let base = path.parent().unwrap_or_else(|| Path::new("."));
    compile_at(&src, base)
}

pub fn compile_at(input: &str, base_path: &Path) -> Result<String, CompileError> {
    let _root = tracing::info_span!(
        "sasspile.compile_at",
        base = %base_path.display(),
        input_len = input.len()
    )
    .entered();

    let (tx, rx) = std::sync::mpsc::channel();

    pipeline::build_with_base(input, Some(base_path))
        .tap(|ch| tracing::trace!(char = %ch, stage = "serialize"))
        .collect::<String>()
        .last()
        .subscribe(move |css| {
            let _ = tx.send(css);
        });

    let css = rx.recv().unwrap_or_default();

    if css.contains("COMPILE ERROR:") {
        return Err(CompileError::InvalidInput {
            message: "compilation error — see output".into(),
        });
    }

    tracing::info!(output_len = css.len(), "compile complete");
    Ok(css)
}
