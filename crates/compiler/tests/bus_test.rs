//! Tests for CompilerBus and variable lookup.

use rxrust::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use lightforger::reactive::{CompilerBus, ValueEvent};

#[test]
fn subscribe_receives_post_sub_events() {
    let bus = CompilerBus::new();
    let collected = Rc::new(RefCell::new(Vec::new()));

    let c = collected.clone();
    bus.var_events().subscribe(move |evt| {
        let ValueEvent::Bind { scope_id, name, value } = evt else {
            unreachable!()
        };
        c.borrow_mut().push((scope_id, name, value));
    });

    bus.var_events().next(ValueEvent::Bind {
        scope_id: 0,
        name: "$size".to_string(),
        value: 100,
    });

    assert_eq!(collected.borrow().len(), 1);
    assert_eq!(
        collected.borrow()[0],
        (0, "$size".to_string(), 100)
    );
}

#[test]
fn clone_subscriptions_shared() {
    let bus = CompilerBus::new();
    let bus2 = bus.clone();
    let collected = Rc::new(RefCell::new(Vec::new()));

    let c = collected.clone();
    bus.var_events().subscribe(move |evt| {
        let ValueEvent::Bind { value, .. } = evt else {
            unreachable!()
        };
        c.borrow_mut().push(value);
    });

    bus2.var_events().next(ValueEvent::Bind {
        scope_id: 0,
        name: "$x".to_string(),
        value: 99,
    });

    assert_eq!(*collected.borrow(), vec![99]);
}
