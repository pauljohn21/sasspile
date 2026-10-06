//! Lowering 单元测试 — 验证 SassAstNode → AstNode 降级转换

use lightforger::lowering::{lower_to_ast, LoweringContext};
use lightforger::parser::SassAstNode;
use lightforger::reactive::{AstNode, CssStmt, Value};

// ── Task 4.2: 简单样式声明降级 ──

#[test]
fn test_lower_style_decl() {
    let node = SassAstNode::StyleDecl {
        prop: "color".to_string(),
        value: "red".to_string(),
    };
    let mut ctx = LoweringContext::new();
    let result = lower_to_ast(node, &mut ctx).unwrap();

    match result {
        AstNode::StyleDecl { property, value } => {
            assert_eq!(property, "color");
            assert_eq!(value, "red");
        }
        other => panic!("expected StyleDecl, got {:?}", other),
    }
}

// ── Task 4.3: 插值展开 ──

#[test]
fn test_lower_variable_interpolation() {
    let node = SassAstNode::Interpolated("$prefix".to_string());
    let mut ctx = LoweringContext::new();
    ctx.set_variable("$prefix", "bs-");

    let result = lower_to_ast(node, &mut ctx).unwrap();
    // 插值展开后应为 CssStmt::Decl 包装的解析值
    match result {
        AstNode::Css(CssStmt::Decl { value, .. }) => {
            assert!(value.contains("bs-"), "value was: {value}");
        }
        other => panic!("expected Css Decl, got {:?}", other),
    }
}

#[test]
fn test_lower_complex_interpolation() {
    let node = SassAstNode::Interpolated("#{$prefix}btn-color".to_string());
    let mut ctx = LoweringContext::new();
    ctx.set_variable("$prefix", "bs-");

    let result = lower_to_ast(node, &mut ctx).unwrap();
    match result {
        AstNode::Css(CssStmt::Decl { value, .. }) => {
            assert!(value.contains("bs-btn-color"), "value was: {value}");
        }
        other => panic!("expected Css Decl, got {:?}", other),
    }
}

#[test]
fn test_lower_undefined_variable_error() {
    let node = SassAstNode::Interpolated("$undefined".to_string());
    let mut ctx = LoweringContext::new();

    let result = lower_to_ast(node, &mut ctx);
    assert!(result.is_err(), "undefined variable should produce an error");
    let err = result.unwrap_err();
    let err_msg = format!("{err}");
    assert!(
        err_msg.contains("undefined") || err_msg.contains("$undefined"),
        "error should mention undefined variable, was: {err_msg}"
    );
}

// ── Task 4.4: 父选择器展开 ──

#[test]
fn test_lower_parent_selector_single_nesting() {
    let node = SassAstNode::Rule {
        selector: "&:hover".to_string(),
        inner: vec![],
    };
    let mut ctx = LoweringContext::new();
    ctx.push_selector(".btn".to_string());

    let result = lower_to_ast(node, &mut ctx).unwrap();

    match result {
        AstNode::RuleSet { selector, .. } => {
            assert_eq!(selector, ".btn:hover");
        }
        other => panic!("expected RuleSet, got {:?}", other),
    }
}

#[test]
fn test_lower_parent_selector_multi_nesting() {
    let node = SassAstNode::Rule {
        selector: "&:focus".to_string(),
        inner: vec![],
    };
    let mut ctx = LoweringContext::new();
    ctx.push_selector(".card".to_string());
    ctx.push_selector(".body".to_string());

    let result = lower_to_ast(node, &mut ctx).unwrap();

    match result {
        AstNode::RuleSet { selector, .. } => {
            assert_eq!(selector, ".card .body:focus");
        }
        other => panic!("expected RuleSet, got {:?}", other),
    }
}

// ── Task 4.5: Map/List 字面量 ──

#[test]
fn test_lower_value_node_number() {
    let node = SassAstNode::Raw("42".to_string());
    let mut ctx = LoweringContext::new();

    let _ = lower_to_ast(node, &mut ctx);
    // just verify it doesn't panic
}

// ── Task 4.6: !default 语义 ──

#[test]
fn test_lower_default_overridden() {
    // 环境中已有 $color: blue，遇到 $color: red !default 时丢弃
    let node = SassAstNode::VariableDecl {
        name: "$color".to_string(),
        value: Box::new(SassAstNode::Raw("red".to_string())),
        has_default: true,
    };
    let mut ctx = LoweringContext::new();
    ctx.set_variable("$color", "blue");

    let result = lower_to_ast(node, &mut ctx).unwrap();
    assert!(
        matches!(result, AstNode::Placeholder),
        "default should be discarded when variable exists"
    );
}

#[test]
fn test_lower_default_applied() {
    // 环境中不存在 $color，遇到 $color: red !default 时生效
    let node = SassAstNode::VariableDecl {
        name: "$color".to_string(),
        value: Box::new(SassAstNode::Raw("red".to_string())),
        has_default: true,
    };
    let mut ctx = LoweringContext::new();

    let result = lower_to_ast(node, &mut ctx).unwrap();
    match &result {
        AstNode::VariableDecl { name, .. } => {
            assert_eq!(name, "$color");
        }
        other => panic!("expected VariableDecl, got {:?}", other),
    }
    // 验证变量已被记录到环境
    assert_eq!(ctx.get_variable("$color"), Some("red"));
}

#[test]
fn test_lower_variable_decl_no_default() {
    // 非 !default 变量：正常产出 Bind 事件
    let node = SassAstNode::VariableDecl {
        name: "$size".to_string(),
        value: Box::new(SassAstNode::Raw("16px".to_string())),
        has_default: false,
    };
    let mut ctx = LoweringContext::new();

    let result = lower_to_ast(node, &mut ctx).unwrap();
    match &result {
        AstNode::VariableDecl { name, value } => {
            assert_eq!(name, "$size");
            assert!(matches!(value, Value::String(s) if s == "16px"));
        }
        other => panic!("expected VariableDecl, got {:?}", other),
    }
}

// ── Task 4.7: 错误传播 ──

#[test]
fn test_lower_error_propagation() {
    let node = SassAstNode::Interpolated("$nonexistent".to_string());
    let mut ctx = LoweringContext::new();

    let result = lower_to_ast(node, &mut ctx);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(format!("{err}").contains("undefined") || format!("{err}").contains("$nonexistent"));
}

// ── 原始文本透传测试 ──

#[test]
fn test_lower_raw_text() {
    let node = SassAstNode::Raw("color: red".to_string());
    let mut ctx = LoweringContext::new();

    let result = lower_to_ast(node, &mut ctx).unwrap();
    assert!(matches!(result, AstNode::Css(CssStmt::Decl { .. })));
}

#[test]
fn test_lower_empty_raw_is_placeholder() {
    let node = SassAstNode::Raw("".to_string());
    let mut ctx = LoweringContext::new();

    let result = lower_to_ast(node, &mut ctx).unwrap();
    assert!(matches!(result, AstNode::Placeholder));
}

// ── Comment 透传测试 ──

#[test]
fn test_lower_comment_is_placeholder() {
    let node = SassAstNode::Comment("/* block comment */".to_string());
    let mut ctx = LoweringContext::new();

    let result = lower_to_ast(node, &mut ctx).unwrap();
    assert!(matches!(result, AstNode::Placeholder));
}

// ── Context 功能测试 ──

#[test]
fn test_context_ancestor_path() {
    let mut ctx = LoweringContext::new();
    ctx.push_selector(".card".to_string());
    ctx.push_selector(".body".to_string());
    assert_eq!(ctx.ancestor_path(), ".card .body");
}

#[test]
fn test_context_variable_operations() {
    let mut ctx = LoweringContext::new();
    assert!(!ctx.has_variable("$x"));
    ctx.set_variable("$x", "42");
    assert!(ctx.has_variable("$x"));
    assert_eq!(ctx.get_variable("$x"), Some("42"));
}
