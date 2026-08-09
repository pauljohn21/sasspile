//! 阶段 3: Parsed —— 抽象语法树。

use super::evaluated::Evaluated;
use crate::error::Result;
use crate::parse::ast::Ast;

/// 语法分析产物——AST。
#[derive(Debug, Clone)]
pub struct Parsed {
    /// 抽象语法树。
    pub ast: Ast,
}

impl Parsed {
    /// 求值——Parsed → Evaluated。
    pub fn evaluate(self) -> Result<Evaluated> {
        use crate::eval::Evaluator;

        let nodes = Evaluator::evaluate(&self.ast)?;
        Ok(Evaluated { nodes })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::ast::Ast;

    #[test]
    fn test_parsed_evaluate() {
        let parsed = Parsed {
            ast: Ast::default(),
        };
        let evaluated = parsed.evaluate().unwrap();
        assert!(evaluated.nodes.is_empty());
    }
}
