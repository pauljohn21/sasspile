use std::path::Path;
use std::sync::Arc;
use rxrust::prelude::*;
use crate::eval::eval_ast_stream_sync;
use crate::lexer::scan;
use crate::runtime::create_runtime;
use crate::serialize::{self, Options};
use crate::types::*;

pub fn from_string(source: &str, options: &Options) -> Result<String, CompileError> {
    from_string_with_paths(source, options, Vec::new())
}

pub fn from_string_with_paths(source: &str, options: &Options, include_paths: Vec<std::path::PathBuf>) -> Result<String, CompileError> {
    let tokens = scan(source);
    let (ctx, _bus) = create_runtime();
    let ast = crate::parser::parse_stream_with_paths(tokens, ctx.scope_id(), include_paths);

    // Use synchronous evaluation for now to avoid async issue
    let stmts = eval_ast_stream_sync(ast, ctx);

    Ok(serialize::serialize(&stmts, options))
}

pub fn from_path(path: &Path, options: &Options) -> Result<String, CompileError> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| CompileError::Io(format!("Failed to read {:?}: {}", path, e)))?;
    // Automatically add the file's parent directory to include paths
    // so that @import statements resolve relative to the file location
    let mut include_paths = vec![];
    if let Some(parent) = path.parent() {
        include_paths.push(parent.to_path_buf());
    }
    from_string_with_paths(&source, options, include_paths)
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
