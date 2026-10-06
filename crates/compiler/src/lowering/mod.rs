//! Lowering — 将 SassAstNode 解析树降级为 AstNode 消费型 AST
//!
//! 处理插值展开、父选择器展开、Map/List 转换、!default 语义等。

mod context;
mod lower;

pub use context::LoweringContext;
pub use lower::lower_to_ast;
