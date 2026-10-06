//! End-to-end tests using Bootstrap component patterns.
//!
//! These tests validate that the reactive compiler can handle real-world
//! SCSS patterns extracted from Bootstrap's source (buttons, grid, variables).

use lightforger::reactive::{
    from_string_ast, pre_analysis, AstNode, CompilerBus, EvalContext, Value,
};
use std::sync::{Arc, Mutex};

/// Build an AST mirroring Bootstrap's `.btn` base styles (subset).
/// Reference: bootstrap/scss/_buttons.scss lines 5-42
fn btn_base_ast() -> Vec<AstNode> {
    vec![
        AstNode::RuleSet {
            selector: ".btn".to_string(),
            inner: vec![
                AstNode::StyleDecl {
                    property: "display".to_string(),
                    value: "inline-block".to_string(),
                },
                AstNode::StyleDecl {
                    property: "padding".to_string(),
                    value: "0.375rem 0.75rem".to_string(),
                },
                AstNode::StyleDecl {
                    property: "font-size".to_string(),
                    value: "1rem".to_string(),
                },
                AstNode::StyleDecl {
                    property: "font-weight".to_string(),
                    value: "400".to_string(),
                },
                AstNode::StyleDecl {
                    property: "line-height".to_string(),
                    value: "1.5".to_string(),
                },
                AstNode::StyleDecl {
                    property: "color".to_string(),
                    value: "#212529".to_string(),
                },
                AstNode::StyleDecl {
                    property: "text-align".to_string(),
                    value: "center".to_string(),
                },
                AstNode::StyleDecl {
                    property: "text-decoration".to_string(),
                    value: "none".to_string(),
                },
                AstNode::StyleDecl {
                    property: "vertical-align".to_string(),
                    value: "middle".to_string(),
                },
                AstNode::StyleDecl {
                    property: "cursor".to_string(),
                    value: "pointer".to_string(),
                },
                AstNode::StyleDecl {
                    property: "user-select".to_string(),
                    value: "none".to_string(),
                },
                AstNode::StyleDecl {
                    property: "border".to_string(),
                    value: "1px solid transparent".to_string(),
                },
                AstNode::StyleDecl {
                    property: "border-radius".to_string(),
                    value: "0.375rem".to_string(),
                },
            ],
        },
        AstNode::RuleSet {
            selector: ".btn:hover".to_string(),
            inner: vec![
                AstNode::StyleDecl {
                    property: "color".to_string(),
                    value: "#212529".to_string(),
                },
                AstNode::StyleDecl {
                    property: "text-decoration".to_string(),
                    value: "none".to_string(),
                },
            ],
        },
    ]
}

/// Build an AST mirroring Bootstrap's responsive breakpoint pattern.
/// Reference: bootstrap/scss/_grid.scss + mixins/_breakpoints.scss
fn responsive_grid_ast() -> Vec<AstNode> {
    vec![
        AstNode::RuleSet {
            selector: ".container".to_string(),
            inner: vec![AstNode::StyleDecl {
                property: "width".to_string(),
                value: "100%".to_string(),
            }],
        },
        AstNode::Media {
            query: "(min-width: 576px)".to_string(),
            inner: vec![AstNode::RuleSet {
                selector: ".container".to_string(),
                inner: vec![AstNode::StyleDecl {
                    property: "max-width".to_string(),
                    value: "540px".to_string(),
                }],
            }],
        },
        AstNode::Media {
            query: "(min-width: 768px)".to_string(),
            inner: vec![AstNode::RuleSet {
                selector: ".container".to_string(),
                inner: vec![AstNode::StyleDecl {
                    property: "max-width".to_string(),
                    value: "720px".to_string(),
                }],
            }],
        },
        AstNode::Media {
            query: "(min-width: 992px)".to_string(),
            inner: vec![AstNode::RuleSet {
                selector: ".container".to_string(),
                inner: vec![AstNode::StyleDecl {
                    property: "max-width".to_string(),
                    value: "960px".to_string(),
                }],
            }],
        },
        AstNode::Media {
            query: "(min-width: 1200px)".to_string(),
            inner: vec![AstNode::RuleSet {
                selector: ".container".to_string(),
                inner: vec![AstNode::StyleDecl {
                    property: "max-width".to_string(),
                    value: "1140px".to_string(),
                }],
            }],
        },
    ]
}

/// Build an AST mirroring Bootstrap's color variables + utility classes.
/// Reference: bootstrap/scss/_variables.scss + _utilities.scss
fn color_utilities_ast() -> Vec<AstNode> {
    vec![
        AstNode::VariableDecl {
            name: "$primary".to_string(),
            value: Value::String("#0d6efd".to_string()),
        },
        AstNode::VariableDecl {
            name: "$secondary".to_string(),
            value: Value::String("#6c757d".to_string()),
        },
        AstNode::VariableDecl {
            name: "$success".to_string(),
            value: Value::String("#198754".to_string()),
        },
        AstNode::RuleSet {
            selector: ".text-primary".to_string(),
            inner: vec![AstNode::StyleDecl {
                property: "color".to_string(),
                value: "#0d6efd".to_string(),
            }],
        },
        AstNode::RuleSet {
            selector: ".text-secondary".to_string(),
            inner: vec![AstNode::StyleDecl {
                property: "color".to_string(),
                value: "#6c757d".to_string(),
            }],
        },
        AstNode::RuleSet {
            selector: ".bg-success".to_string(),
            inner: vec![AstNode::StyleDecl {
                property: "background-color".to_string(),
                value: "#198754".to_string(),
            }],
        },
    ]
}

/// Build an AST mirroring Bootstrap's button variant mixin expansion.
/// Reference: bootstrap/scss/mixins/_buttons.scss — button-variant()
fn button_variant_ast() -> Vec<AstNode> {
    vec![
        AstNode::RuleSet {
            selector: ".btn-primary".to_string(),
            inner: vec![
                AstNode::StyleDecl {
                    property: "color".to_string(),
                    value: "#fff".to_string(),
                },
                AstNode::StyleDecl {
                    property: "background-color".to_string(),
                    value: "#0d6efd".to_string(),
                },
                AstNode::StyleDecl {
                    property: "border-color".to_string(),
                    value: "#0d6efd".to_string(),
                },
            ],
        },
        AstNode::RuleSet {
            selector: ".btn-primary:hover".to_string(),
            inner: vec![
                AstNode::StyleDecl {
                    property: "color".to_string(),
                    value: "#fff".to_string(),
                },
                AstNode::StyleDecl {
                    property: "background-color".to_string(),
                    value: "#0b5ed7".to_string(),
                },
                AstNode::StyleDecl {
                    property: "border-color".to_string(),
                    value: "#0a58ca".to_string(),
                },
            ],
        },
        AstNode::Media {
            query: "(prefers-reduced-motion: reduce)".to_string(),
            inner: vec![AstNode::RuleSet {
                selector: ".btn-primary".to_string(),
                inner: vec![AstNode::StyleDecl {
                    property: "transition".to_string(),
                    value: "none".to_string(),
                }],
            }],
        },
    ]
}

#[test]
fn btn_base_compiles_all_declarations() {
    let items = btn_base_ast();
    let css = from_string_ast(items).unwrap();

    assert!(css.contains(".btn {"), "should contain .btn rule");
    assert!(css.contains("display: inline-block;"));
    assert!(css.contains("padding: 0.375rem 0.75rem;"));
    assert!(css.contains("font-size: 1rem;"));
    assert!(css.contains("font-weight: 400;"));
    assert!(css.contains("line-height: 1.5;"));
    assert!(css.contains("color: #212529;"));
    assert!(css.contains("text-align: center;"));
    assert!(css.contains("text-decoration: none;"));
    assert!(css.contains("vertical-align: middle;"));
    assert!(css.contains("cursor: pointer;"));
    assert!(css.contains("user-select: none;"));
    assert!(css.contains("border: 1px solid transparent;"));
    assert!(css.contains("border-radius: 0.375rem;"));
    assert!(css.contains(".btn:hover {"), "should contain hover rule");
}

#[test]
fn btn_base_nested_ruleset_structure() {
    let items = btn_base_ast();
    let mut counter = 0;
    let analyzed = pre_analysis(items, 0, &mut counter);

    match &analyzed[0] {
        AstNode::RuleSet { selector, inner } => {
            assert_eq!(selector, ".btn");
            assert_eq!(inner.len(), 13, "should have 13 declarations in .btn");
        }
        other => panic!("Expected RuleSet, got {:?}", other),
    }

    match &analyzed[1] {
        AstNode::RuleSet { selector, inner } => {
            assert_eq!(selector, ".btn:hover");
            assert_eq!(inner.len(), 2, "should have 2 declarations in .btn:hover");
        }
        other => panic!("Expected RuleSet, got {:?}", other),
    }
}

#[test]
fn responsive_grid_media_queries() {
    let items = responsive_grid_ast();
    let css = from_string_ast(items).unwrap();

    assert!(css.contains(".container {"));
    assert!(css.contains("width: 100%;"));
    assert!(css.contains("@media (min-width: 576px)"));
    assert!(css.contains("max-width: 540px;"));
    assert!(css.contains("@media (min-width: 768px)"));
    assert!(css.contains("max-width: 720px;"));
    assert!(css.contains("@media (min-width: 992px)"));
    assert!(css.contains("max-width: 960px;"));
    assert!(css.contains("@media (min-width: 1200px)"));
    assert!(css.contains("max-width: 1140px;"));
}

#[test]
fn responsive_grid_media_structure() {
    let items = responsive_grid_ast();
    let mut counter = 0;
    let analyzed = pre_analysis(items, 0, &mut counter);

    assert!(matches!(
        &analyzed[0],
        AstNode::RuleSet { selector, .. } if selector == ".container"
    ));

    for i in 1..=4 {
        assert!(
            matches!(&analyzed[i], AstNode::Media { .. }),
            "Expected Media at index {}",
            i
        );
    }
}

#[test]
fn color_utilities_classes() {
    let items = color_utilities_ast();
    let css = from_string_ast(items).unwrap();

    assert!(css.contains(".text-primary {"));
    assert!(css.contains("color: #0d6efd;"));
    assert!(css.contains(".text-secondary {"));
    assert!(css.contains("color: #6c757d;"));
    assert!(css.contains(".bg-success {"));
    assert!(css.contains("background-color: #198754;"));
}

#[test]
fn color_utilities_variable_registration() {
    use lightforger::reactive::evaluate_to_css;
    use rxrust::prelude::*;

    let bus = CompilerBus::new();
    let ctx = Arc::new(EvalContext::new(bus.clone(), 0));

    let collected = Arc::new(Mutex::new(Vec::new()));
    let c = collected.clone();
    bus.var_events().subscribe(move |evt| {
        if let lightforger::reactive::ValueEvent::Bind {
            scope_id,
            name,
            value,
        } = evt
        {
            c.lock().unwrap().push((scope_id, name, value));
        }
    });

    // Use compile_ast directly to leverage the externally-provided bus
    let items = color_utilities_ast();
    let mut counter = 0;
    let analyzed = pre_analysis(items, 0, &mut counter);

    let stream: lightforger::reactive::AstStream =
        Shared::from_iter(analyzed).box_it();
    let css_stream = evaluate_to_css(stream, ctx);

    // Drive the stream to completion
    let _ = css_stream.subscribe(|_| {});

    let vars = collected.lock().unwrap();
    assert!(
        vars.len() >= 3,
        "Should register at least 3 color variables, got {}",
        vars.len()
    );

    let names: Vec<&str> = vars.iter().map(|(_, name, _)| name.as_str()).collect();
    assert!(names.contains(&"$primary"));
    assert!(names.contains(&"$secondary"));
    assert!(names.contains(&"$success"));
}

#[test]
fn button_variant_includes_media_query() {
    let items = button_variant_ast();
    let css = from_string_ast(items).unwrap();

    assert!(css.contains(".btn-primary {"));
    assert!(css.contains("color: #fff;"));
    assert!(css.contains("background-color: #0d6efd;"));
    assert!(css.contains("border-color: #0d6efd;"));
    assert!(css.contains(".btn-primary:hover {"));
    assert!(css.contains("background-color: #0b5ed7;"));
    assert!(css.contains("border-color: #0a58ca;"));
    assert!(css.contains("@media (prefers-reduced-motion: reduce)"));
    assert!(css.contains("transition: none;"));
}

#[test]
fn button_variant_nested_media_in_rule() {
    let items = button_variant_ast();
    let mut counter = 0;
    let analyzed = pre_analysis(items, 0, &mut counter);

    assert!(matches!(
        &analyzed[0],
        AstNode::RuleSet { selector, .. } if selector == ".btn-primary"
    ));
    assert!(matches!(
        &analyzed[1],
        AstNode::RuleSet { selector, .. } if selector == ".btn-primary:hover"
    ));
    match &analyzed[2] {
        AstNode::Media { query, inner } => {
            assert_eq!(query, "(prefers-reduced-motion: reduce)");
            assert_eq!(inner.len(), 1);
        }
        other => panic!("Expected Media, got {:?}", other),
    }
}

#[test]
fn bootstrap_end_to_end_combined() {
    let mut all_items = Vec::new();
    all_items.extend(btn_base_ast());
    all_items.extend(responsive_grid_ast());
    all_items.extend(color_utilities_ast());
    all_items.extend(button_variant_ast());

    let css = from_string_ast(all_items).unwrap();

    assert!(css.contains(".btn {"));
    assert!(css.contains(".btn-primary {"));
    assert!(css.contains(".container {"));
    assert!(css.contains(".text-primary {"));
    assert!(css.contains(".text-secondary {"));
    assert!(css.contains(".bg-success {"));
    assert!(css.contains("(min-width: 576px)"));
    assert!(css.contains("(min-width: 768px)"));
    assert!(css.contains("(min-width: 992px)"));
    assert!(css.contains("(min-width: 1200px)"));
    assert!(css.contains("(prefers-reduced-motion: reduce)"));
    assert!(
        css.len() > 500,
        "Combined CSS output should be substantial, got {} bytes",
        css.len()
    );
}
