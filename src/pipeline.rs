use std::path::{Path, PathBuf};
use crate::lexer::scan;
use crate::runtime::create_runtime;
use crate::serialize::{serialize_stream, Options};
use crate::types::CompileError;

pub fn from_string(source: &str, options: &Options) -> Result<String, CompileError> {
    from_string_with_paths(source, options, Vec::new())
}

/// 编译入口：组合 Lexer → Parser → Evaluator → Serialize 管线
///
/// 所有 stage 内部用具体类型流动，仅在此 API 入口统一 box_it() 擦除类型。
pub fn from_string_with_paths(source: &str, options: &Options, include_paths: Vec<PathBuf>) -> Result<String, CompileError> {
    // Stage 1: Lexer - String → TokenStream (concrete type)
    let tokens = scan(source);
    // 创建运行时上下文
    let (ctx, _bus) = create_runtime();
    // Stage 2: Parser - TokenStream → AstStream (concrete type)
    let ast = crate::parser::parse_stream_with_paths(tokens, ctx.scope_id(), include_paths);
    // Stage 3: Evaluator - AstStream → CssStream (concrete type)
    let css = crate::eval::eval_stream(ast, ctx);
    // Stage 4: Serializer - CssStream → String stream (concrete type)
    let output = serialize_stream(css, options.clone());
    // OutputStream 已是类型擦除的 SharedBoxedObservable，直接终端消费
    let chunks: Vec<String> = crate::collect_boxed(output)?;
    Ok(chunks.join(""))
}

pub fn from_path(path: &Path, options: &Options) -> Result<String, CompileError> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| CompileError::Io(format!("Failed to read {:?}: {}", path, e)))?;
    let mut include_paths = vec![];
    if let Some(parent) = path.parent() {
        include_paths.push(parent.to_path_buf());
    }
    from_string_with_paths(&source, options, include_paths)
}
