#![allow(
    clippy::many_single_char_names,
    clippy::single_char_pattern,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::items_after_statements
)]
//! `hwb()` 构造函数 + whiteness/blackness 通道提取。

use super::super::Evaluator;
use super::color_hwb_hsl::{angle_to_deg, flatten_space_list, format_hwb_passthrough, merge_named_color_args};
use super::color_parse::extract_calc_f64;
use crate::error::{Result, SassError};
use crate::parse::ast::{Color, Value};
use imbl::HashMap;

/// `hwb($hue, $whiteness, $blackness, $alpha?)` — HWB 颜色构造。
pub fn call_hwb(args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Option<Value>> {
    // 合并命名参数到位置参数
    let merged = merge_named_color_args(args, kw_args, &["hue", "whiteness", "blackness", "alpha"]);
    // 展开空格分隔的 List（CSS hwb() 语法：hwb(0deg 30% 40%)）
    let flat = flatten_space_list(&merged);

    // 解析所有通道：参数不足或任何通道不可解析时透传
    if flat.len() < 3 {
        return Ok(Some(Value::String(format_hwb_passthrough(&flat), false)));
    }

    // 用 ? 传播管道：每个步骤失败时透传原始字符串，成功时 ? 取值
    let passthrough = || format_hwb_passthrough(&flat);
    let (h_parsed, h_unit, h_is_none_kw) = match extract_hue(&flat[0])? {
        Some(tuple) => tuple,
        None => return Ok(Some(Value::String(passthrough(), false))),
    };
    let w_parsed = match extract_channel(&flat[1])? {
        Some(c) => c,
        None => return Ok(Some(Value::String(passthrough(), false))),
    };
    let bk_parsed = match extract_channel(&flat[2])? {
        Some(c) => c,
        None => return Ok(Some(Value::String(passthrough(), false))),
    };

    // alpha 透传：有 4 参数且不可解析时透传
    let a_parsed = match flat.len() {
        4 => match extract_alpha(&flat[3])? {
            Some(n) => n,
            None => return Ok(Some(Value::String(passthrough(), false))),
        },
        _ => 1.0,
    };

    let h = normalize_hue(h_parsed, h_unit.as_ref());
    // h_is_nan 存储为 NaN 的条件：只有 none 关键字才保持 HWB 格式
    let h_store_nan = h_is_none_kw && h_parsed.is_nan();
    Ok(Some(build_hwb(h, h_store_nan, w_parsed, bk_parsed, a_parsed)))
}

/// `whiteness($color)` — 提取颜色的白度通道。
pub fn call_whiteness(args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Option<Value>> {
    let color_arg = args.first().or_else(|| kw_args.get("color"));
    match color_arg {
        Some(Value::Color(c)) => {
            let w = c.legacy_rgb[0].min(c.legacy_rgb[1]).min(c.legacy_rgb[2]) / 255.0 * 100.0;
            Ok(Some(Value::Number(w, Some("%".to_string()))))
        }
        _ => Err(SassError::Eval(
            "whiteness requires 1 color argument".into(),
        )),
    }
}

/// `blackness($color)` — 提取颜色的黑度通道。
pub fn call_blackness(args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Option<Value>> {
    let color_arg = args.first().or_else(|| kw_args.get("color"));
    match color_arg {
        Some(Value::Color(c)) => {
            let b = (1.0 - c.legacy_rgb[0].max(c.legacy_rgb[1]).max(c.legacy_rgb[2]) / 255.0) * 100.0;
            Ok(Some(Value::Number(b, Some("%".to_string()))))
        }
        _ => Err(SassError::Eval(
            "blackness requires 1 color argument".into(),
        )),
    }
}

// ── 内部类型与函数 ──────────────────────────────────────────────────

/// 颜色通道值提取结果——区分数值、missing、不可解析。
/// 用于 HWB whiteness/blackness/none 语义：`none` 表示通道缺失，
/// `Number(NaN)` 表示计算得到的数值 NaN（传播计算）。
#[derive(Clone, Copy)]
enum ChannelVal {
    Number(f64),
    Missing,
}

/// 尝试从 Value 提取通道数值。
fn extract_channel(v: &Value) -> Result<Option<ChannelVal>> {
    match v {
        Value::String(s, false) if s == "none" => Ok(Some(ChannelVal::Missing)),
        Value::Number(n, Some(u)) if u == "%" => Ok(Some(ChannelVal::Number(*n / 100.0))),
        Value::Number(n, _) => Ok(Some(ChannelVal::Number(
            if *n == 0.0 { 0.0 } else { *n },
        ))),
        _ => match extract_calc_f64(&v.to_string()) {
            Some(n) => Ok(Some(ChannelVal::Number(n))),
            None => Ok(None),
        },
    }
}

/// 从 Value 提取 alpha 值（支持 % 单位：100% → 1.0，以及特殊值 clamp）。
fn extract_alpha(v: &Value) -> Result<Option<f64>> {
    match v {
        Value::Number(n, Some(u)) if u == "%" => {
            Ok(Some((*n / 100.0).clamp(0.0, 1.0)))
        }
        Value::Number(n, _) => Ok(Some(n.clamp(0.0, 1.0))),
        _ => match extract_calc_f64(&v.to_string()) {
            Some(f64::INFINITY) => Ok(Some(1.0)),
            Some(f64::NEG_INFINITY) => Ok(Some(0.0)),
            Some(n) if n.is_nan() => Ok(Some(0.0)),
            Some(n) => Ok(Some(n.clamp(0.0, 1.0))),
            None => Ok(None),
        },
    }
}

/// 构建 HWB Color（计算 RGB + 存储通道值）。
fn build_hwb(h: f64, h_is_nan: bool, w_ch: ChannelVal, bk_ch: ChannelVal, a: f64) -> Value {
    let (w_compute, bk_compute) = match (w_ch, bk_ch) {
        (ChannelVal::Number(w), _) if w.is_infinite() && w > 0.0 => (0.0, 0.0),
        (_, ChannelVal::Number(bk)) if bk.is_infinite() && bk > 0.0 => (0.0, 0.0),
        (ChannelVal::Number(w), _) if w.is_infinite() && w < 0.0 => (0.0, 1.0),
        (_, ChannelVal::Number(bk)) if bk.is_infinite() && bk < 0.0 => (0.0, 1.0),
        (ChannelVal::Number(w), ChannelVal::Number(bk)) => {
            (if w.is_nan() { 0.0 } else { w }, if bk.is_nan() { 0.0 } else { bk })
        }
        (w, bk) => {
            let w = match w {
                ChannelVal::Number(n) if n.is_nan() => 0.0,
                ChannelVal::Number(n) => n,
                ChannelVal::Missing => 0.0,
            };
            let bk = match bk {
                ChannelVal::Number(n) if n.is_nan() => 0.0,
                ChannelVal::Number(n) => n,
                ChannelVal::Missing => 0.0,
            };
            (w, bk)
        }
    };
    let (w_store, bk_store) = match (w_ch, bk_ch) {
        (ChannelVal::Missing, ChannelVal::Missing) => (f64::NAN, f64::NAN),
        (ChannelVal::Missing, _) => (f64::NAN, bk_compute),
        (_, ChannelVal::Missing) => (w_compute, f64::NAN),
        _ => (w_compute, bk_compute),
    };
    let h_store = if h_is_nan { f64::NAN } else { h };
    let c = Evaluator::hwb_to_rgb(h, w_compute, bk_compute, a);
    Value::Color(Color::with_hwb(h_store, w_store, bk_store, a, c.legacy_rgb))
}

/// 解析 hue，不可解析时返回 None（调用方决定透传）。
fn extract_hue(v: &Value) -> Result<Option<(f64, Option<String>, bool)>> {
    match v {
        Value::String(s, false) if s == "none" => Ok(Some((f64::NAN, None, true))),
        Value::Number(n, unit) => Ok(Some((*n, unit.clone(), false))),
        _ => match extract_calc_f64(&v.to_string()) {
            Some(n) => Ok(Some((n, None, false))),
            None => Ok(None),
        },
    }
}

/// hue: 非有限值 normalize 到 0。
fn normalize_hue(h: f64, unit: Option<&String>) -> f64 {
    match h.is_finite() {
        true => angle_to_deg(h, unit),
        false => 0.0,
    }
}
