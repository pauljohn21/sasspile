#![allow(
    clippy::many_single_char_names,
    clippy::single_char_pattern,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
//! Color 内建函数（invert/grayscale/color-channel 等）。
//!
//! 包含 invert/grayscale/color-channel/ie-hex-str 及 Level 4 函数
//! (is-powerless/is-missing/is-in-gamut/is-legacy/channel/to-space/to-gamut/space/same)。
//! 注意：hwb/hsl/hsla/adjust-hue/saturate/desaturate/transparentize/opacify/alpha/red/green/blue/hue/saturation/lightness
//! 等 HSL/HWB 函数已拆分到 color_hwb_hsl.rs。

use super::super::Evaluator;
use crate::error::{Result, SassError};
use crate::parse::ast::{Color, ColorOutput, ColorSpace, Value};
use imbl::HashMap;

pub fn call(name: &str, args: &[Value], kw_args: &HashMap<String, Value>) -> Result<Option<Value>> {
    match name {
        "invert" => {
            let color_arg = args.first().or_else(|| kw_args.get("color"));
            let space_arg = kw_args.get("space");
            match color_arg {
                Some(Value::Color(c)) => {
                    match c.space.is_legacy() {
                        true => {
                            // Legacy invert：RGB 反转（255 - channel），输出 Auto（查找命名色）
                            let r = 255.0 - c.legacy_rgb[0];
                            let g = 255.0 - c.legacy_rgb[1];
                            let b = 255.0 - c.legacy_rgb[2];
                            Ok(Some(Value::Color(Color::with_rgb(
                                r, g, b, c.a,
                                ColorSpace::Rgb,
                                ColorOutput::Auto,
                            ))))
                        }
                        false => {
                            // 现代空间 invert：各通道 1 - channel
                            // 如果指定了 $space，先转到该空间
                            let (r, g, b) = match space_arg {
                                Some(Value::String(s, _)) => {
                                    let converted = super::color_conv_ops::convert_space(c, s)?;
                                    match converted {
                                        Value::Color(cc) => (cc.channels[0], cc.channels[1], cc.channels[2]),
                                        _ => (c.channels[0], c.channels[1], c.channels[2]),
                                    }
                                }
                                _ => (c.channels[0], c.channels[1], c.channels[2]),
                            };
                            let target_space = match space_arg {
                                Some(Value::String(s, _)) => {
                                    ColorSpace::from_str(s).unwrap_or(c.space)
                                }
                                _ => c.space,
                            };
                            Ok(Some(Value::Color(Color::with_space(
                                target_space,
                                [1.0 - r, 1.0 - g, 1.0 - b],
                                c.a,
                                c.output,
                                [0.0, 0.0, 0.0],
                            ))))
                        }
                    }
                }
                // CSS 滤镜函数透传：invert(number) 非颜色参数
                _ if !args.is_empty() => {
                    let arg_str = args
                        .iter()
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ");
                    Ok(Some(Value::String(format!("invert({arg_str})"), false)))
                }
                _ => Err(SassError::Eval("invert requires 1 argument".into())),
            }
        }
        "grayscale" => {
            let color_arg = args.first().or_else(|| kw_args.get("color"));
            match color_arg {
                Some(Value::Color(c)) => {
                    let (h, _s, l) =
                        Evaluator::rgb_to_hsl(c.legacy_rgb[0], c.legacy_rgb[1], c.legacy_rgb[2]);
                    let new_c = Evaluator::hsl_to_rgb(h, 0.0, l);
                    Ok(Some(Value::Color(Color::with_rgb(
                        new_c.legacy_rgb[0],
                        new_c.legacy_rgb[1],
                        new_c.legacy_rgb[2],
                        c.a,
                        ColorSpace::Rgb,
                        ColorOutput::Auto,
                    ))))
                }
                _ if !args.is_empty() => {
                    let arg_str = args
                        .iter()
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ");
                    Ok(Some(Value::String(format!("grayscale({arg_str})"), false)))
                }
                _ => Err(SassError::Eval("grayscale requires 1 argument".into())),
            }
        }
        "color-channel" => match args {
            [Value::Color(c), Value::String(ch, _)] => match ch.as_str() {
                "red" => Ok(Some(Value::Number(c.legacy_rgb[0], None))),
                "green" => Ok(Some(Value::Number(c.legacy_rgb[1], None))),
                "blue" => Ok(Some(Value::Number(c.legacy_rgb[2], None))),
                "alpha" => Ok(Some(Value::Number(c.a, None))),
                _ => Err(SassError::Eval(format!("Unknown color channel: {ch}"))),
            },
            _ => Err(SassError::Eval(
                "color-channel 需要 (color, channel) 参数".into(),
            )),
        },
        "adjust-color" => super::color_adjust::adjust_color(args, kw_args).map(Some),
        "change-color" => super::color_change::change_color(args, kw_args).map(Some),
        "scale-color" => super::color_scale::scale_color(args, kw_args).map(Some),
        // ── sass:color 模块函数（Level 4 颜色空间支持）──
        "is-powerless" | "is-missing" | "is-in-gamut" | "is-legacy" => {
            super::color_inspect::call(name, args, kw_args)
        }
        "channel" => super::color_space::channel(args, kw_args),
        "to-space" => super::color_space::to_space(args, kw_args),
        "to-gamut" => super::color_gamut::to_gamut(args, kw_args),
        "space" => super::color_space::space(args, kw_args),
        "same" => super::color_space::same(args, kw_args),
        "ie-hex-str" => {
            let color_arg = args.first().or_else(|| kw_args.get("color"));
            match color_arg {
                Some(Value::Color(c)) => {
                    let alpha = (c.a * 255.0).round() as u8;
                    let r = c.legacy_rgb[0] as u8;
                    let g = c.legacy_rgb[1] as u8;
                    let b = c.legacy_rgb[2] as u8;
                    Ok(Some(Value::String(
                        format!("#{alpha:02X}{r:02X}{g:02X}{b:02X}"),
                        false,
                    )))
                }
                Some(v) => Err(SassError::Eval(format!("$color: {v} is not a color."))),
                None => Err(SassError::Eval("Missing argument $color.".into())),
            }
        }
        // HWB 函数转发到 color_hwb 模块
        "hwb" => return super::color_hwb::call_hwb(args, kw_args),
        "whiteness" => return super::color_hwb::call_whiteness(args, kw_args),
        "blackness" => return super::color_hwb::call_blackness(args, kw_args),
        // HSL/通道操作函数转发到 color_hwb_hsl 模块
        "complement" | "hsl" | "hsla"
        | "adjust-hue" | "saturate" | "desaturate" | "transparentize" | "fade-out"
        | "opacify" | "fade-in" | "alpha" | "opacity" | "red" | "green" | "blue"
        | "hue" | "saturation" | "lightness" => super::color_hwb_hsl::call(name, args, kw_args),
        _ => Ok(None),
    }
}
