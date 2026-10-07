use std::sync::Arc;
use rx_scss::bus::CompilerBus;
use rx_scss::runtime::{create_runtime, EvalContext};
use rx_scss::types::Value;

#[test]
fn child_scope_monotonic_ids() {
    let bus = Arc::new(CompilerBus::new());
    let root = EvalContext::new(bus.clone(), 1);

    // With monotonic counter, child scope_ids are unique and increasing
    let child1 = root.child_scope(5);
    let child2 = root.child_scope(7);
    assert_ne!(child1.scope_id(), root.scope_id(), "child must differ from root");
    assert_ne!(child2.scope_id(), root.scope_id(), "child must differ from root");
    assert_ne!(child1.scope_id(), child2.scope_id(), "siblings must differ");

    // Grandchild has its own unique id
    let grandchild = child1.child_scope(3);
    assert_ne!(grandchild.scope_id(), child1.scope_id());
    assert_ne!(grandchild.scope_id(), root.scope_id());
}

#[test]
fn variable_scope_isolation() {
    let bus = Arc::new(CompilerBus::new());
    let root = EvalContext::new(bus.clone(), 1);
    root.bind_var("x", Value::Number(100.0, None));

    let child = root.child_scope(1);
    // child should inherit from parent
    assert_eq!(child.var("x"), Some(Value::Number(100.0, None)));

    // child's variable should not affect parent
    child.bind_var("x", Value::Number(200.0, None));
    assert_eq!(child.var("x"), Some(Value::Number(200.0, None)));
    assert_eq!(root.var("x"), Some(Value::Number(100.0, None)));
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
