//! Value system tests — tests Number, String, Color, List, Map, Bool, Null.

use sasspile::value::{Number, SassString, Color, SassList, SassMap, Value};
use sasspile::ast::ListSeparator;

#[test]
fn test_number_basic() {
    let n = Number::new(42.0, Some("px".to_string()));
    assert_eq!(n.value, 42.0);
    assert_eq!(n.unit.as_deref(), Some("px"));
    assert!(!n.is_unitless());
}

#[test]
fn test_number_unitless() {
    let n = Number::unitless(10.0);
    assert!(n.is_unitless());
    assert_eq!(n.unit_str(), "");
}

#[test]
fn test_number_add_same_unit() {
    let a = Number::new(10.0, Some("px".to_string()));
    let b = Number::new(20.0, Some("px".to_string()));
    let c = a.add(&b).unwrap();
    assert_eq!(c.value, 30.0);
    assert_eq!(c.unit.as_deref(), Some("px"));
}

#[test]
fn test_number_add_unitless() {
    let a = Number::unitless(10.0);
    let b = Number::new(20.0, Some("px".to_string()));
    let c = a.add(&b).unwrap();
    assert_eq!(c.value, 30.0);
    assert_eq!(c.unit.as_deref(), Some("px"));
}

#[test]
fn test_number_add_incompatible_units() {
    let a = Number::new(10.0, Some("px".to_string()));
    let b = Number::new(20.0, Some("em".to_string()));
    assert!(a.add(&b).is_err());
}

#[test]
fn test_number_mul_unitless() {
    let a = Number::new(10.0, Some("px".to_string()));
    let b = Number::unitless(2.0);
    let c = a.mul(&b);
    assert_eq!(c.value, 20.0);
    assert_eq!(c.unit.as_deref(), Some("px"));
}

#[test]
fn test_number_mul_same_units_cancel() {
    let a = Number::new(10.0, Some("px".to_string()));
    let b = Number::new(2.0, Some("px".to_string()));
    let c = a.mul(&b);
    assert_eq!(c.value, 20.0);
    assert!(c.unit.is_none());
}

#[test]
fn test_number_to_css_integer() {
    let n = Number::new(42.0, Some("px".to_string()));
    assert_eq!(n.to_css_string(), "42px");
}

#[test]
fn test_number_to_css_decimal() {
    let n = Number::new(3.14, None);
    assert_eq!(n.to_css_string(), "3.14");
}

#[test]
fn test_string_quoted() {
    let s = SassString::quoted("hello");
    assert!(s.quoted);
    assert_eq!(s.value, "hello");
}

#[test]
fn test_string_unquoted() {
    let s = SassString::unquoted("hello");
    assert!(!s.quoted);
    assert_eq!(s.value, "hello");
}

#[test]
fn test_color_rgb() {
    let c = Color::rgb(255.0, 128.0, 0.0, 1.0);
    assert_eq!(c.red(), 255.0);
    assert_eq!(c.green(), 128.0);
    assert_eq!(c.blue(), 0.0);
    assert_eq!(c.alpha(), 1.0);
}

#[test]
fn test_color_to_hex() {
    let c = Color::rgb(255.0, 255.0, 255.0, 1.0);
    assert_eq!(c.to_hex(), "#fff");
    let c2 = Color::rgb(170.0, 187.0, 204.0, 1.0);
    assert_eq!(c2.to_hex(), "#abc");
}

#[test]
fn test_color_hsl_to_rgb() {
    let hsl = Color::hsl(0.0, 100.0, 50.0, 1.0);
    let rgb = hsl.to_rgb();
    assert!((rgb.red() - 255.0).abs() < 1.0);
    assert!((rgb.green() - 0.0).abs() < 1.0);
    assert!((rgb.blue() - 0.0).abs() < 1.0);
}

#[test]
fn test_color_rgb_to_hsl() {
    let rgb = Color::rgb(255.0, 0.0, 0.0, 1.0);
    let hsl = rgb.to_hsl();
    assert!((hsl.hue() - 0.0).abs() < 1.0);
    assert!((hsl.saturation() - 100.0).abs() < 1.0);
    assert!((hsl.lightness() - 50.0).abs() < 1.0);
}

#[test]
fn test_list_basic() {
    let items = vec![Value::Number(Number::unitless(1.0)), Value::Number(Number::unitless(2.0))];
    let list = SassList::new(items, ListSeparator::Space, false);
    assert_eq!(list.len(), 2);
    assert!(!list.is_empty());
}

#[test]
fn test_map_basic() {
    let mut map = SassMap::new();
    map.insert(Value::String(SassString::unquoted("a")), Value::Number(Number::unitless(1.0)));
    map.insert(Value::String(SassString::unquoted("b")), Value::Number(Number::unitless(2.0)));
    assert_eq!(map.len(), 2);
    assert!(map.has_key(&Value::String(SassString::unquoted("a"))));
}

#[test]
fn test_map_get() {
    let mut map = SassMap::new();
    map.insert(Value::Number(Number::unitless(1.0)), Value::Bool(true));
    let v = map.get(&Value::Number(Number::unitless(1.0)));
    assert!(v.is_some());
    assert_eq!(*v.unwrap(), Value::Bool(true));
}

#[test]
fn test_value_truthy() {
    assert!(Value::Bool(true).is_truthy());
    assert!(!Value::Bool(false).is_truthy());
    assert!(!Value::Null.is_truthy());
    assert!(Value::Number(Number::unitless(0.0)).is_truthy());
}

#[test]
fn test_value_type_name() {
    assert_eq!(Value::Null.type_name(), "null");
    assert_eq!(Value::Bool(true).type_name(), "bool");
    assert_eq!(Value::Number(Number::unitless(1.0)).type_name(), "number");
    assert_eq!(Value::String(SassString::quoted("x")).type_name(), "string");
}

#[test]
fn test_value_display() {
    assert_eq!(Value::Null.to_string(), "null");
    assert_eq!(Value::Bool(true).to_string(), "true");
    assert_eq!(Value::Bool(false).to_string(), "false");
    assert_eq!(Value::Number(Number::new(10.0, Some("px".to_string()))).to_string(), "10px");
}
