//! calc 表达式单位兼容性表 + 转换。
//!
//! 基于 CSS Values 4 规范的单位等价关系。

/// 判断两个单位是否兼容（可以相互转换）。
///
/// 兼容组：
/// - 长度：px, em, rem, cm, mm, in, pt, pc, q, vw, vh, vmin, vmax
/// - 角度：deg, rad, grad, turn
/// - 时间：s, ms
/// - 频率：hz, khz
/// - 分辨率：dpi, dpcm, dppx
pub fn units_compatible(a: &str, b: &str) -> bool {
    match a == b {
        true => return true,
        false => {}
    }
    let group_a = unit_group(a);
    let group_b = unit_group(b);
    group_a.is_some() && group_a == group_b
}

/// 获取单位所属的兼容组。
fn unit_group(unit: &str) -> Option<UnitGroup> {
    match unit.to_lowercase().as_str() {
        "px" | "em" | "rem" | "cm" | "mm" | "in" | "pt" | "pc" | "q" | "vw" | "vh"
        | "vmin" | "vmax" => Some(UnitGroup::Length),
        "deg" | "rad" | "grad" | "turn" => Some(UnitGroup::Angle),
        "s" | "ms" => Some(UnitGroup::Time),
        "hz" | "khz" => Some(UnitGroup::Frequency),
        "dpi" | "dpcm" | "dppx" => Some(UnitGroup::Resolution),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnitGroup {
    Length,
    Angle,
    Time,
    Frequency,
    Resolution,
}

/// 将 `value` 从 `from_unit` 转换为 `to_unit`。
///
/// 返回 None 如果单位不兼容。
/// 转换基于 CSS 参考像素（96px = 1in）等标准比例。
pub fn convert_unit(value: f64, from_unit: &str, to_unit: &str) -> Option<f64> {
    match from_unit == to_unit {
        true => return Some(value),
        false => {}
    }
    match units_compatible(from_unit, to_unit) {
        false => return None,
        true => {}
    }
    // 先转为基础单位，再从基础单位转到目标
    let in_base = to_base_unit(value, from_unit)?;
    from_base_unit(in_base, to_unit)
}

/// 将值转换为基础单位（组内的参考单位）。
fn to_base_unit(value: f64, unit: &str) -> Option<f64> {
    match unit.to_lowercase().as_str() {
        // 长度 → px（基础单位）
        "px" => Some(value),
        "in" => Some(value * 96.0),
        "cm" => Some(value * 96.0 / 2.54),
        "mm" => Some(value * 96.0 / 25.4),
        "pt" => Some(value * 96.0 / 72.0),
        "pc" => Some(value * 96.0 / 6.0),
        "q" => Some(value * 96.0 / 254.0),
        "em" | "rem" | "vw" | "vh" | "vmin" | "vmax" => Some(value), // 相对单位不转换
        // 角度 → deg（基础单位）
        "deg" => Some(value),
        "rad" => Some(value.to_degrees()),
        "grad" => Some(value * 360.0 / 400.0),
        "turn" => Some(value * 360.0),
        // 时间 → s（基础单位）
        "s" => Some(value),
        "ms" => Some(value / 1000.0),
        // 频率 → hz
        "hz" => Some(value),
        "khz" => Some(value * 1000.0),
        // 分辨率 → dpi
        "dpi" => Some(value),
        "dpcm" => Some(value * 2.54),
        "dppx" => Some(value * 96.0),
        _ => None,
    }
}

/// 将基础单位的值转换为目标单位。
fn from_base_unit(value: f64, unit: &str) -> Option<f64> {
    match unit.to_lowercase().as_str() {
        "px" => Some(value),
        "in" => Some(value / 96.0),
        "cm" => Some(value * 2.54 / 96.0),
        "mm" => Some(value * 25.4 / 96.0),
        "pt" => Some(value * 72.0 / 96.0),
        "pc" => Some(value * 6.0 / 96.0),
        "q" => Some(value * 254.0 / 96.0),
        "em" | "rem" | "vw" | "vh" | "vmin" | "vmax" => Some(value),
        "deg" => Some(value),
        "rad" => Some(value.to_radians()),
        "grad" => Some(value * 400.0 / 360.0),
        "turn" => Some(value / 360.0),
        "s" => Some(value),
        "ms" => Some(value * 1000.0),
        "hz" => Some(value),
        "khz" => Some(value / 1000.0),
        "dpi" => Some(value),
        "dpcm" => Some(value / 2.54),
        "dppx" => Some(value / 96.0),
        _ => None,
    }
}
