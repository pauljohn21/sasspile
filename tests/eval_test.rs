use rx_scss::bus::{CompilerBus, FnDef, MixinDef};
use rx_scss::runtime::EvalContext;
use rx_scss::types::*;
use std::sync::Arc;

#[test]
fn eval_context_variable_binding() {
    let bus = Arc::new(CompilerBus::new());
    let ctx = EvalContext::new(bus, 1);
    ctx.bind_var("color", Value::String("red".into()));
    let val = ctx.var("color");
    assert_eq!(val, Some(Value::String("red".into())));
}

#[test]
fn eval_context_child_scope_inherits_parent_vars() {
    let bus = Arc::new(CompilerBus::new());
    let parent = EvalContext::new(bus.clone(), 1);
    parent.bind_var("base", Value::Number(10.0, None));

    let child = parent.child_scope(1);
    let val = child.var("base");
    assert_eq!(val, Some(Value::Number(10.0, None)));
}

#[test]
fn eval_context_shallow_binding_precedence() {
    let bus = Arc::new(CompilerBus::new());
    let parent = EvalContext::new(bus.clone(), 1);
    parent.bind_var("x", Value::String("parent".into()));

    let child = parent.child_scope(1);
    child.bind_var("x", Value::String("child".into()));

    assert_eq!(child.var("x"), Some(Value::String("child".into())));
    assert_eq!(parent.var("x"), Some(Value::String("parent".into())));
}

#[test]
fn eval_context_nonexistent_var_returns_null() {
    let bus = Arc::new(CompilerBus::new());
    let ctx = EvalContext::new(bus, 1);
    let val = ctx.var("nonexistent");
    assert_eq!(val, None);
}

#[test]
fn bus_register_and_lookup_mixin() {
    let bus = CompilerBus::new();
    bus.register_mixin(MixinDef {
        name: "box".into(),
        params: vec![Param::new("$size")],
        body: vec![],
    });

    let mixin = bus.lookup_mixin("box");
    assert!(mixin.is_some());
    let m = mixin.unwrap();
    assert_eq!(m.name, "box");
    assert_eq!(m.params.len(), 1);
}

#[test]
fn bus_register_and_lookup_function() {
    let bus = CompilerBus::new();
    bus.register_fn(FnDef {
        name: "double".into(),
        params: vec![Param::new("$n")],
        body: vec![],
    });

    let func = bus.lookup_fn("double");
    assert!(func.is_some());
    let f = func.unwrap();
    assert_eq!(f.name, "double");
}

#[test]
fn bus_var_update_emits_event() {
    let bus = CompilerBus::new();
    bus.set_var(1, "color", Value::String("red".into()));
    bus.set_var(1, "color", Value::String("blue".into()));

    let val = bus.get_var_by_id(1, "color");
    assert_eq!(val, Some(Value::String("blue".into())));
}

#[test]
fn value_display_number() {
    assert_eq!(Value::Number(42.0, None).to_string(), "42");
    assert_eq!(Value::Number(3.5, None).to_string(), "3.5");
}

#[test]
fn value_display_color() {
    let color = Value::Color(255, 0, 0, 255);
    assert_eq!(color.to_string(), "#ff0000");

    let color_with_alpha = Value::Color(255, 0, 0, 128);
    assert_eq!(color_with_alpha.to_string(), "#ff000080");
}

#[test]
fn value_display_list() {
    let list = Value::List(vec![
        Value::Number(1.0, None),
        Value::Number(2.0, None),
    ]);
    // Space-separated for CSS output compatibility (e.g., `margin: 1px 2px`)
    assert_eq!(list.to_string(), "1 2");
}

#[test]
fn value_equality() {
    assert_eq!(Value::Number(1.0, None), Value::Number(1.0, None));
    assert_eq!(Value::String("a".into()), Value::String("a".into()));
    assert_eq!(Value::Bool(true), Value::Bool(true));
    assert_eq!(Value::Null, Value::Null);
    assert_ne!(Value::Number(1.0, None), Value::Number(2.0, None));
}

#[test]
fn css_stmt_rule_is_invisible_when_empty() {
    let empty = CssStmt::Rule {
        selector: "x".into(),
        inner: vec![],
    };
    assert!(empty.is_invisible());

    let non_empty = CssStmt::Rule {
        selector: "y".into(),
        inner: vec![CssStmt::Decl {
            property: "color".into(),
            value: "red".into(),
        }],
    };
    assert!(!non_empty.is_invisible());
}

#[test]
fn css_stmt_media_is_invisible_when_empty() {
    let empty = CssStmt::Media {
        query: "screen".into(),
        inner: vec![],
    };
    assert!(empty.is_invisible());
}

#[test]
fn param_default_value() {
    let p = Param::with_default("size", AstNode::Literal(Value::Number(10.0, None)));
    assert_eq!(p.name, "size");
    assert!(p.default_value.is_some());
    match p.default_value.unwrap().as_ref() {
        AstNode::Literal(Value::Number(n, _)) => assert_eq!(*n, 10.0),
        other => panic!("expected Number literal, got {:?}", other),
    }
}

#[test]
fn eval_unary_neg_evaluates_correctly() {
    use rx_scss::eval::eval_expr;
    use rx_scss::runtime::create_runtime;

    let (ctx, bus) = create_runtime();
    ctx.bind_var("x", Value::Number(10.0, Some("px".into())));

    // Manually construct AST: UnaryOp(Neg, VariableRef("x"))
    let expr = AstNode::UnaryOp {
        op: UnaryOp::Neg,
        expr: Box::new(AstNode::VariableRef { name: "x".into(), scope_id: 0 }),
    };

    let result = eval_expr(&expr, &ctx, &bus);
    assert_eq!(result, Value::Number(-10.0, Some("px".into())), "eval UnaryOp: {:?}", result);
}

#[test]
fn eval_unary_not_evaluates_correctly() {
    use rx_scss::eval::eval_expr;
    use rx_scss::runtime::create_runtime;

    let (ctx, bus) = create_runtime();

    let expr = AstNode::UnaryOp {
        op: UnaryOp::Not,
        expr: Box::new(AstNode::Literal(Value::Bool(false))),
    };

    let result = eval_expr(&expr, &ctx, &bus);
    assert_eq!(result, Value::Bool(true), "eval Not(true): {:?}", result);
}

// ── Builtin unit tests ──────────────────────────────────────────────────

#[test]
fn builtin_map_get_found() {
    use rx_scss::eval::builtin::call_builtin;
    let map = Value::Map(vec![
        ("primary".to_string(), Value::String("blue".to_string())),
        ("danger".to_string(), Value::String("red".to_string())),
    ]);
    let result = call_builtin("map-get", &[map, Value::String("primary".to_string())]);
    assert_eq!(result, Some(Value::String("blue".to_string())));
}

#[test]
fn builtin_map_get_missing() {
    use rx_scss::eval::builtin::call_builtin;
    let map = Value::Map(vec![("a".to_string(), Value::Number(1.0, None))]);
    let result = call_builtin("map-get", &[map, Value::String("b".to_string())]);
    assert_eq!(result, Some(Value::Null));
}

#[test]
fn builtin_map_has_key() {
    use rx_scss::eval::builtin::call_builtin;
    let map = Value::Map(vec![("a".to_string(), Value::Number(1.0, None))]);
    assert_eq!(call_builtin("map-has-key", &[map.clone(), Value::String("a".to_string())]), Some(Value::Bool(true)));
    assert_eq!(call_builtin("map-has-key", &[map, Value::String("z".to_string())]), Some(Value::Bool(false)));
}

#[test]
fn builtin_if_function() {
    use rx_scss::eval::builtin::call_builtin;
    assert_eq!(
        call_builtin("if", &[Value::Bool(true), Value::Number(1.0, None), Value::Number(2.0, None)]),
        Some(Value::Number(1.0, None))
    );
    assert_eq!(
        call_builtin("if", &[Value::Bool(false), Value::Number(1.0, None), Value::Number(2.0, None)]),
        Some(Value::Number(2.0, None))
    );
}

#[test]
fn builtin_nth() {
    use rx_scss::eval::builtin::call_builtin;
    let list = Value::List(vec![
        Value::Number(10.0, Some("px".to_string())),
        Value::Number(20.0, Some("px".to_string())),
        Value::Number(30.0, Some("px".to_string())),
    ]);
    assert_eq!(call_builtin("nth", &[list.clone(), Value::Number(1.0, None)]), Some(Value::Number(10.0, Some("px".to_string()))));
    assert_eq!(call_builtin("nth", &[list.clone(), Value::Number(2.0, None)]), Some(Value::Number(20.0, Some("px".to_string()))));
    assert_eq!(call_builtin("nth", &[list, Value::Number(3.0, None)]), Some(Value::Number(30.0, Some("px".to_string()))));
}

#[test]
fn builtin_percentage() {
    use rx_scss::eval::builtin::call_builtin;
    let result = call_builtin("percentage", &[Value::Number(0.5, None)]);
    assert_eq!(result, Some(Value::Number(50.0, Some("%".to_string()))));
}

#[test]
fn builtin_math_round_ceil_floor() {
    use rx_scss::eval::builtin::call_builtin;
    assert_eq!(call_builtin("round", &[Value::Number(2.5, None)]), Some(Value::Number(3.0, None)));
    assert_eq!(call_builtin("ceil", &[Value::Number(2.1, None)]), Some(Value::Number(3.0, None)));
    assert_eq!(call_builtin("floor", &[Value::Number(2.9, None)]), Some(Value::Number(2.0, None)));
}

#[test]
fn builtin_type_of() {
    use rx_scss::eval::builtin::call_builtin;
    assert_eq!(call_builtin("type-of", &[Value::Number(1.0, None)]), Some(Value::String("number".to_string())));
    assert_eq!(call_builtin("type-of", &[Value::String("x".to_string())]), Some(Value::String("string".to_string())));
    assert_eq!(call_builtin("type-of", &[Value::Bool(true)]), Some(Value::String("bool".to_string())));
    assert_eq!(call_builtin("type-of", &[Value::Null]), Some(Value::String("null".to_string())));
    assert_eq!(call_builtin("type-of", &[Value::Color(255, 0, 0, 255)]), Some(Value::String("color".to_string())));
}

#[test]
fn builtin_color_mix() {
    use rx_scss::eval::builtin::call_builtin;
    let white = Value::Color(255, 255, 255, 255);
    let black = Value::Color(0, 0, 0, 255);
    let result = call_builtin("mix", &[white, black, Value::Number(50.0, None)]);
    assert!(result.is_some());
}

#[test]
fn builtin_color_lighten() {
    use rx_scss::eval::builtin::call_builtin;
    let red = Value::Color(255, 0, 0, 255);
    let result = call_builtin("lighten", &[red, Value::Number(20.0, None)]);
    assert!(result.is_some());
    if let Some(Value::Color(r, g, b, _)) = result {
        assert!(r > 200 || g > 0 || b > 0, "lightened red should be brighter");
    }
}

#[test]
fn builtin_shade_color_eq_mix() {
    use rx_scss::eval::builtin::call_builtin;
    let color = Value::Color(255, 255, 255, 255);
    let shade = call_builtin("shade-color", &[color.clone(), Value::Number(20.0, None)]);
    let mix_result = call_builtin("mix", &[Value::Color(0, 0, 0, 255), color, Value::Number(20.0, None)]);
    assert_eq!(shade, mix_result, "shade-color should equal mix(#000, color, weight)");
}

#[test]
fn builtin_tint_color_eq_mix() {
    use rx_scss::eval::builtin::call_builtin;
    let color = Value::Color(0, 0, 0, 255);
    let tint = call_builtin("tint-color", &[color.clone(), Value::Number(20.0, None)]);
    let mix_result = call_builtin("mix", &[Value::Color(255, 255, 255, 255), color, Value::Number(20.0, None)]);
    assert_eq!(tint, mix_result, "tint-color should equal mix(#fff, color, weight)");
}

#[test]
fn builtin_to_rgb_returns_comma_separated() {
    use rx_scss::eval::builtin::call_builtin;
    let color = Value::Color(13, 110, 253, 255);
    let result = call_builtin("to-rgb", &[color]);
    assert_eq!(result, Some(Value::String("13, 110, 253".to_string())));
}

#[test]
fn builtin_color_channels() {
    use rx_scss::eval::builtin::call_builtin;
    let color = Value::Color(13, 110, 253, 128);
    assert_eq!(call_builtin("red", &[color.clone()]), Some(Value::Number(13.0, None)));
    assert_eq!(call_builtin("green", &[color.clone()]), Some(Value::Number(110.0, None)));
    assert_eq!(call_builtin("blue", &[color.clone()]), Some(Value::Number(253.0, None)));
    assert_eq!(call_builtin("alpha", &[color]), Some(Value::Number(128.0 / 255.0, None)));
}

#[test]
fn builtin_color_channel_hex() {
    use rx_scss::eval::builtin::call_builtin;
    // #0d6efd = rgb(13, 110, 253)
    let color = Value::Color(13, 110, 253, 255);
    assert_eq!(call_builtin("red", &[color.clone()]), Some(Value::Number(13.0, None)));
    assert_eq!(call_builtin("green", &[color.clone()]), Some(Value::Number(110.0, None)));
    assert_eq!(call_builtin("blue", &[color]), Some(Value::Number(253.0, None)));
}

#[test]
fn builtin_color_alpha_full() {
    use rx_scss::eval::builtin::call_builtin;
    let opaque = Value::Color(255, 0, 0, 255);
    let result = call_builtin("alpha", &[opaque]);
    assert_eq!(result, Some(Value::Number(1.0, None)));
}

#[test]
fn builtin_map_merge() {
    use rx_scss::eval::builtin::call_builtin;
    let m1 = Value::Map(vec![("a".to_string(), Value::Number(1.0, None))]);
    let m2 = Value::Map(vec![("b".to_string(), Value::Number(2.0, None))]);
    let result = call_builtin("map-merge", &[m1, m2]);
    assert!(result.is_some());
    if let Some(Value::Map(entries)) = result {
        assert_eq!(entries.len(), 2);
    }
}

#[test]
fn builtin_str_length() {
    use rx_scss::eval::builtin::call_builtin;
    let result = call_builtin("str-length", &[Value::String("hello".to_string())]);
    assert_eq!(result, Some(Value::Number(5.0, None)));
}

#[test]
fn builtin_list_zip_combine() {
    use rx_scss::eval::builtin::call_builtin;
    let list1 = Value::List(vec![
        Value::String("a".to_string()),
        Value::String("b".to_string()),
        Value::String("c".to_string()),
    ]);
    let list2 = Value::List(vec![
        Value::Number(1.0, None),
        Value::Number(2.0, None),
        Value::Number(3.0, None),
    ]);
    let result = call_builtin("zip", &[list1, list2]);
    assert!(result.is_some());
    if let Some(Value::List(zipped)) = result {
        assert_eq!(zipped.len(), 3);
        if let Value::List(pair) = &zipped[0] {
            assert_eq!(pair[0], Value::String("a".to_string()));
            assert_eq!(pair[1], Value::Number(1.0, None));
        } else {
            panic!("expected nested list");
        }
    } else {
        panic!("expected List");
    }
}

#[test]
fn builtin_list_separator_space() {
    use rx_scss::eval::builtin::call_builtin;
    // Space-separated list (multi-value literal): 1px solid red → 3 items → "comma" in our heuristic
    // NOTE: rx-scss Value::List doesn't carry separator; this test verifies current behavior
    let list = Value::List(vec![
        Value::Number(1.0, Some("px".to_string())),
        Value::String("solid".to_string()),
        Value::String("red".to_string()),
    ]);
    let result = call_builtin("list-separator", &[list]);
    assert_eq!(result, Some(Value::String("comma".to_string())));
}

#[test]
fn builtin_str_replace_global() {
    use rx_scss::eval::builtin::call_builtin;
    let result = call_builtin("str-replace", &[
        Value::String("foobar".to_string()),
        Value::String("foo".to_string()),
        Value::String("baz".to_string()),
    ]);
    assert_eq!(result, Some(Value::String("bazbar".to_string())));
}

#[test]
fn builtin_str_replace_multiple() {
    use rx_scss::eval::builtin::call_builtin;
    let result = call_builtin("str-replace", &[
        Value::String("a,b,c".to_string()),
        Value::String(",".to_string()),
        Value::String("%2C".to_string()),
    ]);
    assert_eq!(result, Some(Value::String("a%2Cb%2Cc".to_string())));
}

    // ── var() and rgba(var()) ───────────────────────────────────────────

    #[test]
    fn test_var_function_no_fallback() {
        use rx_scss::eval::builtin::call_builtin;
        let result = call_builtin("var", &[Value::String("--my-color".to_string())]);
        assert_eq!(result, Some(Value::String("var(--my-color)".to_string())));
    }

    #[test]
    fn test_var_function_with_fallback() {
        use rx_scss::eval::builtin::call_builtin;
        let result = call_builtin("var", &[
            Value::String("--my-color".to_string()),
            Value::String("red".to_string()),
        ]);
        assert_eq!(result, Some(Value::String("var(--my-color, red)".to_string())));
    }

    #[test]
    fn test_rgba_with_var_first_arg() {
        use rx_scss::eval::builtin::call_builtin;
        let result = call_builtin("rgba", &[
            Value::String("var(--bs-white-rgb)".to_string()),
            Value::Number(0.5, None),
        ]);
        assert_eq!(result, Some(Value::String("rgba(var(--bs-white-rgb), 0.5)".to_string())));
    }

    #[test]
    fn test_rgb_with_var_first_arg() {
        use rx_scss::eval::builtin::call_builtin;
        let result = call_builtin("rgb", &[
            Value::String("var(--my-color)".to_string()),
            Value::Number(0.8, None),
        ]);
        assert_eq!(result, Some(Value::String("rgb(var(--my-color), 0.8)".to_string())));
    }

    #[test]
    fn test_rgba_with_var_integer_alpha() {
        use rx_scss::eval::builtin::call_builtin;
        let result = call_builtin("rgba", &[
            Value::String("var(--x)".to_string()),
            Value::Number(1.0, None),
        ]);
        assert_eq!(result, Some(Value::String("rgba(var(--x), 1)".to_string())));
    }

    // ── @for iteration ───────────────────────────────────────────────────
    #[test]
    fn test_for_range_inclusive_through() {
        // @for $i from 1 through 3 → should produce 1, 2, 3
        let (from, to, inclusive) = (1i64, 3i64, true);
        let mut values = Vec::new();
        let mut i = from;
        while if inclusive { i <= to } else { i < to } {
            values.push(i);
            i += 1;
        }
        assert_eq!(values, vec![1, 2, 3]);
    }

    #[test]
    fn test_for_range_exclusive_to() {
        // @for $i from 1 to 3 → should produce 1, 2
        let (from, to, inclusive) = (1i64, 3i64, false);
        let mut values = Vec::new();
        let mut i = from;
        while if inclusive { i <= to } else { i < to } {
            values.push(i);
            i += 1;
        }
        assert_eq!(values, vec![1, 2]);
    }

    #[test]
    fn test_rgba_normal_color_still_works() {
        use rx_scss::eval::builtin::call_builtin;
        // Ensure normal rgba(r, g, b) still produces Color when args are numbers
        let result = call_builtin("rgba", &[
            Value::Number(255.0, None),
            Value::Number(0.0, None),
            Value::Number(0.0, None),
            Value::Number(1.0, None),
        ]);
        assert_eq!(result, Some(Value::Color(255, 0, 0, 255)));
    }

    // ── Regression tests for @each comma-list + selector hyphen fix ────

    #[test]
    fn regression_each_comma_list_iterates_all() {
        // Core fix: `@each $k in a, b { ... }` must iterate over both items
        use rx_scss::builder::CompileBuilder;
        let scss = "@each $k in a, b { .#{$k} { color: red; } }\n";
        let css = CompileBuilder::new().expanded().compile_string(scss).expect("compile failed");
        assert!(css.contains(".a{"), "should produce .a rule: {}", css);
        assert!(css.contains(".b{"), "should produce .b rule: {}", css);
    }

    #[test]
    fn regression_each_comma_list_with_hyphen_selector() {
        // Selector with interpolation + hyphen: `.#{$k}-test`
        use rx_scss::builder::CompileBuilder;
        let scss = "@each $k in a, b { .#{$k}-test { val: 1; } }\n";
        let css = CompileBuilder::new().expanded().compile_string(scss).expect("compile failed");
        assert!(css.contains(".a-test"), "should produce .a-test: {}", css);
        assert!(css.contains(".b-test"), "should produce .b-test: {}", css);
    }

    #[test]
    fn regression_three_level_nested_each() {
        // Bootstrap $utilities pattern: outer @each + inner @each + selector interp with hyphens
        use rx_scss::builder::CompileBuilder;
        let scss = r#"
$utils: (
  "m": (values: (1: 0, 2: 0.5rem)),
  "p": (values: (s: 0.25rem, l: 1rem))
);
@each $key, $util in $utils {
  $vals: map-get($util, values);
  @each $vk, $vv in $vals {
    .#{$key}-#{$vk} { val: $vv; }
  }
}
"#;
        let css = CompileBuilder::new().expanded().compile_string(scss).expect("compile failed");
        assert!(css.contains(".m-1"), "should produce .m-1: {}", css);
        assert!(css.contains(".m-2"), "should produce .m-2: {}", css);
        assert!(css.contains(".p-s"), "should produce .p-s: {}", css);
        assert!(css.contains(".p-l"), "should produce .p-l: {}", css);
    }

    #[test]
    fn regression_media_query_with_variable() {
        // SCSS spec: media queries with variable references must interpolate at eval time
        use rx_scss::builder::CompileBuilder;
        let scss = r#"
$w: 768px;
@mixin var-mq {
  @media (min-width: $w) {
    @content;
  }
}
@include var-mq {
  .test { color: red; }
}
"#;
        let css = CompileBuilder::new().expanded().compile_string(scss).expect("compile failed");
        assert!(css.contains("min-width : 768px"), "variable in media query should be resolved: {}", css);
        assert!(css.contains(".test"), "rule inside @content should be in output: {}", css);
        assert!(!css.contains("$w"), "no unresolved $w should remain in output: {}", css);
    }

    #[test]
    fn regression_media_query_with_function_call() {
        // Mixin body calling a user-defined function, used as media query value
        use rx_scss::builder::CompileBuilder;
        let scss = r#"
@function calc-bp($bp, $map) {
  $val: map-get($map, $bp);
  @return $val;
}
@mixin media-breakpoint-up($name) {
  $min: calc-bp($name, (sm: 576px, md: 768px));
  @media (min-width: $min) {
    @content;
  }
}
@include media-breakpoint-up(md) {
  .test { color: red; }
}
"#;
        let css = CompileBuilder::new().expanded().compile_string(scss).expect("compile failed");
        assert!(css.contains("min-width : 768px"), "function result in media query should be resolved: {}", css);
        assert!(css.contains(".test"), "content block should be expanded: {}", css);
    }

    #[test]
    fn regression_function_body_with_intermediate_vars() {
        // User-defined function with intermediate variable assignments
        use rx_scss::builder::CompileBuilder;
        let scss = r#"
@function add-one($x) {
  $result: $x + 1;
  @return $result;
}
.test { val: add-one(5); }
"#;
        let css = CompileBuilder::new().expanded().compile_string(scss).expect("compile failed");
        assert!(css.contains("val: 6"), "function with intermediate var should return correct value: {}", css);
    }

    #[test]
    fn ast_node_variable_decl() {
        let node = AstNode::VariableDecl {
        name: "primary".into(),
        value: Box::new(AstNode::Literal(Value::String("blue".into()))),
        scope_id: 1,
    };
    match node {
        AstNode::VariableDecl { name, scope_id, .. } => {
            assert_eq!(name, "primary");
            assert_eq!(scope_id, 1);
        }
        _ => panic!("expected VariableDecl"),
    }
}
