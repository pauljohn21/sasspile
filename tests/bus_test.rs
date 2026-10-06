use std::sync::{Arc, Mutex};
use std::thread;
use rxrust::prelude::*;
use rx_scss::bus::{CompilerBus, FnDef, MixinDef, ModuleDef};
use rx_scss::types::{AstNode, Param, Value};

#[test]
fn set_and_get_var() {
    let bus = CompilerBus::new();
    bus.set_var(1, "x", Value::Number(42.0));
    assert_eq!(bus.get_var(1, "x"), Some(Value::Number(42.0)));
    assert_eq!(bus.get_var(1, "y"), None);
}

#[test]
fn register_and_lookup_mixin() {
    let bus = CompilerBus::new();
    let mixin = MixinDef {
        name: "box".into(),
        params: vec![Param::new("color")],
        body: vec![AstNode::StyleDecl {
            property: "border".into(),
            value: Box::new(AstNode::Literal(Value::String("red".into()))),
        }],
    };
    bus.register_mixin(mixin);
    let found = bus.lookup_mixin("box");
    assert!(found.is_some());
    let m = found.unwrap();
    assert_eq!(m.name, "box");
    assert_eq!(m.params.len(), 1);
}

#[test]
fn register_and_lookup_fn() {
    let bus = CompilerBus::new();
    let func = FnDef {
        name: "double".into(),
        params: vec![Param::new("n")],
        body: vec![],
    };
    bus.register_fn(func);
    assert!(bus.lookup_fn("double").is_some());
    assert!(bus.lookup_fn("missing").is_none());
}

#[test]
fn register_and_lookup_module() {
    let bus = CompilerBus::new();
    let module = ModuleDef {
        path: "variables".into(),
        members: vec!["$primary".into()],
    };
    bus.register_module(module);
    assert!(bus.lookup_module("variables").is_some());
    assert!(bus.lookup_module("missing").is_none());
}

#[test]
fn multicast_var_event() {
    let bus = CompilerBus::new();
    let received = Arc::new(Mutex::new(Vec::new()));
    let r = received.clone();

    let subj = bus.var_events();
    let subj2 = bus.var_events();

    subj.subscribe(move |ev| {
        r.lock().unwrap().push(format!("{:?}", ev));
    });

    bus.set_var(1, "test", Value::Bool(true));

    // Multicast: get the same subject clone should also work
    let _ = subj2;

    let guard = received.lock().unwrap();
    assert!(!guard.is_empty(), "var event should be received");
}

#[test]
fn thread_safe_concurrent_set() {
    let bus = Arc::new(CompilerBus::new());
    let mut handles = Vec::new();

    for i in 0..10 {
        let b = bus.clone();
        handles.push(thread::spawn(move || {
            b.set_var(1, &format!("var_{}", i), Value::Number(i as f64));
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    for i in 0..10 {
        assert_eq!(
            bus.get_var(1, &format!("var_{}", i)),
            Some(Value::Number(i as f64)),
            "variable var_{} should be set",
            i
        );
    }
}

#[test]
fn var_inherits_from_parent_scope() {
    let bus = CompilerBus::new();
    bus.set_var(1, "color", Value::String("blue".into()));
    // child scope 1001 should find var from parent 1
    assert_eq!(bus.get_var(1001, "color"), Some(Value::String("blue".into())));
}
