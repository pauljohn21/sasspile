use sasspile::parse::ast::*;

#[test]
fn test_number_display() {
    assert_eq!(Value::Number(10.0, None).to_string(), "10");
    assert_eq!(Value::Number(3.14, None).to_string(), "3.14");
    assert_eq!(Value::Number(10.0, Some("px".into())).to_string(), "10px");
    assert_eq!(Value::Number(50.0, Some("%".into())).to_string(), "50%");
}

#[test]
fn test_string_display() {
    assert_eq!(Value::String("red".into(), false).to_string(), "red");
    assert_eq!(Value::String("hello".into(), true).to_string(), "\"hello\"");
}

#[test]
fn test_color_display() {
    // 命名颜色反向查找：rgb(255,0,0) → "red", rgb(0,0,0) → "black"
    assert_eq!(Value::Color(Color::rgb(255, 0, 0)).to_string(), "red");
    assert_eq!(Value::Color(Color::rgb(0, 0, 0)).to_string(), "black");
    assert_eq!(
        Value::Color(Color::rgba(0, 0, 0, 0.5)).to_string(),
        "rgba(0, 0, 0, 0.5)"
    );
}

#[test]
fn test_list_display() {
    let list = Value::List(
        vec![
            Value::Number(1.0, None),
            Value::Number(2.0, None),
            Value::Number(3.0, None),
        ],
        Separator::Comma,
        false,
    );
    assert_eq!(list.to_string(), "1, 2, 3");

    let space_list = Value::List(
        vec![
            Value::String("a".into(), false),
            Value::String("b".into(), false),
        ],
        Separator::Space,
        false,
    );
    assert_eq!(space_list.to_string(), "a b");
}

#[test]
fn test_map_display() {
    let map = Value::Map(vec![
        (Value::String("a".into(), false), Value::Number(1.0, None)),
        (Value::String("b".into(), false), Value::Number(2.0, None)),
    ]);
    assert_eq!(map.to_string(), "(a: 1, b: 2)");
}

#[test]
fn test_bool_null_display() {
    assert_eq!(Value::Bool(true).to_string(), "true");
    assert_eq!(Value::Bool(false).to_string(), "false");
    assert_eq!(Value::Null.to_string(), "null");
}

#[test]
fn test_color_rgb() {
    let c = Color::rgb(255, 128, 0);
    assert_eq!(c.r, 255);
    assert_eq!(c.g, 128);
    assert_eq!(c.b, 0);
    assert!((c.a - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_color_rgba() {
    let c = Color::rgba(0, 0, 0, 0.5);
    assert_eq!(c.a, 0.5);
}
