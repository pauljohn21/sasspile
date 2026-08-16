//! Built-in module tests — sass:color, sass:math, sass:list, sass:map, sass:string, sass:meta.

use sasspile::{
    DefinitionRegistry, EvalContext, SymbolTable, Value,
    parser, value,
};

/// Helper: create an eval context.
fn make_ctx<'a>(
    symbols: &'a mut SymbolTable,
    definitions: &'a DefinitionRegistry,
) -> EvalContext<'a> {
    EvalContext::new(symbols, definitions)
}

/// Helper: evaluate an expression to a Value.
fn eval(expr: parser::Expr) -> Value {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);
    ctx.eval_expr(&expr).unwrap()
}

// ============================================================
// sass:color module tests
// ============================================================

#[test]
fn color_lighten() {
    let result = eval(parser::Expr::Call(
        "color.lighten".to_string(),
        vec![
            parser::Expr::Color(0x0000FF), // blue
            parser::Expr::Number(20.0, None),
        ],
    ));
    // blue lightened by 20% should be lighter.
    match &result {
        Value::Color(c) => {
            assert!(c.lightness() > 0.5, "expected lighter than 0.5, got {}", c.lightness());
        }
        _ => panic!("expected color, got {:?}", result),
    }
}

#[test]
fn color_darken() {
    let result = eval(parser::Expr::Call(
        "color.darken".to_string(),
        vec![
            parser::Expr::Color(0x0000FF), // blue
            parser::Expr::Number(20.0, None),
        ],
    ));
    match &result {
        Value::Color(c) => {
            assert!(c.lightness() < 1.0, "expected darker than 1.0");
        }
        _ => panic!("expected color"),
    }
}

#[test]
fn color_saturate() {
    let result = eval(parser::Expr::Call(
        "color.saturate".to_string(),
        vec![
            parser::Expr::Color(0x808080), // gray
            parser::Expr::Number(50.0, None),
        ],
    ));
    match &result {
        Value::Color(c) => {
            assert!(c.saturation() > 0.0, "expected increased saturation");
        }
        _ => panic!("expected color"),
    }
}

#[test]
fn color_desaturate() {
    let result = eval(parser::Expr::Call(
        "color.desaturate".to_string(),
        vec![
            parser::Expr::Color(0xFF0000), // red
            parser::Expr::Number(100.0, None),
        ],
    ));
    match &result {
        Value::Color(c) => {
            assert!(c.saturation() < 0.01, "expected ~0 saturation, got {}", c.saturation());
        }
        _ => panic!("expected color"),
    }
}

#[test]
fn color_mix() {
    let result = eval(parser::Expr::Call(
        "color.mix".to_string(),
        vec![
            parser::Expr::Color(0xFF0000), // red
            parser::Expr::Color(0x0000FF), // blue
            parser::Expr::Number(50.0, None),
        ],
    ));
    match &result {
        Value::Color(c) => {
            // 50% mix of red and blue.
            assert!(c.r > 0 && c.b > 0, "expected mix to have both red and blue");
        }
        _ => panic!("expected color"),
    }
}

#[test]
fn color_hue() {
    let result = eval(parser::Expr::Call(
        "color.hue".to_string(),
        vec![parser::Expr::Color(0xFF0000)], // red = hue 0
    ));
    // Pure red has hue ~0.
    match &result {
        Value::Number(n) => {
            assert!(n.value < 1.0 || n.value > 359.0, "expected hue ~0 for red, got {}", n.value);
        }
        _ => panic!("expected number"),
    }
}

#[test]
fn color_lightness() {
    let result = eval(parser::Expr::Call(
        "color.lightness".to_string(),
        vec![parser::Expr::Color(0xFFFFFF)], // white = 100%
    ));
    match &result {
        Value::Number(n) => {
            assert!((n.value - 100.0).abs() < 1.0, "expected ~100% for white, got {}", n.value);
        }
        _ => panic!("expected number"),
    }
}

#[test]
fn color_saturation() {
    let result = eval(parser::Expr::Call(
        "color.saturation".to_string(),
        vec![parser::Expr::Color(0xFF0000)], // red = 100%
    ));
    match &result {
        Value::Number(n) => {
            assert!((n.value - 100.0).abs() < 1.0, "expected ~100% for red, got {}", n.value);
        }
        _ => panic!("expected number"),
    }
}

#[test]
fn color_alpha() {
    let result = eval(parser::Expr::Call(
        "color.alpha".to_string(),
        vec![parser::Expr::Color(0xFF0000)],
    ));
    match &result {
        Value::Number(n) => {
            assert!((n.value - 1.0).abs() < 0.001, "expected alpha ~1.0");
        }
        _ => panic!("expected number"),
    }
}

#[test]
fn color_complement() {
    let result = eval(parser::Expr::Call(
        "color.complement".to_string(),
        vec![parser::Expr::Color(0xFF0000)], // red -> cyan
    ));
    match &result {
        Value::Color(c) => {
            // Complement of red (hue 0) should be cyan (hue 180).
            assert!(c.g > 0 && c.b > 0, "expected cyan-ish complement");
        }
        _ => panic!("expected color"),
    }
}

#[test]
fn color_adjust_hue() {
    let result = eval(parser::Expr::Call(
        "color.adjust-hue".to_string(),
        vec![
            parser::Expr::Color(0xFF0000),
            parser::Expr::Number(120.0, None),
        ],
    ));
    match &result {
        Value::Color(c) => {
            // Red + 120 degrees = green.
            assert!(c.g > 0, "expected green-ish");
        }
        _ => panic!("expected color"),
    }
}

#[test]
fn color_invert() {
    let result = eval(parser::Expr::Call(
        "color.invert".to_string(),
        vec![parser::Expr::Color(0xFFFFFF)],
    ));
    assert_eq!(result, Value::Color(value::SassColor::from_hex(0x000000)));
}

#[test]
fn color_grayscale() {
    let result = eval(parser::Expr::Call(
        "color.grayscale".to_string(),
        vec![parser::Expr::Color(0xFF0000)],
    ));
    match &result {
        Value::Color(c) => {
            assert!(c.saturation() < 0.01, "expected near-zero saturation");
        }
        _ => panic!("expected color"),
    }
}

// ============================================================
// sass:math module tests
// ============================================================

#[test]
fn math_pi() {
    let result = eval(parser::Expr::Call("math.pi".to_string(), vec![]));
    match &result {
        Value::Number(n) => {
            assert!((n.value - std::f64::consts::PI).abs() < 0.0001);
        }
        _ => panic!("expected number"),
    }
}

#[test]
fn math_ceil() {
    let result = eval(parser::Expr::Call(
        "math.ceil".to_string(),
        vec![parser::Expr::Number(3.2, None)],
    ));
    assert_eq!(result, Value::Number(value::Number::unitless(4.0)));
}

#[test]
fn math_floor() {
    let result = eval(parser::Expr::Call(
        "math.floor".to_string(),
        vec![parser::Expr::Number(3.8, None)],
    ));
    assert_eq!(result, Value::Number(value::Number::unitless(3.0)));
}

#[test]
fn math_round() {
    let result = eval(parser::Expr::Call(
        "math.round".to_string(),
        vec![parser::Expr::Number(3.5, None)],
    ));
    assert_eq!(result, Value::Number(value::Number::unitless(4.0)));
}

#[test]
fn math_abs() {
    let result = eval(parser::Expr::Call(
        "math.abs".to_string(),
        vec![parser::Expr::Number(-7.0, None)],
    ));
    assert_eq!(result, Value::Number(value::Number::unitless(7.0)));
}

#[test]
fn math_min() {
    let result = eval(parser::Expr::Call(
        "math.min".to_string(),
        vec![
            parser::Expr::Number(5.0, None),
            parser::Expr::Number(2.0, None),
            parser::Expr::Number(8.0, None),
        ],
    ));
    assert_eq!(result, Value::Number(value::Number::unitless(2.0)));
}

#[test]
fn math_max() {
    let result = eval(parser::Expr::Call(
        "math.max".to_string(),
        vec![
            parser::Expr::Number(5.0, None),
            parser::Expr::Number(2.0, None),
            parser::Expr::Number(8.0, None),
        ],
    ));
    assert_eq!(result, Value::Number(value::Number::unitless(8.0)));
}

#[test]
fn math_percentage() {
    let result = eval(parser::Expr::Call(
        "math.percentage".to_string(),
        vec![parser::Expr::Number(0.5, None)],
    ));
    assert_eq!(
        result,
        Value::Number(value::Number::new(50.0, value::Unit::Percent))
    );
}

#[test]
fn math_pow() {
    let result = eval(parser::Expr::Call(
        "math.pow".to_string(),
        vec![
            parser::Expr::Number(2.0, None),
            parser::Expr::Number(3.0, None),
        ],
    ));
    assert_eq!(result, Value::Number(value::Number::unitless(8.0)));
}

#[test]
fn math_sqrt() {
    let result = eval(parser::Expr::Call(
        "math.sqrt".to_string(),
        vec![parser::Expr::Number(16.0, None)],
    ));
    assert_eq!(result, Value::Number(value::Number::unitless(4.0)));
}

#[test]
fn math_sin() {
    let result = eval(parser::Expr::Call(
        "math.sin".to_string(),
        vec![parser::Expr::Number(0.0, None)],
    ));
    match &result {
        Value::Number(n) => {
            assert!(n.value.abs() < 0.0001, "sin(0) should be 0");
        }
        _ => panic!("expected number"),
    }
}

#[test]
fn math_cos() {
    let result = eval(parser::Expr::Call(
        "math.cos".to_string(),
        vec![parser::Expr::Number(0.0, None)],
    ));
    match &result {
        Value::Number(n) => {
            assert!((n.value - 1.0).abs() < 0.0001, "cos(0) should be 1");
        }
        _ => panic!("expected number"),
    }
}

#[test]
fn math_unitless() {
    let result = eval(parser::Expr::Call(
        "math.unitless".to_string(),
        vec![parser::Expr::Number(5.0, None)],
    ));
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn math_compatible() {
    let result = eval(parser::Expr::Call(
        "math.compatible".to_string(),
        vec![
            parser::Expr::Number(1.0, Some("px".to_string())),
            parser::Expr::Number(2.0, Some("pt".to_string())),
        ],
    ));
    // px and pt are compatible.
    assert_eq!(result, Value::Boolean(true));
}

// ============================================================
// sass:list module tests
// ============================================================

#[test]
fn list_length() {
    let result = eval(parser::Expr::Call(
        "list.length".to_string(),
        vec![parser::Expr::List(vec![
            parser::Expr::Number(1.0, None),
            parser::Expr::Number(2.0, None),
            parser::Expr::Number(3.0, None),
        ])],
    ));
    assert_eq!(result, Value::Number(value::Number::unitless(3.0)));
}

#[test]
fn list_nth() {
    let result = eval(parser::Expr::Call(
        "list.nth".to_string(),
        vec![
            parser::Expr::List(vec![
                parser::Expr::Number(10.0, None),
                parser::Expr::Number(20.0, None),
                parser::Expr::Number(30.0, None),
            ]),
            parser::Expr::Number(2.0, None),
        ],
    ));
    assert_eq!(result, Value::Number(value::Number::unitless(20.0)));
}

#[test]
fn list_join() {
    let result = eval(parser::Expr::Call(
        "list.join".to_string(),
        vec![
            parser::Expr::List(vec![
                parser::Expr::Number(1.0, None),
                parser::Expr::Number(2.0, None),
            ]),
            parser::Expr::List(vec![
                parser::Expr::Number(3.0, None),
                parser::Expr::Number(4.0, None),
            ]),
        ],
    ));
    assert_eq!(
        result,
        Value::List(
            vec![
                Value::Number(value::Number::unitless(1.0)),
                Value::Number(value::Number::unitless(2.0)),
                Value::Number(value::Number::unitless(3.0)),
                Value::Number(value::Number::unitless(4.0)),
            ],
            value::Separator::Space,
        )
    );
}

#[test]
fn list_append() {
    let result = eval(parser::Expr::Call(
        "list.append".to_string(),
        vec![
            parser::Expr::List(vec![
                parser::Expr::Number(1.0, None),
                parser::Expr::Number(2.0, None),
            ]),
            parser::Expr::Number(3.0, None),
        ],
    ));
    assert_eq!(
        result,
        Value::List(
            vec![
                Value::Number(value::Number::unitless(1.0)),
                Value::Number(value::Number::unitless(2.0)),
                Value::Number(value::Number::unitless(3.0)),
            ],
            value::Separator::Space,
        )
    );
}

#[test]
fn list_index() {
    let result = eval(parser::Expr::Call(
        "list.index".to_string(),
        vec![
            parser::Expr::List(vec![
                parser::Expr::Number(10.0, None),
                parser::Expr::Number(20.0, None),
                parser::Expr::Number(30.0, None),
            ]),
            parser::Expr::Number(20.0, None),
        ],
    ));
    assert_eq!(result, Value::Number(value::Number::unitless(2.0)));
}

#[test]
fn list_separator() {
    let result = eval(parser::Expr::Call(
        "list.separator".to_string(),
        vec![parser::Expr::List(vec![
            parser::Expr::Number(1.0, None),
            parser::Expr::Number(2.0, None),
        ])],
    ));
    match &result {
        Value::String(s, _) => {
            assert_eq!(s, "space");
        }
        _ => panic!("expected string"),
    }
}

#[test]
fn list_set_nth() {
    let result = eval(parser::Expr::Call(
        "list.set-nth".to_string(),
        vec![
            parser::Expr::List(vec![
                parser::Expr::Number(1.0, None),
                parser::Expr::Number(2.0, None),
                parser::Expr::Number(3.0, None),
            ]),
            parser::Expr::Number(2.0, None),
            parser::Expr::Number(99.0, None),
        ],
    ));
    assert_eq!(
        result,
        Value::List(
            vec![
                Value::Number(value::Number::unitless(1.0)),
                Value::Number(value::Number::unitless(99.0)),
                Value::Number(value::Number::unitless(3.0)),
            ],
            value::Separator::Space,
        )
    );
}

#[test]
fn list_zip() {
    let result = eval(parser::Expr::Call(
        "list.zip".to_string(),
        vec![
            parser::Expr::List(vec![
                parser::Expr::Number(1.0, None),
                parser::Expr::Number(2.0, None),
            ]),
            parser::Expr::List(vec![
                parser::Expr::Number(3.0, None),
                parser::Expr::Number(4.0, None),
            ]),
        ],
    ));
    match &result {
        Value::List(items, _) => {
            assert_eq!(items.len(), 2); // 2 zipped sublists.
        }
        _ => panic!("expected list"),
    }
}

// ============================================================
// sass:map module tests
// ============================================================

#[test]
fn map_get() {
    let result = eval(parser::Expr::Call(
        "map.get".to_string(),
        vec![
            parser::Expr::Map(vec![(
                parser::Expr::String("key".to_string()),
                parser::Expr::Number(42.0, None),
            )]),
            parser::Expr::String("key".to_string()),
        ],
    ));
    assert_eq!(result, Value::Number(value::Number::unitless(42.0)));
}

#[test]
fn map_get_missing() {
    let result = eval(parser::Expr::Call(
        "map.get".to_string(),
        vec![
            parser::Expr::Map(vec![(
                parser::Expr::String("key".to_string()),
                parser::Expr::Number(42.0, None),
            )]),
            parser::Expr::String("missing".to_string()),
        ],
    ));
    assert_eq!(result, Value::Null);
}

#[test]
fn map_merge() {
    let result = eval(parser::Expr::Call(
        "map.merge".to_string(),
        vec![
            parser::Expr::Map(vec![(
                parser::Expr::String("a".to_string()),
                parser::Expr::Number(1.0, None),
            )]),
            parser::Expr::Map(vec![(
                parser::Expr::String("b".to_string()),
                parser::Expr::Number(2.0, None),
            )]),
        ],
    ));
    match &result {
        Value::Map(entries) => {
            assert_eq!(entries.len(), 2);
        }
        _ => panic!("expected map"),
    }
}

#[test]
fn map_keys() {
    let result = eval(parser::Expr::Call(
        "map.keys".to_string(),
        vec![parser::Expr::Map(vec![
            (
                parser::Expr::String("a".to_string()),
                parser::Expr::Number(1.0, None),
            ),
            (
                parser::Expr::String("b".to_string()),
                parser::Expr::Number(2.0, None),
            ),
        ])],
    ));
    match &result {
        Value::List(items, _) => {
            assert_eq!(items.len(), 2);
        }
        _ => panic!("expected list"),
    }
}

#[test]
fn map_values() {
    let result = eval(parser::Expr::Call(
        "map.values".to_string(),
        vec![parser::Expr::Map(vec![
            (
                parser::Expr::String("a".to_string()),
                parser::Expr::Number(1.0, None),
            ),
            (
                parser::Expr::String("b".to_string()),
                parser::Expr::Number(2.0, None),
            ),
        ])],
    ));
    match &result {
        Value::List(items, _) => {
            assert_eq!(items.len(), 2);
        }
        _ => panic!("expected list"),
    }
}

#[test]
fn map_has_key() {
    let result = eval(parser::Expr::Call(
        "map.has-key".to_string(),
        vec![
            parser::Expr::Map(vec![(
                parser::Expr::String("key".to_string()),
                parser::Expr::Number(1.0, None),
            )]),
            parser::Expr::String("key".to_string()),
        ],
    ));
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn map_has_key_missing() {
    let result = eval(parser::Expr::Call(
        "map.has-key".to_string(),
        vec![
            parser::Expr::Map(vec![(
                parser::Expr::String("key".to_string()),
                parser::Expr::Number(1.0, None),
            )]),
            parser::Expr::String("missing".to_string()),
        ],
    ));
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn map_remove() {
    let result = eval(parser::Expr::Call(
        "map.remove".to_string(),
        vec![
            parser::Expr::Map(vec![
                (
                    parser::Expr::String("a".to_string()),
                    parser::Expr::Number(1.0, None),
                ),
                (
                    parser::Expr::String("b".to_string()),
                    parser::Expr::Number(2.0, None),
                ),
            ]),
            parser::Expr::String("a".to_string()),
        ],
    ));
    match &result {
        Value::Map(entries) => {
            assert_eq!(entries.len(), 1);
        }
        _ => panic!("expected map"),
    }
}

#[test]
fn map_set() {
    let result = eval(parser::Expr::Call(
        "map.set".to_string(),
        vec![
            parser::Expr::Map(vec![(
                parser::Expr::String("a".to_string()),
                parser::Expr::Number(1.0, None),
            )]),
            parser::Expr::String("a".to_string()),
            parser::Expr::Number(99.0, None),
        ],
    ));
    match &result {
        Value::Map(entries) => {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].1, Value::Number(value::Number::unitless(99.0)));
        }
        _ => panic!("expected map"),
    }
}

// ============================================================
// sass:string module tests
// ============================================================

#[test]
fn string_unquote() {
    let result = eval(parser::Expr::Call(
        "string.unquote".to_string(),
        vec![parser::Expr::String("hello".to_string())],
    ));
    assert_eq!(
        result,
        Value::String("hello".to_string(), value::Quoted::Unquoted)
    );
}

#[test]
fn string_quote() {
    let result = eval(parser::Expr::Call(
        "string.quote".to_string(),
        vec![parser::Expr::String("hello".to_string())],
    ));
    assert_eq!(
        result,
        Value::String("hello".to_string(), value::Quoted::Quoted)
    );
}

#[test]
fn string_length() {
    let result = eval(parser::Expr::Call(
        "string.length".to_string(),
        vec![parser::Expr::String("hello".to_string())],
    ));
    assert_eq!(result, Value::Number(value::Number::unitless(5.0)));
}

#[test]
fn string_index() {
    let result = eval(parser::Expr::Call(
        "string.index".to_string(),
        vec![
            parser::Expr::String("hello world".to_string()),
            parser::Expr::String("world".to_string()),
        ],
    ));
    assert_eq!(result, Value::Number(value::Number::unitless(7.0)));
}

#[test]
fn string_upper_case() {
    let result = eval(parser::Expr::Call(
        "string.upper-case".to_string(),
        vec![parser::Expr::String("hello".to_string())],
    ));
    assert_eq!(
        result,
        Value::String("HELLO".to_string(), value::Quoted::Quoted)
    );
}

#[test]
fn string_lower_case() {
    let result = eval(parser::Expr::Call(
        "string.lower-case".to_string(),
        vec![parser::Expr::String("HELLO".to_string())],
    ));
    assert_eq!(
        result,
        Value::String("hello".to_string(), value::Quoted::Quoted)
    );
}

#[test]
fn string_slice() {
    let result = eval(parser::Expr::Call(
        "string.slice".to_string(),
        vec![
            parser::Expr::String("hello world".to_string()),
            parser::Expr::Number(1.0, None),
            parser::Expr::Number(5.0, None),
        ],
    ));
    assert_eq!(
        result,
        Value::String("hello".to_string(), value::Quoted::Quoted)
    );
}

#[test]
fn string_insert() {
    let result = eval(parser::Expr::Call(
        "string.insert".to_string(),
        vec![
            parser::Expr::String("hello".to_string()),
            parser::Expr::String(" world".to_string()),
            parser::Expr::Number(6.0, None),
        ],
    ));
    assert_eq!(
        result,
        Value::String("hello world".to_string(), value::Quoted::Quoted)
    );
}

#[test]
fn string_unique_id() {
    let result = eval(parser::Expr::Call("string.unique-id".to_string(), vec![]));
    match &result {
        Value::String(s, _) => {
            assert!(s.starts_with('u'), "unique id should start with 'u'");
        }
        _ => panic!("expected string"),
    }
}

// ============================================================
// sass:meta module tests
// ============================================================

#[test]
fn meta_type_of_number() {
    let result = eval(parser::Expr::Call(
        "meta.type-of".to_string(),
        vec![parser::Expr::Number(42.0, None)],
    ));
    assert_eq!(
        result,
        Value::String("number".to_string(), value::Quoted::Quoted)
    );
}

#[test]
fn meta_type_of_string() {
    let result = eval(parser::Expr::Call(
        "meta.type-of".to_string(),
        vec![parser::Expr::String("hello".to_string())],
    ));
    assert_eq!(
        result,
        Value::String("string".to_string(), value::Quoted::Quoted)
    );
}

#[test]
fn meta_type_of_bool() {
    let result = eval(parser::Expr::Call(
        "meta.type-of".to_string(),
        vec![parser::Expr::Boolean(true)],
    ));
    assert_eq!(
        result,
        Value::String("bool".to_string(), value::Quoted::Quoted)
    );
}

#[test]
fn meta_type_of_null() {
    let result = eval(parser::Expr::Call(
        "meta.type-of".to_string(),
        vec![parser::Expr::Null],
    ));
    assert_eq!(
        result,
        Value::String("null".to_string(), value::Quoted::Quoted)
    );
}

#[test]
fn meta_type_of_color() {
    let result = eval(parser::Expr::Call(
        "meta.type-of".to_string(),
        vec![parser::Expr::Color(0xFF0000)],
    ));
    assert_eq!(
        result,
        Value::String("color".to_string(), value::Quoted::Quoted)
    );
}

#[test]
fn meta_type_of_list() {
    let result = eval(parser::Expr::Call(
        "meta.type-of".to_string(),
        vec![parser::Expr::List(vec![parser::Expr::Number(1.0, None)])],
    ));
    assert_eq!(
        result,
        Value::String("list".to_string(), value::Quoted::Quoted)
    );
}

#[test]
fn meta_type_of_map() {
    let result = eval(parser::Expr::Call(
        "meta.type-of".to_string(),
        vec![parser::Expr::Map(vec![(
            parser::Expr::String("k".to_string()),
            parser::Expr::Number(1.0, None),
        )])],
    ));
    assert_eq!(
        result,
        Value::String("map".to_string(), value::Quoted::Quoted)
    );
}

#[test]
fn meta_content_exists() {
    let result = eval(parser::Expr::Call("meta.content-exists".to_string(), vec![]));
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn meta_function_exists_builtin() {
    let result = eval(parser::Expr::Call(
        "meta.function-exists".to_string(),
        vec![parser::Expr::String("unquote".to_string())],
    ));
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn meta_function_exists_missing() {
    let result = eval(parser::Expr::Call(
        "meta.function-exists".to_string(),
        vec![parser::Expr::String("nonexistent_func".to_string())],
    ));
    assert_eq!(result, Value::Boolean(false));
}
