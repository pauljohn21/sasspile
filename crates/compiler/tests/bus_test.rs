//! Tests for CompilerBus and variable lookup.

use std::sync::{Arc, Mutex};

use rxrust::prelude::*;

use lightforger::reactive::{CompilerBus, ValueEvent};

#[test]
fn subscribe_receives_post_sub_events() {
    let bus = CompilerBus::new();
    let collected = Arc::new(Mutex::new(Vec::new()));

    let c = collected.clone();
    bus.var_events().subscribe(move |evt| {
        let ValueEvent::Bind { scope_id, name, value } = evt else {
            unreachable!()
        };
        c.lock().unwrap().push((scope_id, name, value));
    });

    bus.var_events().next(ValueEvent::Bind {
        scope_id: 0,
        name: "$size".to_string(),
        value: 100,
    });

    assert_eq!(collected.lock().unwrap().len(), 1);
    assert_eq!(
        collected.lock().unwrap()[0],
        (0, "$size".to_string(), 100)
    );
}

#[test]
fn clone_subscriptions_shared() {
    let bus = CompilerBus::new();
    let bus2 = bus.clone();
    let collected = Arc::new(Mutex::new(Vec::new()));

    let c = collected.clone();
    bus.var_events().subscribe(move |evt| {
        let ValueEvent::Bind { value, .. } = evt else {
            unreachable!()
        };
        c.lock().unwrap().push(value);
    });

    bus2.var_events().next(ValueEvent::Bind {
        scope_id: 0,
        name: "$x".to_string(),
        value: 99,
    });

    assert_eq!(*collected.lock().unwrap(), vec![99]);
}
