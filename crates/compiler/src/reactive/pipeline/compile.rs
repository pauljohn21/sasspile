//! Core compilation entry: AstNode stream → CssStmt stream → String.

use std::path::Path;
use std::sync::Arc;

use rxrust::prelude::*;
use tracing::{debug_span, info_span};

use crate::reactive::{
    evaluate_to_css, pre_analysis, serialize_to_string, AstNode, AstStream, CompilerBus,
    EvalContext, Options, OutputStyle, ScopeId,
};

/// Collect a `CssStream` synchronously into a `Vec<CssStmt>`.
///
/// Uses rxrust's `collect` operator to aggregate all emitted items into
/// a single `Vec`, then returns it.
pub(crate) fn collect_css(stream: crate::reactive::CssStream) -> Vec<crate::reactive::CssStmt> {
    let span = debug_span!("collect_css");
    let _guard = span.enter();

    let collected = stream.collect::<Vec<_>>();
    let result: Arc<std::sync::Mutex<Option<Vec<crate::reactive::CssStmt>>>> =
        Arc::new(std::sync::Mutex::new(None));
    let r = result.clone();
    collected.subscribe(move |stmts| {
        *r.lock().unwrap() = Some(stmts);
    });

    match Arc::try_unwrap(result) {
        Ok(mutex) => mutex.into_inner().unwrap().unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

/// Compile a `Vec<AstNode>` into a `Vec<CssStmt>`.
pub fn compile_ast(items: Vec<AstNode>, scope_id: ScopeId) -> Vec<crate::reactive::CssStmt> {
    let span = info_span!("compile_ast", scope_id);
    let _guard = span.enter();

    let bus = CompilerBus::new();
    let ctx = Arc::new(EvalContext::new(bus, scope_id));

    let mut counter = 0;
    let analyzed = pre_analysis(items, scope_id, &mut counter);

    let stream: AstStream = Shared::from_iter(analyzed).box_it();
    let css_stream = evaluate_to_css(stream, ctx);

    collect_css(css_stream)
}

/// Compile a `Vec<AstNode>` into formatted CSS string.
pub fn compile_ast_to_string(items: Vec<AstNode>, scope_id: ScopeId, options: &Options) -> String {
    let span = info_span!("compile_ast_to_string", scope_id);
    let _guard = span.enter();

    let bus = CompilerBus::new();
    let ctx = Arc::new(EvalContext::new(bus, scope_id));

    let mut counter = 0;
    let analyzed = pre_analysis(items, scope_id, &mut counter);

    let stream: AstStream = Shared::from_iter(analyzed).box_it();
    let css_stream = evaluate_to_css(stream, ctx);

    serialize_to_string(css_stream, options)
}

/// Compile a pre-built AST into CSS (convenience for tests).
pub fn from_string_ast(items: Vec<AstNode>) -> Result<String, crate::Error> {
    let css_stmts = compile_ast(items, 0);
    // Use the existing serialize_to_string on a reconstructed stream
    let stream = Shared::from_iter(css_stmts).box_it();
    Ok(serialize_to_string(stream, &Options::default()))
}

/// Compile from a source string with the given options.
///
/// This is the main public API for the compiler. It requires parser
/// integration to be fully functional; currently it returns an error
/// until the parser pipeline is wired in.
pub fn from_string(source: &str, options: &Options) -> Result<String, crate::Error> {
    let _span = info_span!("from_string", style = ?options.style);

    // TODO: integrate parser pipeline here
    // let token_stream = crate::lexer::lex(source);
    // let ast_stream = crate::parser::parse(token_stream);
    // For now, return a stub error
    let _ = source;

    if source.trim().is_empty() {
        return Ok(String::new());
    }

    Err(crate::Error::msg(
        "from_string: parser integration pending — use from_string_ast or compile_ast_to_string",
    ))
}

/// Trait for filesystem access (enables mocking in tests).
pub trait Fs {
    /// Read a file at the given path to a string.
    fn read_to_string(&self, path: &Path) -> std::io::Result<String>;
}

/// Default filesystem implementation.
#[derive(Default)]
pub struct RealFs;

impl Fs for RealFs {
    fn read_to_string(&self, path: &Path) -> std::io::Result<String> {
        std::fs::read_to_string(path)
    }
}

/// Compile from a file path with the given options.
///
/// Uses the `Fs` trait for filesystem access so tests can inject mocks.
pub fn from_path<F: Fs>(path: &Path, options: &Options, fs: &F) -> Result<String, crate::Error> {
    let _span = info_span!("from_path", display_path = %path.display());

    let source = fs
        .read_to_string(path)
        .map_err(|e| crate::Error::io(format!("failed to read {}: {}", path.display(), e)))?;

    if source.trim().is_empty() {
        return Ok(String::new());
    }

    from_string(&source, options)
}
