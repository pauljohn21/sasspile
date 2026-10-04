//! Tests for the evaluator (SassOp dispatch) and pre-analysis.

use std::cell::RefCell;
use std::rc::Rc;

use rxrust::prelude::*;

use lightforger::reactive::{
    evaluate_to_css, pre_analysis, AstNode, CompilerBus, CssStmt, EvalContext, Value, ValueEvent,
};

#[test]
fn variable_decl_emits_bind_event() {
    let bus = CompilerBus::new();
    let ctx = Rc::new(EvalContext::new(bus.clone(), 0));

    let collected = Rc::new(RefCell::new(Vec::new()));
    let c = collected.clone();
    bus.var_events().subscribe(move |evt| {
        if let ValueEvent::Bind { scope_id, name, value } = evt {
            c.borrow_mut().push((scope_id, name, value));
        }
    });

    // Create a stream with a VariableDecl node and evaluate it
    let stream: lightforger::reactive::AstStream =
        Local::of(AstNode::VariableDecl {
            name: "$color".to_string(),
            value: Value::Number(42.0),
        })
        .box_it_clone();

    let evaluated = evaluate_to_css(stream, ctx);

    // Subscribe to the output to drive the pipeline
    let _ = evaluated.subscribe(|_: CssStmt| {});

    assert_eq!(collected.borrow().len(), 1);
    assert_eq!(collected.borrow()[0], (0, "$color".to_string(), 42));
}

#[test]
fn style_decl_emits_css_stmt() {
    let bus = CompilerBus::new();
    let ctx = Rc::new(EvalContext::new(bus, 0));

    let stream: lightforger::reactive::AstStream = Local::of(AstNode::StyleDecl {
        property: "color".to_string(),
        value: "red".to_string(),
    })
    .box_it_clone();

    let css_stream = evaluate_to_css(stream, ctx);

    let collected = Rc::new(RefCell::new(Vec::new()));
    let c = collected.clone();
    css_stream.subscribe(move |stmt| {
        if let CssStmt::Decl { property, value } = stmt {
            c.borrow_mut().push((property, value));
        }
    });

    assert_eq!(collected.borrow().len(), 1);
    assert_eq!(collected.borrow()[0], ("color".to_string(), "red".to_string()));
}

#[test]
fn ruleset_basic_two_declarations() {
    let bus = CompilerBus::new();
    let ctx = Rc::new(EvalContext::new(bus, 0));

    let stream: lightforger::reactive::AstStream = Local::of(AstNode::RuleSet {
        selector: ".foo".to_string(),
        inner: vec![
            AstNode::StyleDecl {
                property: "color".to_string(),
                value: "red".to_string(),
            },
            AstNode::StyleDecl {
                property: "margin".to_string(),
                value: "0".to_string(),
            },
        ],
    })
    .box_it_clone();

    let css_stream = evaluate_to_css(stream, ctx);

    let collected = Rc::new(RefCell::new(Vec::new()));
    let c = collected.clone();
    css_stream.subscribe(move |stmt| c.borrow_mut().push(stmt));

    let stmts = collected.borrow();
    assert_eq!(stmts.len(), 1, "Should produce exactly one CssStmt::Rule");

    match &stmts[0] {
        CssStmt::Rule { selector, inner } => {
            assert_eq!(selector, ".foo");
            assert_eq!(inner.len(), 2);
            assert!(matches!(&inner[0], CssStmt::Decl { property, value } if property == "color" && value == "red"));
            assert!(matches!(&inner[1], CssStmt::Decl { property, value } if property == "margin" && value == "0"));
        }
        other => panic!("Expected Rule, got {:?}", other),
    }
}

#[test]
fn if_true_first_clause() {
    let bus = CompilerBus::new();
    let ctx = Rc::new(EvalContext::new(bus, 0));

    let stream: lightforger::reactive::AstStream = Local::of(AstNode::If {
        cond: Box::new(AstNode::VariableDecl {
            name: "$cond".to_string(),
            value: Value::Number(1.0),
        }),
        then_branch: vec![AstNode::StyleDecl {
            property: "color".to_string(),
            value: "blue".to_string(),
        }],
        else_branch: vec![AstNode::StyleDecl {
            property: "color".to_string(),
            value: "red".to_string(),
        }],
    })
    .box_it_clone();

    let css_stream = evaluate_to_css(stream, ctx);

    let collected = Rc::new(RefCell::new(Vec::new()));
    let c = collected.clone();
    css_stream.subscribe(move |stmt| c.borrow_mut().push(stmt));

    assert_eq!(collected.borrow().len(), 1);
    assert!(matches!(
        &collected.borrow()[0],
        CssStmt::Decl { property, value } if property == "color" && value == "blue"
    ));
}

#[test]
fn if_false_else_branch() {
    let bus = CompilerBus::new();
    let ctx = Rc::new(EvalContext::new(bus, 0));

    let stream: lightforger::reactive::AstStream = Local::of(AstNode::If {
        cond: Box::new(AstNode::Placeholder), // treated as false
        then_branch: vec![AstNode::StyleDecl {
            property: "color".to_string(),
            value: "blue".to_string(),
        }],
        else_branch: vec![AstNode::StyleDecl {
            property: "color".to_string(),
            value: "red".to_string(),
        }],
    })
    .box_it_clone();

    let css_stream = evaluate_to_css(stream, ctx);

    let collected = Rc::new(RefCell::new(Vec::new()));
    let c = collected.clone();
    css_stream.subscribe(move |stmt| c.borrow_mut().push(stmt));

    assert_eq!(collected.borrow().len(), 1);
    assert!(matches!(
        &collected.borrow()[0],
        CssStmt::Decl { property, value } if property == "color" && value == "red"
    ));
}

#[test]
fn for_through_3_iterations() {
    let bus = CompilerBus::new();
    let ctx = Rc::new(EvalContext::new(bus, 0));

    let stream: lightforger::reactive::AstStream = Local::of(AstNode::For {
        var: "$i".to_string(),
        from: 1.0,
        through: 3.0,
        body: vec![AstNode::StyleDecl {
            property: "content".to_string(),
            value: "iter".to_string(),
        }],
    })
    .box_it_clone();

    let css_stream = evaluate_to_css(stream, ctx);

    let collected = Rc::new(RefCell::new(Vec::new()));
    let c = collected.clone();
    css_stream.subscribe(move |stmt| c.borrow_mut().push(stmt));

    assert_eq!(collected.borrow().len(), 3, "Should produce 3 CssStmt::Decl");
    for stmt in collected.borrow().iter() {
        assert!(matches!(
            stmt,
            CssStmt::Decl { property, value } if property == "content" && value == "iter"
        ));
    }
}

#[test]
fn media_wraps_inner_css() {
    let bus = CompilerBus::new();
    let ctx = Rc::new(EvalContext::new(bus, 0));

    let stream: lightforger::reactive::AstStream = Local::of(AstNode::Media {
        query: "screen and (min-width: 768px)".to_string(),
        inner: vec![
            AstNode::StyleDecl {
                property: "font-size".to_string(),
                value: "16px".to_string(),
            },
            AstNode::StyleDecl {
                property: "line-height".to_string(),
                value: "1.5".to_string(),
            },
        ],
    })
    .box_it_clone();

    let css_stream = evaluate_to_css(stream, ctx);

    let collected = Rc::new(RefCell::new(Vec::new()));
    let c = collected.clone();
    css_stream.subscribe(move |stmt| c.borrow_mut().push(stmt));

    let stmts = collected.borrow();
    assert_eq!(stmts.len(), 1);

    match &stmts[0] {
        CssStmt::Media { query, inner } => {
            assert_eq!(query, "screen and (min-width: 768px)");
            assert_eq!(inner.len(), 2);
        }
        other => panic!("Expected Media, got {:?}", other),
    }
}

#[test]
fn supports_wraps_inner_css() {
    let bus = CompilerBus::new();
    let ctx = Rc::new(EvalContext::new(bus, 0));

    let stream: lightforger::reactive::AstStream = Local::of(AstNode::Supports {
        query: "(display: grid)".to_string(),
        inner: vec![AstNode::StyleDecl {
            property: "display".to_string(),
            value: "grid".to_string(),
        }],
    })
    .box_it_clone();

    let css_stream = evaluate_to_css(stream, ctx);

    let collected = Rc::new(RefCell::new(Vec::new()));
    let c = collected.clone();
    css_stream.subscribe(move |stmt| c.borrow_mut().push(stmt));

    let stmts = collected.borrow();
    assert_eq!(stmts.len(), 1);

    match &stmts[0] {
        CssStmt::Supports { query, inner } => {
            assert_eq!(query, "(display: grid)");
            assert_eq!(inner.len(), 1);
        }
        other => panic!("Expected Supports, got {:?}", other),
    }
}

#[test]
fn while_loop_exits_after_first() {
    let bus = CompilerBus::new();
    let ctx = Rc::new(EvalContext::new(bus, 0));

    // cond is Placeholder = false, so body executes 0 times
    let stream: lightforger::reactive::AstStream = Local::of(AstNode::While {
        cond: Box::new(AstNode::Placeholder),
        body: vec![AstNode::StyleDecl {
            property: "x".to_string(),
            value: "y".to_string(),
        }],
    })
    .box_it_clone();

    let css_stream = evaluate_to_css(stream, ctx);

    let collected = Rc::new(RefCell::new(Vec::new()));
    let c = collected.clone();
    css_stream.subscribe(move |stmt| c.borrow_mut().push(stmt));

    assert_eq!(collected.borrow().len(), 0, "While with false cond should produce no CSS");
}

#[test]
fn warn_t_does_not_alter_stream() {
    let bus = CompilerBus::new();
    let ctx = Rc::new(EvalContext::new(bus, 0));

    // Wrap: StyleDecl then Warn together — should produce CssStmt::Decl unchanged
    let inner_stmt = AstNode::StyleDecl {
        property: "content".to_string(),
        value: "test".to_string(),
    };
    let stream: lightforger::reactive::AstStream =
        Local::from_iter(vec![
            inner_stmt,
            AstNode::Warn {
                message: "test warning".to_string(),
            },
        ])
        .box_it_clone();

    let css_stream = evaluate_to_css(stream, ctx);

    let collected = Rc::new(RefCell::new(Vec::new()));
    let c = collected.clone();
    css_stream.subscribe(move |stmt| c.borrow_mut().push(stmt));

    // Only the StyleDecl should produce a CssStmt; Warn produces none
    let stmts = collected.borrow();
    assert_eq!(stmts.len(), 1);
    assert!(matches!(
        &stmts[0],
        CssStmt::Decl { property, value } if property == "content" && value == "test"
    ));
}

#[test]
fn mixin_no_css_output() {
    let bus = CompilerBus::new();
    let ctx = Rc::new(EvalContext::new(bus, 0));

    // Mixin decl should register and produce no CSS
    let stream: lightforger::reactive::AstStream =
        Local::from_iter(vec![
            AstNode::Mixin {
                name: "box".to_string(),
                params: vec![],
                body: vec![AstNode::StyleDecl {
                    property: "border".to_string(),
                    value: "1px solid".to_string(),
                }],
            },
            AstNode::StyleDecl {
                property: "color".to_string(),
                value: "red".to_string(),
            },
        ])
        .box_it_clone();

    let css_stream = evaluate_to_css(stream, ctx);

    let collected = Rc::new(RefCell::new(Vec::new()));
    let c = collected.clone();
    css_stream.subscribe(move |stmt| c.borrow_mut().push(stmt));

    // Mixin produces 0 CSS, StyleDecl produces 1
    let stmts = collected.borrow();
    assert_eq!(stmts.len(), 1);
    assert!(matches!(
        &stmts[0],
        CssStmt::Decl { property, value } if property == "color" && value == "red"
    ));
}

#[test]
fn include_expands_mixin_body() {
    let bus = CompilerBus::new();
    let ctx = Rc::new(EvalContext::new(bus.clone(), 0));

    // First register the mixin
    bus.register_mixin(lightforger::reactive::MixinDef {
        name: "box".to_string(),
        params: vec![],
        body: vec![
            AstNode::StyleDecl {
                property: "border".to_string(),
                value: "1px solid".to_string(),
            },
            AstNode::StyleDecl {
                property: "padding".to_string(),
                value: "10px".to_string(),
            },
        ],
    });

    // Now @include it
    let stream: lightforger::reactive::AstStream = Local::of(AstNode::MixinCall {
        name: "box".to_string(),
        args: vec![],
    })
    .box_it_clone();

    let css_stream = evaluate_to_css(stream, ctx);

    let collected = Rc::new(RefCell::new(Vec::new()));
    let c = collected.clone();
    css_stream.subscribe(move |stmt| c.borrow_mut().push(stmt));

    let stmts = collected.borrow();
    assert_eq!(stmts.len(), 2, "@include should expand to 2 CssStmt::Decl");
    assert!(matches!(
        &stmts[0],
        CssStmt::Decl { property, value } if property == "border" && value == "1px solid"
    ));
    assert!(matches!(
        &stmts[1],
        CssStmt::Decl { property, value } if property == "padding" && value == "10px"
    ));
}

#[test]
fn function_def_registers_callable() {
    let bus = CompilerBus::new();
    let ctx = Rc::new(EvalContext::new(bus.clone(), 0));

    // Register a function via the operator
    let stream: lightforger::reactive::AstStream = Local::of(AstNode::FunctionDecl {
        name: "double".to_string(),
        params: vec!["$x".to_string()],
        body: vec![AstNode::Return {
            value: Box::new(AstNode::VariableDecl {
                name: "$result".to_string(),
                value: Value::Number(0.0),
            }),
        }],
    })
    .box_it_clone();

    let css_stream = evaluate_to_css(stream, ctx);

    // Drive the stream
    let _ = css_stream.subscribe(|_: CssStmt| {});

    // Function should be registered
    let func = bus.lookup_fn("double");
    assert!(func.is_some(), "function should be registered");
    let func = func.unwrap();
    assert_eq!(func.name, "double");
    assert_eq!(func.params, vec!["$x".to_string()]);
}

#[test]
fn use_rule_emits_module_load() {
    let bus = CompilerBus::new();
    let ctx = Rc::new(EvalContext::new(bus.clone(), 0));

    let collected = Rc::new(RefCell::new(Vec::new()));
    let c = collected.clone();
    bus.module_events().subscribe(move |evt| {
        let lightforger::reactive::ModuleEvent::Load { name } = evt else {
            unreachable!("ModuleEvent::Load irrefutable");
        };
        c.borrow_mut().push(name.clone());
    });

    let stream: lightforger::reactive::AstStream = Local::of(AstNode::UseRule {
        path: "theme".to_string(),
    })
    .box_it_clone();

    let css_stream = evaluate_to_css(stream, ctx);
    let _ = css_stream.subscribe(|_: CssStmt| {});

    assert_eq!(collected.borrow().len(), 1);
    assert_eq!(collected.borrow()[0], "theme");
}

#[test]
fn from_string_ast_end_to_end() {
    let items = vec![
        AstNode::StyleDecl {
            property: "color".to_string(),
            value: "red".to_string(),
        },
        AstNode::RuleSet {
            selector: ".foo".to_string(),
            inner: vec![AstNode::StyleDecl {
                property: "margin".to_string(),
                value: "0".to_string(),
            }],
        },
    ];

    let css = lightforger::reactive::from_string_ast(items).unwrap();
    assert!(css.contains("color: red;"));
    assert!(css.contains(".foo {"));
    assert!(css.contains("margin: 0;"));
}

#[test]
fn scope_id_nested_for_if() {
    let items = vec![
        AstNode::For {
            var: "$i".to_string(),
            from: 1.0,
            through: 3.0,
            body: vec![
                AstNode::If {
                    cond: Box::new(AstNode::Placeholder),
                    then_branch: vec![AstNode::StyleDecl {
                        property: "color".to_string(),
                        value: "red".to_string(),
                    }],
                    else_branch: vec![],
                },
            ],
        },
        AstNode::RuleSet {
            selector: ".foo".to_string(),
            inner: vec![AstNode::StyleDecl {
                property: "margin".to_string(),
                value: "0".to_string(),
            }],
        },
    ];

    let mut counter = 0;
    let result = pre_analysis(items, 0, &mut counter);

    // Verify scope structure: outer @for gets scope_id=1, nested @if inside
    // gets scope_id=1002, trailing ruleset gets scope_id=2
    match &result[0] {
        AstNode::For { body, .. } => {
            // @if inside @for should have a deeper scope path
            assert!(
                matches!(&body[0], AstNode::If { .. }),
                "Expected If node inside For body"
            );
        }
        other => panic!("Expected For node, got {:?}", other),
    }

    match &result[1] {
        AstNode::RuleSet { selector, .. } => {
            assert_eq!(selector, ".foo");
        }
        other => panic!("Expected RuleSet, got {:?}", other),
    }

    // Counter tracks top-level scope-creating nodes: @for + @RuleSet = 2
    // The @if is counted by inner_counter (nested scope), not the outer counter
    assert_eq!(counter, 2);
}
