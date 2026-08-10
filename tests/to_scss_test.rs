use sasspile::parse::ast::*;

#[test]
fn test_rule_to_scss() {
    let node = Node::Rule {
        selector: "a".into(),
        body: vec![Node::Decl {
            property: "color".into(),
            value: Value::String("red".into(), false),
            important: false,
        }],
    };
    let scss = node.to_scss(0);
    assert!(scss.contains("a {"));
    assert!(scss.contains("color: red;"));
    assert!(scss.contains("}"));
}

#[test]
fn test_decl_to_scss() {
    let node = Node::Decl {
        property: "width".into(),
        value: Value::Number(100.0, Some("px".into())),
        important: true,
    };
    let scss = node.to_scss(0);
    assert_eq!(scss, "width: 100px !important;");
}

#[test]
fn test_variable_to_scss() {
    let node = Node::Variable {
        name: "color".into(),
        value: Value::String("blue".into(), false),
        flags: VarFlags { default: true, global: false },
    };
    let scss = node.to_scss(0);
    assert_eq!(scss, "$color: blue !default;");
}

#[test]
fn test_comment_to_scss() {
    let silent = Node::Comment("hello".into(), true);
    let loud = Node::Comment("world".into(), false);
    assert_eq!(silent.to_scss(0), "// hello");
    assert_eq!(loud.to_scss(0), "/* world */");
}

#[test]
fn test_if_to_scss() {
    let node = Node::If {
        branches: vec![(Value::Bool(true), vec![Node::Decl {
            property: "color".into(), value: Value::String("red".into(), false), important: false,
        }])],
        else_body: Some(vec![Node::Decl {
            property: "color".into(), value: Value::String("blue".into(), false), important: false,
        }]),
    };
    let scss = node.to_scss(0);
    assert!(scss.contains("@if true"));
    assert!(scss.contains("@else"));
}

#[test]
fn test_for_to_scss() {
    let node = Node::For {
        var: "i".into(),
        from: Value::Number(1.0, None),
        to: Value::Number(10.0, None),
        inclusive: true,
        body: vec![Node::Decl {
            property: "w".into(), value: Value::Variable("i".into()), important: false,
        }],
    };
    let scss = node.to_scss(0);
    assert!(scss.contains("@for $i from 1 through 10"));
}

#[test]
fn test_include_to_scss() {
    let node = Node::Include {
        name: "my-mixin".into(),
        args: vec![],
        content: None,
    };
    assert_eq!(node.to_scss(0), "@include my-mixin;");
}

#[test]
fn test_extend_to_scss() {
    let node = Node::Extend { selector: ".btn".into(), optional: true };
    assert_eq!(node.to_scss(0), "@extend .btn !optional;");
}

#[test]
fn test_use_to_scss() {
    let node = Node::Use {
        url: "sass:color".into(), namespace: None, star: false, config: vec![],
    };
    assert_eq!(node.to_scss(0), "@use \"sass:color\";");
}

#[test]
fn test_return_to_scss() {
    let node = Node::Return(Value::Number(42.0, None));
    assert_eq!(node.to_scss(0), "@return 42;");
}

#[test]
fn test_content_to_scss() {
    let node = Node::Content;
    assert_eq!(node.to_scss(0), "@content;");
}
