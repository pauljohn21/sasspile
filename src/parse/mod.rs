//! 语法分析器——递归下降 + Result 组合。
//!
//! 纯函数风格：解析器不携带可变状态，通过索引传递。

pub mod ast;
mod parser;
mod utils;
mod value;

pub use ast::{Ast, Color, Node, Separator, Value};
pub use parser::Parser;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::token::Token;

    fn lex(input: &str) -> Vec<Token> {
        crate::lex::Lexer::new(input)
            .filter(|t| !matches!(t.as_ref(), Ok(Token::Whitespace) | Ok(Token::Eof)))
            .map(|t| t.unwrap())
            .collect::<Vec<_>>()
    }

    #[test]
    fn test_parse_simple_rule() {
        let tokens = lex("a { color: red; }");
        let ast = Parser::parse(&tokens).unwrap();
        assert_eq!(ast.nodes.len(), 1);
        match &ast.nodes[0] {
            Node::Rule { selector, body } => {
                assert_eq!(selector, "a");
                assert_eq!(body.len(), 1);
            }
            _ => panic!("期望 Rule 节点"),
        }
    }

    #[test]
    fn test_parse_decl() {
        let tokens = lex("color: red;");
        let ast = Parser::parse(&tokens).unwrap();
        assert_eq!(ast.nodes.len(), 1);
        match &ast.nodes[0] {
            Node::Decl {
                property,
                value,
                important,
            } => {
                assert_eq!(property, "color");
                assert_eq!(*value, Value::String("red".to_string(), false));
                assert!(!important);
            }
            _ => panic!("期望 Decl 节点"),
        }
    }

    #[test]
    fn test_parse_variable() {
        let tokens = lex("$w: 10px;");
        let ast = Parser::parse(&tokens).unwrap();
        assert_eq!(ast.nodes.len(), 1);
        match &ast.nodes[0] {
            Node::Variable { name, value } => {
                assert_eq!(name, "w");
                assert_eq!(*value, Value::Number(10.0, Some("px".to_string())));
            }
            _ => panic!("期望 Variable 节点"),
        }
    }

    #[test]
    fn test_parse_nested_rule() {
        let tokens = lex(".outer { .inner { color: red; } }");
        let ast = Parser::parse(&tokens).unwrap();
        assert_eq!(ast.nodes.len(), 1);
        match &ast.nodes[0] {
            Node::Rule { selector, body } => {
                assert_eq!(selector, ".outer");
                assert_eq!(body.len(), 1);
            }
            _ => panic!("期望 Rule 节点"),
        }
    }

    #[test]
    fn test_parse_at_rule() {
        let tokens = lex("@media (max-width: 768px) { a { color: red; } }");
        let ast = Parser::parse(&tokens).unwrap();
        assert_eq!(ast.nodes.len(), 1);
        match &ast.nodes[0] {
            Node::AtRule { name, params, body } => {
                assert_eq!(name, "media");
                assert!(params.is_some());
                assert!(body.is_some());
            }
            _ => panic!("期望 AtRule 节点"),
        }
    }
}
