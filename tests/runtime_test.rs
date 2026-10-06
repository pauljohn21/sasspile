use std::sync::Arc;
use rx_scss::bus::CompilerBus;
use rx_scss::runtime::{create_runtime, EvalContext};
use rx_scss::types::Value;

#[test]
fn child_scope_arithmetic_derive() {
    let bus = Arc::new(CompilerBus::new());
    let root = EvalContext::new(bus.clone(), 1);

    let child = root.child_scope(5);
    assert_eq!(child.scope_id(), 1 * 1000 + 5);

    let grandchild = child.child_scope(3);
    assert_eq!(grandchild.scope_id(), (1 * 1000 + 5) * 1000 + 3);
}

#[test]
fn variable_scope_isolation() {
    let bus = Arc::new(CompilerBus::new());
    let root = EvalContext::new(bus.clone(), 1);
    root.bind_var("x", Value::Number(100.0));

    let child = root.child_scope(1);
    // child should inherit from parent
    assert_eq!(child.var("x"), Some(Value::Number(100.0)));

    // child's variable should not affect parent
    child.bind_var("x", Value::Number(200.0));
    assert_eq!(child.var("x"), Some(Value::Number(200.0)));
    assert_eq!(root.var("x"), Some(Value::Number(100.0)));
}

#[test]
fn create_runtime_returns_valid_pair() {
    let (ctx, _bus) = create_runtime();
    assert_eq!(ctx.scope_id(), 1);
    assert!(ctx.var("nonexistent").is_none());
}

#[test]
fn bind_and_get_var() {
    let bus = Arc::new(CompilerBus::new());
    let ctx = EvalContext::new(bus.clone(), 1);
    ctx.bind_var("color", Value::String("red".into()));
    assert_eq!(ctx.var("color"), Some(Value::String("red".into())));
}

#[test]
fn scope_id_is_unique() {
    let bus = Arc::new(CompilerBus::new());
    let root = EvalContext::new(bus.clone(), 1);
    let c1 = root.child_scope(1);
    let c2 = root.child_scope(2);
    assert_ne!(c1.scope_id(), c2.scope_id());
}
