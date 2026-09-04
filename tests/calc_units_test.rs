//! calc 单位兼容性 + 转换测试。

use sasspile::eval::value::calc_units::{convert_unit, units_compatible};

#[test]
fn test_same_unit_compatible() {
    assert!(units_compatible("px", "px"));
    assert!(units_compatible("deg", "deg"));
}

#[test]
fn test_incompatible_units() {
    assert!(!units_compatible("px", "deg"));
    assert!(!units_compatible("s", "px"));
}

#[test]
fn test_length_units_compatible() {
    assert!(units_compatible("px", "em"));
    assert!(units_compatible("px", "pt"));
    assert!(units_compatible("cm", "mm"));
    assert!(units_compatible("in", "pt"));
}

#[test]
fn test_angle_units_compatible() {
    assert!(units_compatible("deg", "rad"));
    assert!(units_compatible("deg", "grad"));
    assert!(units_compatible("deg", "turn"));
}

#[test]
fn test_time_units_compatible() {
    assert!(units_compatible("s", "ms"));
}

#[test]
fn test_convert_deg_to_rad() {
    let result = convert_unit(180.0, "deg", "rad");
    assert!(result.is_some());
    let r = result.unwrap();
    assert!((r - std::f64::consts::PI).abs() < 1e-6);
}

#[test]
fn test_convert_pt_to_px() {
    let result = convert_unit(72.0, "pt", "px");
    assert!(result.is_some());
    let r = result.unwrap();
    assert!((r - 96.0).abs() < 1e-6);
}

#[test]
fn test_convert_ms_to_s() {
    let result = convert_unit(500.0, "ms", "s");
    assert!(result.is_some());
    let r = result.unwrap();
    assert!((r - 0.5).abs() < 1e-6);
}

#[test]
fn test_convert_incompatible() {
    assert_eq!(convert_unit(1.0, "px", "deg"), None);
}
