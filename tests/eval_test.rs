use sasspile::compile_expanded;
use sasspile::css::node::CssNode;
use sasspile::eval::Evaluator;
use sasspile::parse::ast::*;

#[test]
fn test_eval_interp_not_css_if() {
    sasspile::init_tracing();
    // #{"not"} css() 应保留为 not css()（插值 not + CSS 透传 css()）
    let input = r#"a {b: if(#{"not"} css(): c)}"#;
    let css = compile_expanded(input).unwrap_or_else(|err| {
        tracing::error!("编译失败: {:?}", err);
        String::new()
    });
    tracing::info!("结果: [{}]", css);
    assert_eq!(css, "a {\n  b: if(not css(): c);\n}\n");
}

#[test]
fn test_eval_interp_and_keyword() {
    sasspile::init_tracing();
    // 测试 if(#{"and"}: c) — 插值 and 应作为 CSS 透传
    let input = r#"a {b: if(#{"and"}: c)}"#;
    let css = compile_expanded(input).unwrap_or_else(|_e| String::new());
    // and 应作为 CSS 透传，条件无法求值为 true
    assert_eq!(css, "a {\n  b: if(and: c);\n}\n");
}

#[test]
fn test_eval_simple() {
    let ast = Ast {
        nodes: vec![Node::Rule {
            selector: "a".into(),
            body: vec![Node::Decl {
                property: "color".into(),
                value: Value::String("red".into(), false),
                important: false,
            }],
        }],
    };
    let css = Evaluator::evaluate(&ast).unwrap();
    assert_eq!(css.len(), 1);
}

#[test]
fn test_eval_variable() {
    let ast = Ast {
        nodes: vec![
            Node::Variable {
                name: "x".into(),
                value: Value::Number(10.0, Some("px".into())),
                flags: VarFlags::default(),
            },
            Node::Decl {
                property: "w".into(),
                value: Value::Variable("x".into()),
                important: false,
            },
        ],
    };
    let css = Evaluator::evaluate(&ast).unwrap();
    assert_eq!(css.len(), 1);
    if let CssNode::Declaration { value, .. } = &css[0] {
        assert_eq!(value, "10px");
    }
}
