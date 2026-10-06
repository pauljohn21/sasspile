//! Unit tests for builtin modules: color, math, string, list, map.

use lightforger::{
    builtin::{
        self, color, list, map, math, string, Scope,
    },
    Value,
};

// ────────────────── Color tests ──────────────────

#[test]
fn color_darken_zero_pct_unchanged() {
    let result = color::darken(&[Value::String("#ff0000".into()), Value::Number(0.0)]).unwrap();
    assert_eq!(result, Value::String("#ff0000".into()));
}

#[test]
fn color_darken_100_pct_is_black() {
    let result = color::darken(&[Value::String("#ff8000".into()), Value::Number(100.0)]).unwrap();
    assert_eq!(result, Value::String("#000000".into()));
}

#[test]
fn color_lighten_zero_pct_unchanged() {
    let result = color::lighten(&[Value::String("#00ff00".into()), Value::Number(0.0)]).unwrap();
    assert_eq!(result, Value::String("#00ff00".into()));
}

#[test]
fn color_lighten_100_pct_is_white() {
    let result = color::lighten(&[Value::String("#008000".into()), Value::Number(100.0)]).unwrap();
    assert_eq!(result, Value::String("#ffffff".into()));
}

#[test]
fn color_mix_50_pct() {
    let result = color::mix(
        &[Value::String("#000000".into()), Value::String("#ffffff".into()), Value::Number(50.0)],
    )
    .unwrap();
    assert_eq!(result, Value::String("#808080".into()));
}

#[test]
fn color_parse_shorthand() {
    let result = color::darken(&[Value::String("#f00".into()), Value::Number(0.0)]).unwrap();
    assert_eq!(result, Value::String("#ff0000".into()));
}

// ────────────────── Math tests ──────────────────

#[test]
fn math_clamp_below_min() {
    let result = math::clamp(&[Value::Number(10.0), Value::Number(5.0), Value::Number(100.0)]).unwrap();
    assert_eq!(result, Value::Number(10.0));
}

#[test]
fn math_clamp_above_max() {
    // clamp(min, val, max): val exceeding max returns max
    let result = math::clamp(&[Value::Number(0.0), Value::Number(200.0), Value::Number(100.0)]).unwrap();
    assert_eq!(result, Value::Number(100.0));
}

#[test]
fn math_max_multiple() {
    let result = math::max(&[Value::Number(3.0), Value::Number(7.0), Value::Number(2.0)]).unwrap();
    assert_eq!(result, Value::Number(7.0));
}

#[test]
fn math_min_multiple() {
    let result = math::min(&[Value::Number(3.0), Value::Number(7.0), Value::Number(2.0)]).unwrap();
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn math_round_up() {
    let result = math::round(&[Value::Number(2.6)]).unwrap();
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn math_abs_negative() {
    let result = math::abs(&[Value::Number(-42.5)]).unwrap();
    assert_eq!(result, Value::Number(42.5));
}

#[test]
fn math_percentage_half() {
    let result = math::percentage(&[Value::Number(0.5)]).unwrap();
    assert_eq!(result, Value::Number(50.0));
}

// ────────────────── String tests ──────────────────

#[test]
fn string_index_found() {
    let result = string::index(&[Value::String("hello world".into()), Value::String("world".into())]).unwrap();
    assert_eq!(result, Value::Number(7.0));
}

#[test]
fn string_index_not_found() {
    let result = string::index(&[Value::String("hello".into()), Value::String("xyz".into())]).unwrap();
    assert_eq!(result, Value::Null);
}

#[test]
fn string_length_empty() {
    let result = string::length(&[Value::String("".into())]).unwrap();
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn string_length_nonempty() {
    let result = string::length(&[Value::String("hello".into())]).unwrap();
    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn string_slice_middle() {
    // Sass slice is 1-based, inclusive both ends: slice("hello", 2, 4) = "ell"
    let result = string::slice(
        &[Value::String("hello".into()), Value::Number(2.0), Value::Number(4.0)],
    )
    .unwrap();
    assert_eq!(result, Value::String("ell".into()));
}

#[test]
fn string_to_upper_case() {
    let result = string::to_upper_case(&[Value::String("hello".into())]).unwrap();
    assert_eq!(result, Value::String("HELLO".into()));
}

#[test]
fn string_to_lower_case() {
    let result = string::to_lower_case(&[Value::String("WORLD".into())]).unwrap();
    assert_eq!(result, Value::String("world".into()));
}

// ────────────────── List tests ──────────────────

#[test]
fn list_length_empty() {
    let result = list::length(&[Value::List(vec![])]).unwrap();
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn list_length_three() {
    let result = list::length(&[Value::List(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
    ])])
    .unwrap();
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn list_nth_first() {
    let result = list::nth(
        &[Value::List(vec![
            Value::String("a".into()),
            Value::String("b".into()),
        ])],
    )
    .unwrap();
    assert_eq!(result, Value::String("a".into()));
}

#[test]
fn list_nth_second() {
    let result = list::nth(
        &[Value::List(vec![
            Value::String("a".into()),
            Value::String("b".into()),
        ]), Value::Number(2.0)],
    )
    .unwrap();
    assert_eq!(result, Value::String("b".into()));
}

#[test]
fn list_nth_out_of_bounds() {
    // list.nth returns Err on out-of-bounds (does not panic)
    let result = list::nth(
        &[Value::List(vec![Value::Number(1.0)]), Value::Number(5.0)],
    );
    assert!(result.is_err());
}

#[test]
fn list_append_adds_to_end() {
    let result = list::append(
        &[Value::List(vec![Value::Number(1.0)]), Value::Number(2.0)],
    )
    .unwrap();
    assert_eq!(result, Value::List(vec![Value::Number(1.0), Value::Number(2.0)]));
}

#[test]
fn list_index_found() {
    let result = list::index(
        &[Value::List(vec![
            Value::Number(10.0),
            Value::Number(20.0),
        ]), Value::Number(20.0)],
    )
    .unwrap();
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn list_index_not_found() {
    let result = list::index(
        &[Value::List(vec![Value::Number(10.0)]), Value::Number(99.0)],
    )
    .unwrap();
    assert_eq!(result, Value::Null);
}

#[test]
fn list_join_with_separator() {
    let result = list::join(
        &[Value::List(vec![
            Value::String("a".into()),
            Value::String("b".into()),
            Value::String("c".into()),
        ]), Value::String("-".into())],
    )
    .unwrap();
    assert_eq!(result, Value::String("a-b-c".into()));
}

// ────────────────── Map tests ──────────────────

#[test]
fn map_get_existing_key() {
    let result = map::get(
        &[Value::Map(vec![("name".into(), Value::String("alice".into()))]), Value::String("name".into())],
    )
    .unwrap();
    assert_eq!(result, Value::String("alice".into()));
}

#[test]
fn map_get_missing_key() {
    let result = map::get(
        &[Value::Map(vec![("name".into(), Value::String("alice".into()))]), Value::String("age".into())],
    )
    .unwrap();
    assert_eq!(result, Value::Null);
}

#[test]
fn map_has_key_true() {
    let result = map::has_key(
        &[Value::Map(vec![("x".into(), Value::Number(1.0))]), Value::String("x".into())],
    )
    .unwrap();
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn map_has_key_false() {
    let result = map::has_key(
        &[Value::Map(vec![("x".into(), Value::Number(1.0))]), Value::String("y".into())],
    )
    .unwrap();
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn map_keys_returns_all() {
    let result = map::keys(
        &[Value::Map(vec![
            ("a".into(), Value::Number(1.0)),
            ("b".into(), Value::Number(2.0)),
        ])],
    )
    .unwrap();
    assert_eq!(
        result,
        Value::List(vec![Value::String("a".into()), Value::String("b".into())])
    );
}

#[test]
fn map_values_returns_all() {
    let result = map::values(
        &[Value::Map(vec![
            ("a".into(), Value::Number(1.0)),
            ("b".into(), Value::Number(2.0)),
        ])],
    )
    .unwrap();
    assert_eq!(
        result,
        Value::List(vec![Value::Number(1.0), Value::Number(2.0)])
    );
}

#[test]
fn map_merge_overwrites_conflict() {
    let result = map::merge(
        &[
            Value::Map(vec![("x".into(), Value::Number(1.0))]),
            Value::Map(vec![("x".into(), Value::Number(99.0))]),
        ],
    )
    .unwrap();
    match result {
        Value::Map(entries) => {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0], ("x".into(), Value::Number(99.0)));
        }
        _ => panic!("expected map"),
    }
}

#[test]
fn map_remove_deletes_key() {
    let result = map::remove(
        &[Value::Map(vec![
            ("a".into(), Value::Number(1.0)),
            ("b".into(), Value::Number(2.0)),
        ]), Value::String("a".into())],
    )
    .unwrap();
    match result {
        Value::Map(entries) => {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].0, "b");
        }
        _ => panic!("expected map"),
    }
}

// ────────────────── Scope registration test ──────────────────

#[test]
fn scope_register_all_builtins_accessible() {
    let scope = Scope::new();

    // Verify all builtin function names are registered
    assert!(scope.contains("color.darken"));
    assert!(scope.contains("color.lighten"));
    assert!(scope.contains("color.mix"));
    assert!(scope.contains("math.clamp"));
    assert!(scope.contains("math.percentage"));
    assert!(scope.contains("string.index"));
    assert!(scope.contains("string.length"));
    assert!(scope.contains("list.nth"));
    assert!(scope.contains("list.length"));
    assert!(scope.contains("map.get"));
    assert!(scope.contains("map.has-key"));
}

#[test]
fn scope_call_builtin_math_percentage() {
    let scope = Scope::new();
    let result = scope
        .call("math.percentage", &[Value::Number(0.5)])
        .unwrap();
    assert_eq!(result, Value::Number(50.0));
}
