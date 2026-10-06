//! lower_to_ast — 降级转换的核心函数

use super::context::LoweringContext;
use crate::parser::SassAstNode;
use crate::reactive::AstNode;
use crate::Error;

/// 将 SassAstNode 降级为 AstNode
pub fn lower_to_ast(
    _node: SassAstNode,
    _ctx: &mut LoweringContext,
) -> Result<AstNode, Error> {
    // TODO: implement lowering
    Err(Error::msg("lowering: not yet implemented"))
}
