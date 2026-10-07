use rx_scss::types::Value;

#[test]
fn display_number_integer() {
    assert_eq!(Value::Number(42.0, None).to_string(), "42");
}

#[test]
fn display_number_float() {
    let s = Value::Number(3.5, None).to_string();
    assert!(s.contains("3.5"), "expected '3.5' in '{}'", s);
}

#[test]
fn display_string() {
    assert_eq!(Value::String("hello".into()).to_string(), "hello");
}

#[test]
fn display_color_rgb() {
    assert_eq!(Value::Color(255, 0, 0, 255).to_string(), "#ff0000");
}

#[test]
fn display_color_rgba() {
    let s = Value::Color(255, 0, 0, 128).to_string();
    assert_eq!(s, "#ff000080");
}

#[test]
fn display_bool() {
    assert_eq!(Value::Bool(true).to_string(), "true");
    assert_eq!(Value::Bool(false).to_string(), "false");
}

#[test]
fn display_null() {
    assert_eq!(Value::Null.to_string(), "null");
}

#[test]
fn display_list() {
    let list = Value::List(vec![
        Value::Number(1.0, None),
        Value::Number(2.0, None),
        Value::Number(3.0, None),
    ]);
    let s = list.to_string();
    assert!(s.contains("1"));
    assert!(s.contains("2"));
    assert!(s.contains("3"));
}

#[test]
fn display_map() {
    let map = Value::Map(vec![
        ("key1".into(), Value::Number(1.0, None)),
        ("key2".into(), Value::String("v".into())),
    ]);
    let s = map.to_string();
    assert!(s.contains("key1"));
    assert!(s.contains("key2"));
}

#[test]
fn value_partial_eq() {
    assert_eq!(Value::Number(1.0, None), Value::Number(1.0, None));
    assert_ne!(Value::Number(1.0, None), Value::Number(2.0, None));
    assert_eq!(Value::Bool(true), Value::Bool(true));
    assert_eq!(Value::Null, Value::Null);
    assert_ne!(Value::Number(1.0, None), Value::String("1".into()));
}

#[test]
fn value_color_eq() {
    assert_eq!(Value::Color(1, 2, 3, 4), Value::Color(1, 2, 3, 4));
    assert_ne!(Value::Color(1, 2, 3, 4), Value::Color(1, 2, 3, 5));
}
