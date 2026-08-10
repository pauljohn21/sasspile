use sasspile::eval::Evaluator;
use sasspile::parse::ast::*;
use sasspile::css::node::CssNode;

#[test]
fn test_eval_simple() {
    let ast = Ast { nodes: vec![Node::Rule {
        selector: "a".into(),
        body: vec![Node::Decl {
            property: "color".into(),
            value: Value::String("red".into(), false),
            important: false,
        }],
    }]};
    let css = Evaluator::evaluate(&ast).unwrap();
    assert_eq!(css.len(), 1);
}

#[test]
fn test_eval_variable() {
    let ast = Ast { nodes: vec![
        Node::Variable { name: "x".into(), value: Value::Number(10.0, Some("px".into())), flags: VarFlags::default() },
        Node::Decl { property: "w".into(), value: Value::Variable("x".into()), important: false },
    ]};
    let css = Evaluator::evaluate(&ast).unwrap();
    assert_eq!(css.len(), 1);
    if let CssNode::Declaration { value, .. } = &css[0] {
        assert_eq!(value, "10px");
    }
}
