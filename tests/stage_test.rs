//! Reactor 管线集成测试——验证直接调用 Lexer/Parser/Evaluator/Serializer。
//!
//! 替代已删除的 stage 模块测试, 确保新 Reactor 管线行为等价。

use sasspile::css::node::CssNode;
use sasspile::css::Serializer;
use sasspile::eval::reactor::Reactor;
use sasspile::OutputStyle;

// —— Reactor 管线完整性 ——

#[test]
fn test_source_creation() {
    // Reactor::new 创建管线起点
    let reactor = Reactor::new("a { color: red; }");
    assert_eq!(reactor.stage(), sasspile::eval::reactor::CompileStage::Raw);
}

#[test]
fn test_source_to_lexed() {
    // lex 后进入 Lexed 状态
    let reactor = Reactor::new("a").lex().expect("unexpected failure in test");
    assert_eq!(reactor.stage(), sasspile::eval::reactor::CompileStage::Lex);
}

#[test]
fn test_lexed_parse() {
    // 管线: Raw → Lexed → Parsed
    let reactor = Reactor::new("a { color: red; }")
        .lex()
        .expect("unexpected failure in test")
        .parse()
        .expect("unexpected failure in test");
    assert_eq!(reactor.stage(), sasspile::eval::reactor::CompileStage::Parse);
}

#[test]
fn test_parsed_evaluate() {
    // 管线: Raw → Lexed → Parsed → Evaluated (空输入产生 0 节点)
    let reactor = Reactor::new(String::new())
        .lex()
        .expect("unexpected failure in test")
        .parse()
        .expect("unexpected failure in test")
        .evaluate()
        .expect("unexpected failure in test");
    assert_eq!(reactor.stage(), sasspile::eval::reactor::CompileStage::Evaluate);
    assert!(reactor.css_nodes.is_empty());
}

// —— CSS Serializer (取代旧 Evaluated::serialize 测试) ——

#[test]
fn test_serialize_empty() {
    let css = Serializer::serialize(&[], OutputStyle::Expanded);
    assert_eq!(css, "\n");

    // 通过 Reactor 管线也产生相同结果
    let reactor_css = Reactor::new(String::new())
        .lex()
        .expect("unexpected failure in test")
        .parse()
        .expect("unexpected failure in test")
        .evaluate()
        .expect("unexpected failure in test")
        .serialize(OutputStyle::Expanded)
        .finish()
        .expect("unexpected failure in test");
    assert_eq!(reactor_css, "\n");
}

#[test]
fn test_serialize_single_decl() {
    let css = Serializer::serialize(
        &[CssNode::Declaration {
            property: "color".to_string(),
            value: "red".to_string(),
            important: false,
        }],
        OutputStyle::Expanded,
    );
    assert_eq!(css, "color: red;\n");
}

#[test]
fn test_serialize_decl() {
    let nodes = vec![CssNode::Declaration {
        property: "color".into(),
        value: "red".into(),
        important: false,
    }];
    assert_eq!(
        Serializer::serialize(&nodes, OutputStyle::Expanded),
        "color: red;\n"
    );
}

#[test]
fn test_serialize_rule() {
    let nodes = vec![CssNode::Rule {
        selector: "a".into(),
        declarations: vec![CssNode::Declaration {
            property: "color".into(),
            value: "red".into(),
            important: false,
        }],
        children: vec![],
    }];
    assert_eq!(
        Serializer::serialize(&nodes, OutputStyle::Expanded),
        "a {\n  color: red;\n}\n"
    );
}
