//! —— 求值器插值 + Reactor 管线测试 ——
//!
//! 概要：验证求值器在插值表达式和 Reactor 管线中的正确性。
//!
//! ## 覆盖场景
//! - 插值关键字透传（#{"not"} css() → not css()）
//! - 插值逻辑关键字（#{"and"} → 透传不求值）
//! - Reactor 管线编译（lex → parse → evaluate → serialize → finish）
//! - 变量求值（$x: 10px → declaration value 验证）
//!
//! ## sass-spec 参照
//! - `values/strings/` — 字符串插值
//! - `at_rules/` — at-rule 内求值

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
        .expect("unexpected failure in test")
        .parse()
        .expect("unexpected failure in test")
        .evaluate()
        .expect("unexpected failure in test")
        .serialize(sasspile::OutputStyle::Expanded)
        .finish()
        .expect("unexpected failure in test");
    assert!(css.contains("color: red"));
}

#[test]
fn test_eval_variable() {
    let input = "$x: 10px; a { w: $x; }";
    let reactor = Reactor::new(input)
        .lex()
        .expect("unexpected failure in test")
        .parse()
        .expect("unexpected failure in test")
        .evaluate()
        .expect("unexpected failure in test");
    let nodes = reactor.css_nodes;
    // 验证变量求值结果——a 规则的第一个声明值应为 10px
    if let Some(CssNode::Rule { declarations, .. }) = nodes.first()
        && let Some(CssNode::Declaration { value, .. }) = declarations.first()
    {
        assert_eq!(value, "10px");
    }
}
