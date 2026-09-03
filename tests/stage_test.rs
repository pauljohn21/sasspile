//! 编译管线阶段测试——Source → Lexed → Parsed → Evaluated → Serialized。
//!
//! 物理隔离：所有阶段类型测试集中于此，不使用内联 #[cfg(test)] 模块。
//! 同时包含 CSS Serializer 的单元测试。

use sasspile::OutputStyle;
use sasspile::css::Serializer;
use sasspile::css::node::CssNode;
use sasspile::stage::evaluated::Evaluated;
use sasspile::stage::serialized::Serialized;
use sasspile::stage::source::Source;

// —— Source 阶段 ——

#[test]
fn test_source_creation() {
    let src = Source::new("a { color: red; }".to_string());
    assert_eq!(src.text, "a { color: red; }");
}

#[test]
fn test_source_to_lexed() {
    let src = Source::new("a".to_string());
    let lexed = src.lex().unwrap();
    assert_eq!(lexed.tokens.len(), 1);
}

// —— Lexed 阶段 ——

#[test]
fn test_lexed_parse() {
    // 链式：Source → Lexed → Parsed
    let parsed = Source::new("a { color: red; }".to_string())
        .lex()
        .unwrap()
        .parse()
        .unwrap();
    assert!(!parsed.ast.nodes.is_empty());
}

// —— Parsed 阶段 ——

#[test]
fn test_parsed_evaluate() {
    // 链式：Source → Lexed → Parsed → Evaluated
    let evaluated = Source::new(String::new())
        .lex()
        .unwrap()
        .parse()
        .unwrap()
        .evaluate()
        .unwrap();
    assert!(evaluated.nodes.is_empty());
}

// —— Evaluated 阶段 ——

#[test]
fn test_serialize_empty() {
    let evaluated = Evaluated { nodes: vec![] };
    let serialized = evaluated.serialize(OutputStyle::Expanded);
    assert_eq!(serialized.css, "\n");
}

#[test]
fn test_serialize_single_decl() {
    let evaluated = Evaluated {
        nodes: vec![CssNode::Declaration {
            property: "color".to_string(),
            value: "red".to_string(),
            important: false,
        }],
    };
    let serialized = evaluated.serialize(OutputStyle::Expanded);
    assert_eq!(serialized.css, "color: red;\n");
}

// —— Serialized 阶段 ——

#[test]
fn test_serialized_display() {
    let s = Serialized {
        css: "a{color:red;}".to_string(),
    };
    assert_eq!(format!("{s}"), "a{color:red;}");
}

#[test]
fn test_serialized_as_ref() {
    let s = Serialized {
        css: "test".to_string(),
    };
    assert_eq!(s.as_ref(), "test");
}

// —— CSS Serializer ——

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
