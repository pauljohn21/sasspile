use rx_scss::bus::{CompilerBus, FnDef, MixinDef};
use rx_scss::runtime::EvalContext;
use rx_scss::types::*;
use std::sync::Arc;

#[test]
fn eval_context_variable_binding() {
    let bus = Arc::new(CompilerBus::new());
    let ctx = EvalContext::new(bus, 1);
    ctx.bind_var("color", Value::String("red".into()));
    let val = ctx.var("color");
    assert_eq!(val, Some(Value::String("red".into())));
}

#[test]
fn eval_context_child_scope_inherits_parent_vars() {
    let bus = Arc::new(CompilerBus::new());
    let parent = EvalContext::new(bus.clone(), 1);
    parent.bind_var("base", Value::Number(10.0));

    let child = parent.child_scope(1);
    let val = child.var("base");
    assert_eq!(val, Some(Value::Number(10.0)));
}

#[test]
fn eval_context_shallow_binding_precedence() {
    let bus = Arc::new(CompilerBus::new());
    let parent = EvalContext::new(bus.clone(), 1);
    parent.bind_var("x", Value::String("parent".into()));

    let child = parent.child_scope(1);
    child.bind_var("x", Value::String("child".into()));

    assert_eq!(child.var("x"), Some(Value::String("child".into())));
    assert_eq!(parent.var("x"), Some(Value::String("parent".into())));
}

#[test]
fn eval_context_nonexistent_var_returns_null() {
    let bus = Arc::new(CompilerBus::new());
    let ctx = EvalContext::new(bus, 1);
    let val = ctx.var("nonexistent");
    assert_eq!(val, None);
}

#[test]
fn bus_register_and_lookup_mixin() {
    let bus = CompilerBus::new();
    bus.register_mixin(MixinDef {
        name: "box".into(),
        params: vec![Param::new("$size")],
        body: vec![],
    });

    let mixin = bus.lookup_mixin("box");
    assert!(mixin.is_some());
    let m = mixin.unwrap();
    assert_eq!(m.name, "box");
    assert_eq!(m.params.len(), 1);
}

#[test]
fn bus_register_and_lookup_function() {
    let bus = CompilerBus::new();
    bus.register_fn(FnDef {
        name: "double".into(),
        params: vec![Param::new("$n")],
        body: vec![],
    });

    let func = bus.lookup_fn("double");
    assert!(func.is_some());
    let f = func.unwrap();
    assert_eq!(f.name, "double");
}

#[test]
fn bus_var_update_emits_event() {
    let bus = CompilerBus::new();
    bus.set_var(1, "color", Value::String("red".into()));
    bus.set_var(1, "color", Value::String("blue".into()));

    let val = bus.get_var(1, "color");
    assert_eq!(val, Some(Value::String("blue".into())));
}

#[test]
fn value_display_number() {
    assert_eq!(Value::Number(42.0).to_string(), "42");
    assert_eq!(Value::Number(3.14).to_string(), "3.14");
}

#[test]
fn value_display_color() {
    let color = Value::Color(255, 0, 0, 255);
    assert_eq!(color.to_string(), "#ff0000");

    let color_with_alpha = Value::Color(255, 0, 0, 128);
    assert_eq!(color_with_alpha.to_string(), "#ff000080");
}

#[test]
fn value_display_list() {
    let list = Value::List(vec![
        Value::Number(1.0),
        Value::Number(2.0),
    ]);
    assert_eq!(list.to_string(), "(1, 2)");
}

#[test]
fn value_equality() {
    assert_eq!(Value::Number(1.0), Value::Number(1.0));
    assert_eq!(Value::String("a".into()), Value::String("a".into()));
    assert_eq!(Value::Bool(true), Value::Bool(true));
    assert_eq!(Value::Null, Value::Null);
    assert_ne!(Value::Number(1.0), Value::Number(2.0));
}

#[test]
fn css_stmt_rule_is_invisible_when_empty() {
    let empty = CssStmt::Rule {
        selector: "x".into(),
        inner: vec![],
    };
    assert!(empty.is_invisible());

    let non_empty = CssStmt::Rule {
        selector: "y".into(),
        inner: vec![CssStmt::Decl {
            property: "color".into(),
            value: "red".into(),
        }],
    };
    assert!(!non_empty.is_invisible());
}

#[test]
fn css_stmt_media_is_invisible_when_empty() {
    let empty = CssStmt::Media {
        query: "screen".into(),
        inner: vec![],
    };
    assert!(empty.is_invisible());
}

#[test]
fn param_default_value() {
    let p = Param::with_default("size", AstNode::Literal(Value::Number(10.0)));
    assert_eq!(p.name, "size");
    assert!(p.default_value.is_some());
    match p.default_value.unwrap().as_ref() {
        AstNode::Literal(Value::Number(n)) => assert_eq!(*n, 10.0),
        other => panic!("expected Number literal, got {:?}", other),
    }
}

#[test]
fn ast_node_variable_decl() {
    let node = AstNode::VariableDecl {
        name: "primary".into(),
        value: Box::new(AstNode::Literal(Value::String("blue".into()))),
        scope_id: 1,
    };
    match node {
        AstNode::VariableDecl { name, scope_id, .. } => {
            assert_eq!(name, "primary");
            assert_eq!(scope_id, 1);
        }
        _ => panic!("expected VariableDecl"),
    }
}
