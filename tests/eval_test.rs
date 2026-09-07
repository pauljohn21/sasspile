use sasspile::css::node::CssNode;
use sasspile::eval::reactor::Reactor;
use sasspile::compile_expanded;

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
    // 通过 Reactor 管线编译
    let css = Reactor::new("a { color: red; }")
        .lex()
        .unwrap()
        .parse()
        .unwrap()
        .evaluate()
        .unwrap()
        .serialize(sasspile::OutputStyle::Expanded)
        .finish()
        .unwrap();
    assert!(css.contains("color: red"));
}

#[test]
fn test_eval_variable() {
    let input = "$x: 10px; a { w: $x; }";
    let reactor = Reactor::new(input)
        .lex()
        .unwrap()
        .parse()
        .unwrap()
        .evaluate()
        .unwrap();
    let nodes = reactor.css_nodes;
    // 验证变量求值结果——a 规则的第一个声明值应为 10px
    if let Some(CssNode::Rule { declarations, .. }) = nodes.first()
        && let Some(CssNode::Declaration { value, .. }) = declarations.first()
    {
        assert_eq!(value, "10px");
    }
}
