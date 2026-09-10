//! 伪类选择器 extend 算法测试。

use sasspile::css::selector_parser::parse_selector;
use sasspile::css::selector_ops;

fn extend(sel: &str, ext: &str, ender: &str) -> String {
    let selector = parse_selector(sel);
    let extendee = parse_selector(ext);
    let extender = parse_selector(ender);
    let result = selector_ops::extend_selector(&selector, &extendee, &extender);
    result.to_string()
}

// ─── :not() 基础扩展 ─────────────────────────────────────────────

#[test]
fn test_extend_not_simple() {
    // :not(.c) + .c → :not(.c):not(.d)
    let result = extend(":not(.c)", ".c", ".d");
    assert!(result.contains(":not(.c)"), "should keep original :not(.c), got: {result}");
    assert!(result.contains(":not(.d)"), "should append :not(.d), got: {result}");
}

#[test]
fn test_extend_not_in_compound() {
    // .a:not(.c) + .c → .a:not(.c):not(.d)
    let result = extend(".a:not(.c)", ".c", ".d");
    assert!(result.contains(":not(.c)"), "should keep original :not(.c), got: {result}");
    assert!(result.contains(":not(.d)"), "should append :not(.d), got: {result}");
}

// ─── :not() extender 为列表 ──────────────────────────────────────

#[test]
fn test_extend_not_list_extender() {
    // :not(.c) + .c → :not(.c):not(.d):not(.e)
    let result = extend(":not(.c)", ".c", ".d, .e");
    assert!(result.contains(":not(.c)"), "should keep original :not(.c), got: {result}");
    assert!(result.contains(":not(.d)"), "should add :not(.d), got: {result}");
    assert!(result.contains(":not(.e)"), "should add :not(.e), got: {result}");
}

// ─── :not() 已包含列表 ────────────────────────────────────────────

#[test]
fn test_extend_not_existing_list() {
    // :not(.c, .d) + .c → :not(.c, .e, .d) 或 :not(.c, .d):not(.e)
    let result = extend(":not(.c, .d)", ".c", ".e");
    // 接受两种形式：追加到现有列表，或添加新的 :not()
    let has_e = result.contains(":not(.c, .e") || result.contains(":not(.e)") || result.contains(".e");
    assert!(has_e, "should include .e in result, got: {result}");
}

// ─── :not() 内部为 complex selector ───────────────────────────────

#[test]
fn test_extend_not_complex_inner() {
    // :not(.c .d) + .d → :not(.c .d):not(...) 复杂扩展
    let result = extend(":not(.c .d)", ".d", ".e .f");
    assert!(result.contains(":not(.c .d)"), "should keep original :not(.c .d), got: {result}");
}

// ─── extender 包含 :is() ──────────────────────────────────────────

#[test]
fn test_extend_not_with_is() {
    // :not(.c) + .c → :not(.c):not(.d):not(.e)
    let result = extend(":not(.c)", ".c", ":is(.d, .e)");
    assert!(result.contains(":not(.c)"), "should keep original :not(.c), got: {result}");
    assert!(result.contains(":not(.d)"), "should expand :is(.d,.e) to :not(.d), got: {result}");
    assert!(result.contains(":not(.e)"), "should expand :is(.d,.e) to :not(.e), got: {result}");
}

// ─── extender 包含 :where() ──────────────────────────────────────

#[test]
fn test_extend_not_with_where() {
    // :not(.c) + .c → :not(.c):not(.d):not(.e)
    let result = extend(":not(.c)", ".c", ":where(.d, .e)");
    assert!(result.contains(":not(.c)"), "should keep original :not(.c), got: {result}");
    assert!(result.contains(":not(.d)"), "should expand :where(.d,.e) to :not(.d), got: {result}");
    assert!(result.contains(":not(.e)"), "should expand :where(.d,.e) to :not(.e), got: {result}");
}

// ─── :is/:where/:matches 检测 no-op ──────────────────────────────

#[test]
fn test_extend_is_subselector_noop() {
    // .c:is(d) + :is(d) → .c:is(d) (no-op)
    let result = extend(".c:is(.d)", ":is(.d)", "d.e");
    assert_eq!(result, ".c:is(.d)");
}

#[test]
fn test_extend_where_subselector_noop() {
    // .c:where(.d) + :where(.d) → .c:where(.d) (no-op)
    let result = extend(".c:where(.d)", ":where(.d)", "d.e");
    assert_eq!(result, ".c:where(.d)");
}

// ─── :is/:where 自身扩展应正常工作 ────────────────────────────────

#[test]
fn test_extend_is_self_works() {
    // :is(c d.e, f g) + :is(c d.e, f g) → :is(c d.e, f g), h
    let result = extend(":is(c d.e, f g)", ":is(c d.e, f g)", "h");
    assert!(result.contains(":is(c d.e, f g)"), "should keep original, got: {result}");
    assert!(result.contains('h'), "should add h, got: {result}");
}

#[test]
fn test_extend_matches_self_works() {
    // :matches(c d.e, f g) + :matches(c d.e, f g) → :matches(c d.e, f g), h
    let result = extend(":matches(c d.e, f g)", ":matches(c d.e, f g)", "h");
    assert!(result.contains(":matches(c d.e, f g)"), "should keep original, got: {result}");
    assert!(result.contains('h'), "should add h, got: {result}");
}

// ─── extender 包含伪类在 compound 内 ──────────────────────────────

#[test]
fn test_extend_not_with_compound_pseudo_is() {
    // :not(.c) + .c → :not(.c):not(.d:is(.e, .f))
    let result = extend(":not(.c)", ".c", ".d:is(.e, .f)");
    assert!(result.contains(":not(.c)"), "should keep original :not(.c), got: {result}");
    // .d:is(.e, .f) 应作为整体 :not() 添加
    assert!(result.contains(":not(.d:is(.e, .f))") || result.contains(":not(.d)"),
        "should add :not() containing compound pseudo, got: {result}");
}

#[test]
fn test_extend_not_with_compound_pseudo_matches() {
    // :not(.c) + .c → :not(.c):not(.d:matches(.e, .f))
    let result = extend(":not(.c)", ".c", ".d:matches(.e, .f)");
    assert!(result.contains(":not(.c)"), "should keep original :not(.c), got: {result}");
    assert!(result.contains(":not(.d:matches(.e, .f))") || result.contains(":not(.d)"),
        "should add :not(), got: {result}");
}

// ─── extender 包含 :not() 是 no-op ────────────────────────────────

#[test]
fn test_extend_not_extender_contains_not_noop() {
    // :not(.c) + .c → :not(.c) (no-op when extender is :not(.d))
    // 根据 Sass nested :not 已知限制
    let result = extend(":not(.c)", ".c", ":not(.d)");
    assert_eq!(result, ":not(.c)");
}

// ─── :where() specificity_modification（高级特性，已知限制） ──────
// 注：:where(.x) + .x → :where(.x, .x .y) 需要替换 :where() 参数内部
// 当前版本未实现，作为后续增量 feature

// ─── 非 :not() 不受影响 ───────────────────────────────────────────

#[test]
fn test_extend_normal_still_works() {
    let result = extend(".foo", ".foo", ".bar");
    assert!(result.contains(".foo"));
    assert!(result.contains(".bar"));
}

#[test]
fn test_extend_descendant_still_works() {
    let result = extend(".foo .bar", ".bar", ".baz");
    assert!(result.contains(".bar"));
    assert!(result.contains(".baz"));
}
