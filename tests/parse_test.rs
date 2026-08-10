use sasspile::parse::{Parser, ast::*};
use sasspile::lex::Lexer;
use sasspile::lex::token::Token;

fn parse(input: &str) -> Ast {
    let tokens: Vec<Token> = Lexer::new(input)
        .filter(|t| !matches!(t.as_ref(), Ok(Token::Whitespace) | Ok(Token::Eof)))
        .map(|t| t.unwrap())
        .collect();
    Parser::parse(&tokens).unwrap()
}

#[test]
fn test_parse_rule() {
    let ast = parse("a { color: red; }");
    assert_eq!(ast.nodes.len(), 1);
    assert!(matches!(&ast.nodes[0], Node::Rule { selector, .. } if selector == "a"));
}

#[test]
fn test_parse_variable() {
    let ast = parse("$w: 10px;");
    match &ast.nodes[0] {
        Node::Variable { name, value, .. } => {
            assert_eq!(name, "w");
            assert_eq!(*value, Value::Number(10.0, Some("px".to_string())));
        }
        _ => panic!(),
    }
}

#[test]
fn test_parse_if() {
    let ast = parse("@if true { a { color: red; } }");
    assert!(matches!(&ast.nodes[0], Node::If { .. }));
}

#[test]
fn test_parse_mixin() {
    let ast = parse("@mixin foo($x) { color: $x; }");
    assert!(matches!(&ast.nodes[0], Node::MixinDef { name, .. } if name == "foo"));
}

#[test]
fn test_parse_expr_precedence() {
    let ast = parse("$x: 1 + 2 * 3;");
    match &ast.nodes[0] {
        Node::Variable { value: Value::BinOp(b), .. } => {
            assert_eq!(b.op, BinOpKind::Add);
            assert!(matches!(&b.right, Value::BinOp(rb) if rb.op == BinOpKind::Mul));
        }
        _ => panic!("期望 BinOp"),
    }
}
