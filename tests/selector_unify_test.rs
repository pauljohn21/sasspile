//! 选择器 unify 算法测试。

use sasspile::css::selector_parser::parse_selector;
use sasspile::css::selector_ops;
use sasspile::Reactor;

#[test]
fn test_selector_append_basic() {
    let input = "\
        @use 'sass:selector';\
        .a { x: selector.append(\".b\", \".c\"); }\
    ";
    let result = Reactor::new(input).lex().unwrap().parse().unwrap().evaluate().unwrap().serialize(sasspile::OutputStyle::Expanded).finish().unwrap();
    assert!(result.contains(".b.c"), "Expected .b.c, got {result}");
}

#[test]
fn test_selector_append_second_arg_leading_combinator_error() {
    let input = "\
        @use 'sass:selector';\
        .a { x: selector.append(\".b\", \"> .c\"); }\
    ";
    let eval_result = Reactor::new(input).lex().unwrap().parse().unwrap().evaluate();
    assert!(eval_result.is_err(), "Expected error for second arg starting with combinator");
}

#[test]
fn test_selector_append_first_arg_leading_combinator_ok() {
    let input = "\
        @use 'sass:selector';\
        .a { x: selector.append(\"> .b\", \".c\"); }\
    ";
    let result = Reactor::new(input).lex().unwrap().parse().unwrap().evaluate().unwrap().serialize(sasspile::OutputStyle::Expanded).finish().unwrap();
    assert!(result.contains("> .b.c"), "Expected > .b.c, got {result}");
}

#[test]
fn test_global_selector_append() {
    let input = "\
        .a { x: selector-append(\".b\", \".c\"); }\
    ";
    let result = Reactor::new(input).lex().unwrap().parse().unwrap().evaluate().unwrap().serialize(sasspile::OutputStyle::Expanded).finish().unwrap();
    assert!(result.contains(".b.c"), "Expected .b.c, got {result}");
}

#[test]
fn test_selector_append_with_inner_combinator() {
    let input = r#"
        @use 'sass:selector';
        .a { x: selector.append(".b > .c", ".d"); }
    "#;
    let result = Reactor::new(input).lex().unwrap().parse().unwrap().evaluate().unwrap().serialize(sasspile::OutputStyle::Expanded).finish().unwrap();
    assert!(result.contains(".b > .c.d"), "Expected .b > .c.d, got {result}");
}

#[test]
fn test_selector_append_empty_args_error() {
    let input = r#"
        @use 'sass:selector';
        .a { x: selector.append(); }
    "#;
    let eval_result = Reactor::new(input).lex().unwrap().parse().unwrap().evaluate();
    assert!(eval_result.is_err(), "Expected error for empty args");
}

#[test]
fn test_selector_append_invalid_type_error() {
    let input = r#"
        @use 'sass:selector';
        .a { x: selector.append(1, 2); }
    "#;
    let eval_result = Reactor::new(input).lex().unwrap().parse().unwrap().evaluate();
    assert!(eval_result.is_err(), "Expected error for invalid type");
}

#[test]
fn test_selector_append_only_combinator_error() {
    let input = r#"
        @use 'sass:selector';
        .a { x: selector.append(".a", ">"); }
    "#;
    let eval_result = Reactor::new(input).lex().unwrap().parse().unwrap().evaluate();
    assert!(eval_result.is_err(), "Expected error for only combinator");
}

#[test]
fn test_selector_append_only_leading_combinator_error() {
    let input = r#"
        @use 'sass:selector';
        .a { x: selector.append(">", ".b"); }
    "#;
    let eval_result = Reactor::new(input).lex().unwrap().parse().unwrap().evaluate();
    assert!(eval_result.is_err(), "Expected error for leading only combinator");
}

#[test]
fn test_selector_append_trailing_combinator_error() {
    let input = r#"
        @use 'sass:selector';
        .a { x: selector.append(".a >", ".b"); }
    "#;
    let eval_result = Reactor::new(input).lex().unwrap().parse().unwrap().evaluate();
    assert!(eval_result.is_err(), "Expected error for trailing combinator");
}

#[test]
fn test_selector_nest_basic() {
    let input = r#"
        @use 'sass:selector';
        .a { x: selector.nest(".b", ".c"); }
    "#;
    let result = Reactor::new(input).lex().unwrap().parse().unwrap().evaluate().unwrap().serialize(sasspile::OutputStyle::Expanded).finish().unwrap();
    assert!(result.contains(".b .c"), "Expected .b .c, got {result}");
}

#[test]
fn test_selector_nest_with_parent_ref() {
    let input = r#"
        @use 'sass:selector';
        .a { x: selector.nest(".b", "&.c"); }
    "#;
    let result = Reactor::new(input).lex().unwrap().parse().unwrap().evaluate().unwrap().serialize(sasspile::OutputStyle::Expanded).finish().unwrap();
    assert!(result.contains(".b.c"), "Expected .b.c, got {result}");
}


#[test]
fn test_unify_same_class() {
    let a = parse_selector(".foo");
    let b = parse_selector(".foo");
    let result = selector_ops::unify(&a, &b);
    assert_eq!(result.map(|s| s.to_string()), Some(".foo".to_string()));
}

#[test]
fn test_unify_different_classes() {
    let a = parse_selector(".foo");
    let b = parse_selector(".bar");
    let result = selector_ops::unify(&a, &b);
    assert_eq!(result.map(|s| s.to_string()), Some(".foo.bar".to_string()));
}

#[test]
fn test_unify_type_conflict() {
    let a = parse_selector("div");
    let b = parse_selector("span");
    let result = selector_ops::unify(&a, &b);
    assert_eq!(result, None);
}

#[test]
fn test_unify_id_conflict() {
    let a = parse_selector("#main");
    let b = parse_selector("#other");
    let result = selector_ops::unify(&a, &b);
    assert_eq!(result, None);
}

#[test]
fn test_unify_universal_with_type() {
    let a = parse_selector("*");
    let b = parse_selector("div");
    let result = selector_ops::unify(&a, &b);
    assert_eq!(result.map(|s| s.to_string()), Some("div".to_string()));
}

#[test]
fn test_unify_descendant() {
    let a = parse_selector("a b");
    let b = parse_selector("c d");
    let result = selector_ops::unify(&a, &b);
    // a b + c d: 最后一个复合 b 和 d 类型冲突 → None
    assert_eq!(result, None);
}

#[test]
fn test_unify_descendant_same_type() {
    let a = parse_selector(".a b");
    let b = parse_selector(".c b");
    let result = selector_ops::unify(&a, &b);
    // .a b + .c b: 最后一个复合 b 和 b 统一 = b，前缀 .a + .c
    assert!(result.is_some());
    let s = result.unwrap().to_string();
    assert!(s.contains("b"));
}

#[test]
fn test_unify_class_with_id() {
    let a = parse_selector(".foo");
    let b = parse_selector("#bar");
    let result = selector_ops::unify(&a, &b);
    assert_eq!(result.map(|s| s.to_string()), Some("#bar.foo".to_string()));
}

#[test]
fn test_unify_comma_list() {
    let a = parse_selector(".a, .b");
    let b = parse_selector(".c");
    let result = selector_ops::unify(&a, &b);
    assert!(result.is_some());
    let s = result.unwrap().to_string();
    assert!(s.contains(".a.c") || s.contains(".c.a"));
    assert!(s.contains(".b.c") || s.contains(".c.b"));
}

// ─── selector-extend tests ───────────────────────────────────────

#[test]
fn test_extend_parent_replacement() {
    // selector.extend(".c.x .d", ".c", ".e") → ".c.x .d, .x.e .d"
    let input = r#"
        @use 'sass:selector';
        .a { x: selector.extend(".c.x .d", ".c", ".e"); }
    "#;
    let result = Reactor::new(input).lex().unwrap().parse().unwrap().evaluate().unwrap().serialize(sasspile::OutputStyle::Expanded).finish().unwrap();
    assert!(result.contains(".c.x .d"), "Expected original .c.x .d, got {result}");
    assert!(result.contains(".x.e .d"), "Expected extended .x.e .d, got {result}");
}

#[test]
fn test_extend_multi_compound_extender() {
    // selector.extend(".c.x .d", ".c", ".e .f") → ".c.x .d, .e .x.f .d"
    let input = r#"
        @use 'sass:selector';
        .a { x: selector.extend(".c.x .d", ".c", ".e .f"); }
    "#;
    let result = Reactor::new(input).lex().unwrap().parse().unwrap().evaluate().unwrap().serialize(sasspile::OutputStyle::Expanded).finish().unwrap();
    assert!(result.contains(".c.x .d"), "Expected original, got {result}");
    assert!(result.contains(".e .x.f .d"), "Expected .e .x.f .d, got {result}");
}

#[test]
fn test_extend_multi_extender_list() {
    // selector.extend(".c.x .d", ".c", ".e, .f") → ".c.x .d, .x.e .d, .x.f .d"
    let input = r#"
        @use 'sass:selector';
        .a { x: selector.extend(".c.x .d", ".c", ".e, .f"); }
    "#;
    let result = Reactor::new(input).lex().unwrap().parse().unwrap().evaluate().unwrap().serialize(sasspile::OutputStyle::Expanded).finish().unwrap();
    assert!(result.contains(".c.x .d"), "Expected original, got {result}");
    assert!(result.contains(".x.e .d"), "Expected .x.e .d, got {result}");
    assert!(result.contains(".x.f .d"), "Expected .x.f .d, got {result}");
}

#[test]
fn test_extend_grandparent_replacement() {
    // selector.extend(".c .d.x .e", ".d", ".f") → ".c .d.x .e, .c .x.f .e"
    let input = r#"
        @use 'sass:selector';
        .a { x: selector.extend(".c .d.x .e", ".d", ".f"); }
    "#;
    let result = Reactor::new(input).lex().unwrap().parse().unwrap().evaluate().unwrap().serialize(sasspile::OutputStyle::Expanded).finish().unwrap();
    assert!(result.contains(".c .d.x .e"), "Expected original, got {result}");
    assert!(result.contains(".c .x.f .e"), "Expected .c .x.f .e, got {result}");
}

#[test]
fn test_extend_grandparent_multi_compound_extender() {
    // selector.extend(".c .d.x .e", ".d", ".f .g") → ".c .d.x .e, .c .f .x.g .e, .f .c .x.g .e"
    let input = r#"
        @use 'sass:selector';
        .a { x: selector.extend(".c .d.x .e", ".d", ".f .g"); }
    "#;
    let result = Reactor::new(input).lex().unwrap().parse().unwrap().evaluate().unwrap().serialize(sasspile::OutputStyle::Expanded).finish().unwrap();
    assert!(result.contains(".c .d.x .e"), "Expected original, got {result}");
    assert!(result.contains(".c .f .x.g .e"), "Expected .c .f .x.g .e, got {result}");
}

#[test]
fn test_extend_grandparent_list_extender() {
    // selector.extend(".c .d.x .e", ".d", ".f, .g") → ".c .d.x .e, .c .x.f .e, .c .x.g .e"
    let input = r#"
        @use 'sass:selector';
        .a { x: selector.extend(".c .d.x .e", ".d", ".f, .g"); }
    "#;
    let result = Reactor::new(input).lex().unwrap().parse().unwrap().evaluate().unwrap().serialize(sasspile::OutputStyle::Expanded).finish().unwrap();
    assert!(result.contains(".c .d.x .e"), "Expected original, got {result}");
    assert!(result.contains(".c .x.f .e"), "Expected .c .x.f .e, got {result}");
    assert!(result.contains(".c .x.g .e"), "Expected .c .x.g .e, got {result}");
}

#[test]
fn test_extend_leading_child_combinator() {
    // selector.extend(".c .d", ".d", "> .e") → ".c .d, .c > .e"
    let input = r#"
        @use 'sass:selector';
        .a { x: selector.extend(".c .d", ".d", "> .e"); }
    "#;
    let result = Reactor::new(input).lex().unwrap().parse().unwrap().evaluate().unwrap().serialize(sasspile::OutputStyle::Expanded).finish().unwrap();
    assert!(result.contains(".c .d"), "Expected original, got {result}");
    assert!(result.contains(".c > .e"), "Expected .c > .e, got {result}");
}

#[test]
fn test_extend_leading_adjacent_combinator() {
    // selector.extend(".c .d", ".d", "+ .e") → ".c .d, .c + .e"
    let input = r#"
        @use 'sass:selector';
        .a { x: selector.extend(".c .d", ".d", "+ .e"); }
    "#;
    let result = Reactor::new(input).lex().unwrap().parse().unwrap().evaluate().unwrap().serialize(sasspile::OutputStyle::Expanded).finish().unwrap();
    assert!(result.contains(".c .d"), "Expected original, got {result}");
    assert!(result.contains(".c + .e"), "Expected .c + .e, got {result}");
}

// ─── Parser error detection tests ────────────────────────────────

#[test]
fn test_unclosed_attribute_selector_error() {
    // selector.append("[c", "d") → error
    let input = r#"
        @use 'sass:selector';
        .a { x: selector.append("[c", "d"); }
    "#;
    let eval_result = Reactor::new(input).lex().unwrap().parse().unwrap().evaluate();
    assert!(eval_result.is_err(), "Expected error for unclosed attribute selector [c");
}

#[test]
fn test_unclosed_attribute_with_value_error() {
    // selector.append("[foo=bar", "d") → error
    let input = r#"
        @use 'sass:selector';
        .a { x: selector.append("[foo=bar", "d"); }
    "#;
    let eval_result = Reactor::new(input).lex().unwrap().parse().unwrap().evaluate();
    assert!(eval_result.is_err(), "Expected error for unclosed attribute selector [foo=bar");
}

#[test]
fn test_valid_attribute_selector_passes() {
    // selector.append([foo="bar"], "d") → success
    let input = r#"
        @use 'sass:selector';
        .a { x: selector.append('[foo="bar"]', "d"); }
    "#;
    let result = Reactor::new(input).lex().unwrap().parse().unwrap().evaluate().unwrap().serialize(sasspile::OutputStyle::Expanded).finish().unwrap();
    assert!(result.contains("[foo=\"bar\"]"), "Expected [foo=\"bar\"] in result, got {result}");
}

#[test]
fn test_invalid_characters_error() {
    // selector.append("!invalid", ".b") → error
    let input = r#"
        @use 'sass:selector';
        .a { x: selector.append("!invalid", ".b"); }
    "#;
    let eval_result = Reactor::new(input).lex().unwrap().parse().unwrap().evaluate();
    assert!(eval_result.is_err(), "Expected error for invalid selector characters");
}
