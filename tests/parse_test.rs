use sasspile::lex::Lexer;
use sasspile::lex::token::Token;
use sasspile::parse::{Parser, ast::*};

fn parse(input: &str) -> Ast {
    let tokens: Vec<Token> = Lexer::new(input)
        .filter(|t| !matches!(t.as_ref(), Ok(Token::Eof)))
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
        Node::Variable {
            value: Value::BinOp(b),
            ..
        } => {
            assert_eq!(b.op, BinOpKind::Add);
            assert!(matches!(&b.right, Value::BinOp(rb) if rb.op == BinOpKind::Mul));
        }
        _ => panic!("期望 BinOp"),
    }
}

#[test]
fn test_parse_interp_not_css_if() {
    // if(#{"not"} css(): c) 应解析为 Call("if", [Arg { condition: List([Interp, Calc], Space), value: "c" }])
    let ast = parse("a { b: if(#{\"not\"} css(): c); }");
    match &ast.nodes[0] {
        Node::Rule { body, .. } => {
            assert_eq!(body.len(), 1);
            match &body[0] {
                Node::Decl { value, .. } => match value {
                    Value::Call(name, args) => {
                        assert_eq!(name, "if");
                        assert_eq!(args.len(), 1);
                        let cond = args[0].condition.as_ref().expect("应有 condition");
                        match cond {
                            Value::List(items, sep, _) => {
                                assert_eq!(*sep, Separator::Space);
                                assert_eq!(items.len(), 2);
                                assert!(matches!(&items[0], Value::Interp(_)));
                                assert!(matches!(&items[1], Value::Calc(_)));
                            }
                            _ => panic!("期望 List, 实际 {cond:#?}"),
                        }
                    }
                    _ => panic!("期望 Call, 实际 {value:#?}"),
                },
                _ => panic!("期望 Decl"),
            }
        }
        _ => panic!("期望 Rule"),
    }
}
