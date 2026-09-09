//! Reactor 管线集成测试。
//!
//! 验证消费-返回 API + 类型状态机 + OTel 链式追踪。

use std::path::PathBuf;
use std::sync::Arc;
use std::collections::HashMap;

use sasspile::eval::reactor::{
    Reactor, CompileStage, MockReactorIO,
};
use sasspile::OutputStyle;

// ═══════════════════════════════════════════════════════════════════════════
// 基础管线测试
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_reactor_basic_pipeline() {
    let css = Reactor::new("a { color: red; }")
        .lex()
        .expect("unexpected failure in test")
        .parse()
        .expect("unexpected failure in test")
        .evaluate()
        .expect("unexpected failure in test")
        .serialize(OutputStyle::Expanded)
        .finish()
        .expect("unexpected failure in test");

    assert_eq!(css, "a {\n  color: red;\n}\n", "basic pipeline output mismatch");
}

#[test]
fn test_reactor_type_state_lex() {
    let reactor = Reactor::new("$x: 1; a { color: blue; }");
    let lexed = reactor.lex().expect("unexpected failure in test");
    assert_eq!(lexed.stage(), CompileStage::Lex);
}

#[test]
fn test_reactor_type_state_parse() {
    let reactor = Reactor::new("a { color: green; }");
    let parsed = reactor.lex().expect("unexpected failure in test").parse().expect("unexpected failure in test");
    assert_eq!(parsed.stage(), CompileStage::Parse);
}

#[test]
fn test_reactor_type_state_evaluate() {
    let reactor = Reactor::new("a { color: green; }");
    let evaluated = reactor.lex().expect("unexpected failure in test").parse().expect("unexpected failure in test").evaluate().expect("unexpected failure in test");
    assert_eq!(evaluated.stage(), CompileStage::Evaluate);
    assert!(!evaluated.css_nodes.is_empty());
}

#[test]
fn test_reactor_type_state_serialize() {
    let reactor = Reactor::new("a { color: green; }");
    let serialized = reactor
        .lex()
        .expect("unexpected failure in test")
        .parse()
        .expect("unexpected failure in test")
        .evaluate()
        .expect("unexpected failure in test")
        .serialize(OutputStyle::Expanded);
    assert_eq!(serialized.stage(), CompileStage::Serialize);
}

#[test]
fn test_reactor_compressed_output() {
    let css = Reactor::new("a { color: red; }")
        .lex()
        .expect("unexpected failure in test")
        .parse()
        .expect("unexpected failure in test")
        .evaluate()
        .expect("unexpected failure in test")
        .serialize(OutputStyle::Compressed)
        .finish()
        .expect("unexpected failure in test");

    assert_eq!(css, "a{color:red;}", "compressed output mismatch");
}

#[test]
fn test_reactor_variables() {
    let css = Reactor::new(
        r#"
        $primary: #3498db;
        $padding: 10px;
        .btn {
            background: $primary;
            padding: $padding;
        }
        "#,
    )
    .lex()
    .expect("unexpected failure in test")
    .parse()
    .expect("unexpected failure in test")
    .evaluate()
    .expect("unexpected failure in test")
    .serialize(OutputStyle::Expanded)
    .finish()
    .expect("unexpected failure in test");

    assert!(css.contains("#3498db"), "missing primary color: {css}");
    assert!(css.contains("10px"), "missing padding: {css}");
}

#[test]
fn test_reactor_nesting() {
    let css = Reactor::new(
        r#"
        .card {
            background: white;
            &:hover {
                background: gray;
            }
            .title {
                font-size: 18px;
            }
        }
        "#,
    )
    .lex()
    .expect("unexpected failure in test")
    .parse()
    .expect("unexpected failure in test")
    .evaluate()
    .expect("unexpected failure in test")
    .serialize(OutputStyle::Expanded)
    .finish()
    .expect("unexpected failure in test");

    assert!(css.contains(".card:hover"), "missing hover selector: {css}");
    assert!(css.contains(".card .title"), "missing nested title: {css}");
}

// ═══════════════════════════════════════════════════════════════════════════
// IO Mock 测试 (验证 ReactorIO trait 与 MockReactorIO)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_mocked_io_basic() {
    let mut files = HashMap::new();
    files.insert(PathBuf::from("main.scss"), "a { color: red; }".to_string());

    let css = Reactor::new("a { color: red; }")
        .with_io(Arc::new(MockReactorIO::new(files)))
        .lex()
        .expect("unexpected failure in test")
        .parse()
        .expect("unexpected failure in test")
        .evaluate()
        .expect("unexpected failure in test")
        .serialize(OutputStyle::Expanded)
        .finish()
        .expect("unexpected failure in test");

    assert!(css.contains("color: red"));
}

// ═══════════════════════════════════════════════════════════════════════════
// OTel 追踪测试 (链式追踪)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_trace_id_consistent_across_pipeline() {
    let reactor = Reactor::new("a { color: red; }");
    let trace_id = reactor.trace().trace_id;

    let lexed = reactor.lex().expect("unexpected failure in test");
    assert_eq!(lexed.trace().trace_id, trace_id, "lex stage changed trace_id");

    let parsed = lexed.parse().expect("unexpected failure in test");
    assert_eq!(parsed.trace().trace_id, trace_id, "parse stage changed trace_id");

    let evaluated = parsed.evaluate().expect("unexpected failure in test");
    assert_eq!(
        evaluated.trace().trace_id,
        trace_id,
        "evaluate stage changed trace_id"
    );

    let serialized = evaluated.serialize(OutputStyle::Expanded);
    assert_eq!(
        serialized.trace().trace_id,
        trace_id,
        "serialize stage changed trace_id"
    );
}

#[test]
fn test_snapshot_at_each_stage() {
    let reactor = Reactor::new("a { color: blue; }");
    let snap = reactor.snapshot();
    assert_eq!(snap.stage, CompileStage::Raw);
    assert_eq!(snap.n_css_nodes, 0);

    let lexed = reactor.lex().expect("unexpected failure in test");
    let snap = lexed.snapshot();
    assert_eq!(snap.stage, CompileStage::Lex);
    assert_eq!(snap.n_css_nodes, 0);

    let parsed = lexed.parse().expect("unexpected failure in test");
    let snap = parsed.snapshot();
    assert_eq!(snap.stage, CompileStage::Parse);
    assert_eq!(snap.n_css_nodes, 0);

    let evaluated = parsed.evaluate().expect("unexpected failure in test");
    let snap = evaluated.snapshot();
    assert_eq!(snap.stage, CompileStage::Evaluate);
    assert!(snap.n_css_nodes > 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// 边界情况测试
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_empty_source() {
    let css = Reactor::new("")
        .lex()
        .expect("unexpected failure in test")
        .parse()
        .expect("unexpected failure in test")
        .evaluate()
        .expect("unexpected failure in test")
        .serialize(OutputStyle::Expanded)
        .finish()
        .expect("unexpected failure in test");

    assert!(css.is_empty() || css.trim().is_empty(), "empty source should produce empty CSS");
}

#[test]
fn test_comment_only_source() {
    let css = Reactor::new("// just a comment\n")
        .lex()
        .expect("unexpected failure in test")
        .parse()
        .expect("unexpected failure in test")
        .evaluate()
        .expect("unexpected failure in test")
        .serialize(OutputStyle::Expanded)
        .finish()
        .expect("unexpected failure in test");

    // Silent comments produce no CSS
    assert!(css.trim().is_empty(), "silent comment-only source should produce empty CSS");
}

#[test]
fn test_deep_nesting() {
    let mut scss = String::new();
    for _ in 0..10 {
        scss.push_str(".level { ");
    }
    scss.push_str("color: red; ");
    for _ in 0..10 {
        scss.push_str("} ");
    }

    let css = Reactor::new(scss)
        .lex()
        .expect("unexpected failure in test")
        .parse()
        .expect("unexpected failure in test")
        .evaluate()
        .expect("unexpected failure in test")
        .serialize(OutputStyle::Expanded)
        .finish()
        .expect("unexpected failure in test");

    assert!(css.contains("color: red"), "deep nesting should still produce CSS");
}
