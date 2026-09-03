use sasspile::OutputStyle;
use sasspile::compile_expanded;
use sasspile::css::node::CssNode;
use sasspile::stage::source::Source;

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
    // 链式调用：Source → Lexed → Parsed → Evaluated → Serialized
    let css = Source::new("a { color: red; }".to_string())
        .lex()
        .unwrap()
        .parse()
        .unwrap()
        .evaluate()
        .unwrap()
        .serialize(OutputStyle::Expanded)
        .into_string();
    assert!(css.contains("color: red"));
}

#[test]
fn test_eval_variable() {
    let input = "$x: 10px; a { w: $x; }";
    let nodes = Source::new(input.to_string())
        .lex()
        .unwrap()
        .parse()
        .unwrap()
        .evaluate()
        .unwrap();
    // 验证变量求值结果——a 规则的第一个声明值应为 10px
    if let Some(CssNode::Rule { declarations, .. }) = nodes.nodes.first()
        && let Some(CssNode::Declaration { value, .. }) = declarations.first()
    {
        assert_eq!(value, "10px");
    }
}
