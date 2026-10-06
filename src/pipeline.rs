use std::path::Path;
use std::sync::Arc;
use rxrust::prelude::*;
use crate::eval::{eval_stream, eval_ast_stream_sync};
use crate::lexer::scan;
use crate::parser::parse_stream;
use crate::runtime::create_runtime;
use crate::serialize::{self, Options};
use crate::types::*;

pub fn from_string(source: &str, options: &Options) -> Result<String, CompileError> {
    let tokens = scan(source);
    let (ctx, _bus) = create_runtime();
    let ast = parse_stream(tokens, ctx.scope_id());

    // Use synchronous evaluation for now to avoid async issue
    let stmts = eval_ast_stream_sync(ast, ctx);

    Ok(serialize::serialize(&stmts, options))
}

pub fn from_path(path: &Path, options: &Options) -> Result<String, CompileError> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| CompileError::Io(format!("Failed to read {:?}: {}", path, e)))?;
    from_string(&source, options)
}

pub fn collect_stream(stream: OutputStream) -> String {
    let result = Arc::new(std::sync::Mutex::new(String::new()));
    let r = result.clone();
    stream.subscribe(move |chunk| {
        r.lock().unwrap().push_str(&chunk);
    });
    let guard = result.lock().unwrap();
    guard.clone()
}
