use super::*;
use crate::error::Result;

impl Evaluator {
    /// CSS 命名颜色查找——将 "white" / "black" 等名称转为 Color。
    pub(crate) fn lookup_named_color(name: &str) -> Option<Color> {
        let (r, g, b) = match name {
            "aliceblue" => (240, 248, 255),
            "antiquewhite" => (250, 235, 215),
            "aqua" => (0, 255, 255),
            "aquamarine" => (127, 255, 212),
            "azure" => (240, 255, 255),
            "beige" => (245, 245, 220),
            "bisque" => (255, 228, 196),
            "black" => (0, 0, 0),
            "blanchedalmond" => (255, 235, 205),
            "blue" => (0, 0, 255),
            "blueviolet" => (138, 43, 226),
            "brown" => (165, 42, 42),
            "burlywood" => (222, 184, 135),
            "cadetblue" => (95, 158, 160),
            "chartreuse" => (127, 255, 0),
            "chocolate" => (210, 105, 30),
            "coral" => (255, 127, 80),
            "cornflowerblue" => (100, 149, 237),
            "cornsilk" => (255, 248, 220),
            "crimson" => (220, 20, 60),
            "cyan" => (0, 255, 255),
            "darkblue" => (0, 0, 139),
            "darkcyan" => (0, 139, 139),
            "darkgoldenrod" => (184, 134, 11),
            "darkgray" => (169, 169, 169),
            "darkgreen" => (0, 100, 0),
            "darkgrey" => (169, 169, 169),
            "darkkhaki" => (189, 183, 107),
            "darkmagenta" => (139, 0, 139),
            "darkolivegreen" => (85, 107, 47),
            "darkorange" => (255, 140, 0),
            "darkorchid" => (153, 50, 204),
            "darkred" => (139, 0, 0),
            "darksalmon" => (233, 150, 122),
            "darkseagreen" => (143, 188, 143),
            "darkslateblue" => (72, 61, 139),
            "darkslategray" => (47, 79, 79),
            "darkslategrey" => (47, 79, 79),
            "darkturquoise" => (0, 206, 209),
            "darkviolet" => (148, 0, 211),
            "deeppink" => (255, 20, 147),
            "deepskyblue" => (0, 191, 255),
            "dimgray" => (105, 105, 105),
            "dimgrey" => (105, 105, 105),
            "dodgerblue" => (30, 144, 255),
            "firebrick" => (178, 34, 34),
            "floralwhite" => (255, 250, 240),
            "forestgreen" => (34, 139, 34),
            "fuchsia" => (255, 0, 255),
            "gainsboro" => (220, 220, 220),
            "ghostwhite" => (248, 248, 255),
            "gold" => (255, 215, 0),
            "goldenrod" => (218, 165, 32),
            "gray" => (128, 128, 128),
            "green" => (0, 128, 0),
            "greenyellow" => (173, 255, 47),
            "grey" => (128, 128, 128),
            "honeydew" => (240, 255, 240),
            "hotpink" => (255, 105, 180),
            "indianred" => (205, 92, 92),
            "indigo" => (75, 0, 130),
            "ivory" => (255, 255, 240),
            "khaki" => (240, 230, 140),
            "lavender" => (230, 230, 250),
            "lavenderblush" => (255, 240, 245),
            "lawngreen" => (124, 252, 0),
            "lemonchiffon" => (255, 250, 205),
            "lightblue" => (173, 216, 230),
            "lightcoral" => (240, 128, 128),
            "lightcyan" => (224, 255, 255),
            "lightgoldenrodyellow" => (250, 250, 210),
            "lightgray" => (211, 211, 211),
            "lightgreen" => (144, 238, 144),
            "lightgrey" => (211, 211, 211),
            "lightpink" => (255, 182, 193),
            "lightsalmon" => (255, 160, 122),
            "lightseagreen" => (32, 178, 170),
            "lightskyblue" => (135, 206, 250),
            "lightslategray" => (119, 136, 153),
            "lightslategrey" => (119, 136, 153),
            "lightsteelblue" => (176, 196, 222),
            "lightyellow" => (255, 255, 224),
            "lime" => (0, 255, 0),
            "limegreen" => (50, 205, 50),
            "linen" => (250, 240, 230),
            "magenta" => (255, 0, 255),
            "maroon" => (128, 0, 0),
            "mediumaquamarine" => (102, 205, 170),
            "mediumblue" => (0, 0, 205),
            "mediumorchid" => (186, 85, 211),
            "mediumpurple" => (147, 112, 219),
            "mediumseagreen" => (60, 179, 113),
            "mediumslateblue" => (123, 104, 238),
            "mediumspringgreen" => (0, 250, 154),
            "mediumturquoise" => (72, 209, 204),
            "mediumvioletred" => (199, 21, 133),
            "midnightblue" => (25, 25, 112),
            "mintcream" => (245, 255, 250),
            "mistyrose" => (255, 228, 225),
            "moccasin" => (255, 228, 181),
            "navajowhite" => (255, 222, 173),
            "navy" => (0, 0, 128),
            "oldlace" => (253, 245, 230),
            "olive" => (128, 128, 0),
            "olivedrab" => (107, 142, 35),
            "orange" => (255, 165, 0),
            "orangered" => (255, 69, 0),
            "orchid" => (218, 112, 214),
            "palegoldenrod" => (238, 232, 170),
            "palegreen" => (152, 251, 152),
            "paleturquoise" => (175, 238, 238),
            "palevioletred" => (219, 112, 147),
            "papayawhip" => (255, 239, 213),
            "peachpuff" => (255, 218, 185),
            "peru" => (205, 133, 63),
            "pink" => (255, 192, 203),
            "plum" => (221, 160, 221),
            "powderblue" => (176, 224, 230),
            "purple" => (128, 0, 128),
            "rebeccapurple" => (102, 51, 153),
            "red" => (255, 0, 0),
            "rosybrown" => (188, 143, 143),
            "royalblue" => (65, 105, 225),
            "saddlebrown" => (139, 69, 19),
            "salmon" => (250, 128, 114),
            "sandybrown" => (244, 164, 96),
            "seagreen" => (46, 139, 87),
            "seashell" => (255, 245, 238),
            "sienna" => (160, 82, 45),
            "silver" => (192, 192, 192),
            "skyblue" => (135, 206, 235),
            "slateblue" => (106, 90, 205),
            "slategray" => (112, 128, 144),
            "slategrey" => (112, 128, 144),
            "snow" => (255, 250, 250),
            "springgreen" => (0, 255, 127),
            "steelblue" => (70, 130, 180),
            "tan" => (210, 180, 140),
            "teal" => (0, 128, 128),
            "thistle" => (216, 191, 216),
            "tomato" => (255, 99, 71),
            "turquoise" => (64, 224, 208),
            "violet" => (238, 130, 238),
            "wheat" => (245, 222, 179),
            "white" => (255, 255, 255),
            "whitesmoke" => (245, 245, 245),
            "yellow" => (255, 255, 0),
            "yellowgreen" => (154, 205, 50),
            "transparent" => (0, 0, 0),
            _ => return None,
        };
        let alpha = if name == "transparent" { 0.0 } else { 1.0 };
        Some(Color::rgba(r, g, b, alpha))
    }

    /// CSS 命名颜色反向查找——根据 RGB 值返回名称（如 (255,0,0) → "red"）。
    /// 用于序列化时优先输出颜色名称。
    pub(crate) fn reverse_lookup_named_color(c: &Color) -> Option<&'static str> {
        if c.a != 1.0 {
            return None;
        }
        Some(match (c.r, c.g, c.b) {
            (240, 248, 255) => "aliceblue",
            (250, 235, 215) => "antiquewhite",
            (0, 255, 255) => "aqua",
            (127, 255, 212) => "aquamarine",
            (240, 255, 255) => "azure",
            (245, 245, 220) => "beige",
            (255, 228, 196) => "bisque",
            (0, 0, 0) => "black",
            (255, 235, 205) => "blanchedalmond",
            (0, 0, 255) => "blue",
            (138, 43, 226) => "blueviolet",
            (165, 42, 42) => "brown",
            (222, 184, 135) => "burlywood",
            (95, 158, 160) => "cadetblue",
            (127, 255, 0) => "chartreuse",
            (210, 105, 30) => "chocolate",
            (255, 127, 80) => "coral",
            (100, 149, 237) => "cornflowerblue",
            (255, 248, 220) => "cornsilk",
            (220, 20, 60) => "crimson",
            (0, 139, 139) => "darkcyan",
            (184, 134, 11) => "darkgoldenrod",
            (169, 169, 169) => "darkgray",
            (0, 100, 0) => "darkgreen",
            (189, 183, 107) => "darkkhaki",
            (139, 0, 139) => "darkmagenta",
            (85, 107, 47) => "darkolivegreen",
            (255, 140, 0) => "darkorange",
            (153, 50, 204) => "darkorchid",
            (139, 0, 0) => "darkred",
            (233, 150, 122) => "darksalmon",
            (143, 188, 143) => "darkseagreen",
            (72, 61, 139) => "darkslateblue",
            (47, 79, 79) => "darkslategray",
            (0, 206, 209) => "darkturquoise",
            (148, 0, 211) => "darkviolet",
            (255, 20, 147) => "deeppink",
            (0, 191, 255) => "deepskyblue",
            (105, 105, 105) => "dimgray",
            (30, 144, 255) => "dodgerblue",
            (178, 34, 34) => "firebrick",
            (255, 250, 240) => "floralwhite",
            (34, 139, 34) => "forestgreen",
            (255, 0, 255) => "fuchsia",
            (220, 220, 220) => "gainsboro",
            (248, 248, 255) => "ghostwhite",
            (255, 215, 0) => "gold",
            (218, 165, 32) => "goldenrod",
            (128, 128, 128) => "gray",
            (0, 128, 0) => "green",
            (173, 255, 47) => "greenyellow",
            (240, 255, 240) => "honeydew",
            (255, 105, 180) => "hotpink",
            (205, 92, 92) => "indianred",
            (75, 0, 130) => "indigo",
            (255, 255, 240) => "ivory",
            (240, 230, 140) => "khaki",
            (230, 230, 250) => "lavender",
            (255, 240, 245) => "lavenderblush",
            (124, 252, 0) => "lawngreen",
            (255, 250, 205) => "lemonchiffon",
            (173, 216, 230) => "lightblue",
            (240, 128, 128) => "lightcoral",
            (224, 255, 255) => "lightcyan",
            (250, 250, 210) => "lightgoldenrodyellow",
            (211, 211, 211) => "lightgray",
            (144, 238, 144) => "lightgreen",
            (255, 182, 193) => "lightpink",
            (255, 160, 122) => "lightsalmon",
            (32, 178, 170) => "lightseagreen",
            (135, 206, 250) => "lightskyblue",
            (119, 136, 153) => "lightslategray",
            (176, 196, 222) => "lightsteelblue",
            (255, 255, 224) => "lightyellow",
            (0, 255, 0) => "lime",
            (50, 205, 50) => "limegreen",
            (250, 240, 230) => "linen",
            (128, 0, 0) => "maroon",
            (102, 205, 170) => "mediumaquamarine",
            (0, 0, 205) => "mediumblue",
            (186, 85, 211) => "mediumorchid",
            (147, 112, 219) => "mediumpurple",
            (60, 179, 113) => "mediumseagreen",
            (123, 104, 238) => "mediumslateblue",
            (0, 250, 154) => "mediumspringgreen",
            (72, 209, 204) => "mediumturquoise",
            (199, 21, 133) => "mediumvioletred",
            (25, 25, 112) => "midnightblue",
            (245, 255, 250) => "mintcream",
            (255, 228, 225) => "mistyrose",
            (255, 228, 181) => "moccasin",
            (255, 222, 173) => "navajowhite",
            (0, 0, 128) => "navy",
            (253, 245, 230) => "oldlace",
            (128, 128, 0) => "olive",
            (107, 142, 35) => "olivedrab",
            (255, 165, 0) => "orange",
            (255, 69, 0) => "orangered",
            (218, 112, 214) => "orchid",
            (238, 232, 170) => "palegoldenrod",
            (152, 251, 152) => "palegreen",
            (175, 238, 238) => "paleturquoise",
            (219, 112, 147) => "palevioletred",
            (255, 239, 213) => "papayawhip",
            (255, 218, 185) => "peachpuff",
            (205, 133, 63) => "peru",
            (255, 192, 203) => "pink",
            (221, 160, 221) => "plum",
            (176, 224, 230) => "powderblue",
            (128, 0, 128) => "purple",
            (102, 51, 153) => "rebeccapurple",
            (255, 0, 0) => "red",
            (188, 143, 143) => "rosybrown",
            (65, 105, 225) => "royalblue",
            (139, 69, 19) => "saddlebrown",
            (250, 128, 114) => "salmon",
            (244, 164, 96) => "sandybrown",
            (46, 139, 87) => "seagreen",
            (255, 245, 238) => "seashell",
            (160, 82, 45) => "sienna",
            (192, 192, 192) => "silver",
            (135, 206, 235) => "skyblue",
            (106, 90, 205) => "slateblue",
            (112, 128, 144) => "slategray",
            (255, 250, 250) => "snow",
            (0, 255, 127) => "springgreen",
            (70, 130, 180) => "steelblue",
            (210, 180, 140) => "tan",
            (0, 128, 128) => "teal",
            (216, 191, 216) => "thistle",
            (255, 99, 71) => "tomato",
            (64, 224, 208) => "turquoise",
            (238, 130, 238) => "violet",
            (245, 222, 179) => "wheat",
            (255, 255, 255) => "white",
            (245, 245, 245) => "whitesmoke",
            (255, 255, 0) => "yellow",
            (154, 205, 50) => "yellowgreen",
            _ => return None,
        })
    }

    pub(crate) fn hsl_to_rgb(h: f64, s: f64, l: f64) -> Color {
        let h = h.rem_euclid(360.0);
        crate::__tracing::trace!(
            target: "sasspile::color",
            fn = "hsl_to_rgb",
            h = h, s = s, l = l,
            "converting HSL to RGB"
        );
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;
        let (r1, g1, b1) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
        let result = Color::rgb(
            ((r1 + m) * 255.0).round() as u8,
            ((g1 + m) * 255.0).round() as u8,
            ((b1 + m) * 255.0).round() as u8,
        );
        crate::__tracing::trace!(
            target: "sasspile::color",
            fn = "hsl_to_rgb",
            r = result.r, g = result.g, b = result.b,
            "HSL to RGB result"
        );
        result
    }

    /// HWB → RGB 转换 (W3C CSS Color 4 算法)。
    pub(crate) fn hwb_to_rgb(h: f64, w: f64, b: f64, alpha: f64) -> Color {
        crate::__tracing::trace!(
            target: "sasspile::color",
            fn = "hwb_to_rgb",
            h = h, w = w, b = b, alpha = alpha,
            "converting HWB to RGB"
        );
        let h = (h % 360.0) / 360.0;
        let mut w = w;
        let mut b = b;
        let sum = w + b;
        if sum > 1.0 {
            w /= sum;
            b /= sum;
        }
        let factor = 1.0 - w - b;
        let hue_to_rgb = |m1: f64, m2: f64, mut hue: f64| -> f64 {
            if hue < 0.0 {
                hue += 1.0;
            }
            if hue > 1.0 {
                hue -= 1.0;
            }
            if hue < 1.0 / 6.0 {
                m1 + (m2 - m1) * hue * 6.0
            } else if hue < 0.5 {
                m2
            } else if hue < 2.0 / 3.0 {
                m1 + (m2 - m1) * (2.0 / 3.0 - hue) * 6.0
            } else {
                m1
            }
        };
        let to_rgb = |hue: f64| -> f64 { hue_to_rgb(0.0, 1.0, hue) * factor + w };
        let r = to_rgb(h + 1.0 / 3.0);
        let g = to_rgb(h);
        let bl = to_rgb(h - 1.0 / 3.0);
        Color::rgba(
            (r * 255.0).round() as u8,
            (g * 255.0).round() as u8,
            (bl * 255.0).round() as u8,
            alpha,
        )
    }

    /// RGB → HSL 转换。
    pub(crate) fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
        crate::__tracing::trace!(
            target: "sasspile::color",
            fn = "rgb_to_hsl",
            r = r, g = g, b = b,
            "converting RGB to HSL"
        );
        let r = r as f64 / 255.0;
        let g = g as f64 / 255.0;
        let b = b as f64 / 255.0;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;
        if (max - min).abs() < f64::EPSILON {
            return (0.0, 0.0, l);
        }
        let d = max - min;
        let s = if l > 0.5 {
            d / (2.0 - max - min)
        } else {
            d / (max + min)
        };
        let h = if max == r {
            ((g - b) / d + if g < b { 6.0 } else { 0.0 }) * 60.0
        } else if max == g {
            ((b - r) / d + 2.0) * 60.0
        } else {
            ((r - g) / d + 4.0) * 60.0
        };
        let result = (h, s, l);
        crate::__tracing::trace!(
            target: "sasspile::color",
            fn = "rgb_to_hsl",
            h = result.0, s = result.1, l = result.2,
            "RGB to HSL result"
        );
        result
    }

    /// 简单伪随机数——基于系统时间。
    pub(crate) fn simple_random() -> f64 {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let val = (nanos % 1_000_000) as f64;
        val / 1_000_000.0
    }

    pub(crate) fn builtin_rgba(fn_name: &str, args: &[Value]) -> Result<Value> {
        // 检测是否为空格分隔的 CSS Level 4 语法（rgb(R G B) 或 rgb(R G B / A)）
        let is_space_sep = matches!(args.first(), Some(Value::List(_, Separator::Space, false)));
        // 展开空格分隔的 List（CSS Level 4 语法：rgb(R G B / A)）
        let args: Vec<Value> = if is_space_sep {
            if let Value::List(items, Separator::Space, false) = &args[0] {
                let mut flat = items.clone();
                // alpha 参数追加到末尾
                if args.len() > 1 {
                    flat.extend(args[1..].iter().cloned());
                }
                flat
            } else {
                args.to_vec()
            }
        } else {
            args.to_vec()
        };
        // 检测是否有 none 参数——有则 CSS 原样透传
        let has_none = args.iter().any(|a| matches!(a, Value::String(s, false) if s == "none"));
        // 检测 alpha 参数是否存在（4 个参数时最后一个为 alpha）
        let has_alpha = args.len() == 4;
        match &args[..] {
            [
                Value::Number(r, ru),
                Value::Number(g, gu),
                Value::Number(b, bu),
            ] => {
                // 百分比参数转换为 0-255
                let r_val = if ru.as_deref() == Some("%") { (r * 255.0 / 100.0).round() as u8 } else { *r as u8 };
                let g_val = if gu.as_deref() == Some("%") { (g * 255.0 / 100.0).round() as u8 } else { *g as u8 };
                let b_val = if bu.as_deref() == Some("%") { (b * 255.0 / 100.0).round() as u8 } else { *b as u8 };
                crate::__tracing::debug!(
                    target: "sasspile::color",
                    fn = "rgba",
                    r = *r, g = *g, b = *b,
                    "rgba 3-arg input"
                );
                Ok(Value::Color(Color::rgba_fmt(r_val, g_val, b_val, 1.0, ColorFormat::Rgb)))
            }
            [
                Value::Number(r, ru),
                Value::Number(g, gu),
                Value::Number(b, bu),
                Value::Number(a, ua),
            ] => {
                let r_val = if ru.as_deref() == Some("%") { (r * 255.0 / 100.0).round() as u8 } else { *r as u8 };
                let g_val = if gu.as_deref() == Some("%") { (g * 255.0 / 100.0).round() as u8 } else { *g as u8 };
                let b_val = if bu.as_deref() == Some("%") { (b * 255.0 / 100.0).round() as u8 } else { *b as u8 };
                let alpha = if ua.as_deref() == Some("%") {
                    *a / 100.0
                } else {
                    *a
                };
                crate::__tracing::debug!(
                    target: "sasspile::color",
                    fn = "rgba",
                    r = *r, g = *g, b = *b, a = *a,
                    "rgba 4-arg input"
                );
                Ok(Value::Color(Color::rgba_fmt(
                    r_val, g_val, b_val, alpha, ColorFormat::Rgb,
                )))
            }
            // rgba($color, $alpha) — 修改颜色的 alpha 通道
            [Value::Color(c), Value::Number(a, _)] => {
                Ok(Value::Color(Color::rgba_fmt(c.r, c.g, c.b, *a, c.format.clone())))
            }
            // CSS 透传：参数包含 none/var()/calc() 等非数值时，原样输出
            _ if has_none || args.iter().any(|a| {
                matches!(a, Value::Calc(_) | Value::String(_, false))
                    && !matches!(a, Value::Color(_))
            }) => {
                let (rgb_args, alpha) = if has_alpha {
                    (&args[..3], Some(&args[3]))
                } else {
                    (&args[..], None)
                };
                let sep = if is_space_sep { " " } else { ", " };
                let rgb_str = rgb_args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(sep);
                let full_str = if let Some(a) = alpha {
                    if is_space_sep {
                        format!("{rgb_str} / {a}")
                    } else {
                        format!("{rgb_str}, {a}")
                    }
                } else {
                    rgb_str
                };
                Ok(Value::String(format!("{fn_name}({full_str})"), false))
            }
            _ => Err(SassError::Eval("rgba requires 3-4 number arguments".into())),
        }
    }

    pub(crate) fn builtin_darken(args: &[Value]) -> Result<Value> {
        match args {
            [Value::Color(c), Value::Number(amount, _)] => {
                crate::__tracing::debug!(
                    target: "sasspile::color",
                    fn = "darken",
                    input_r = c.r, input_g = c.g, input_b = c.b, input_a = c.a,
                    amount = *amount,
                    "darken input"
                );
                // Sass darken = HSL lightness 减少
                let (h, s, l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);
                let new_l = (l - *amount / 100.0).max(0.0);
                let new_c = Evaluator::hsl_to_rgb(h, s, new_l);
                let result = Value::Color(Color::rgba_fmt(new_c.r, new_c.g, new_c.b, c.a, ColorFormat::RgbPercent(h, s, new_l)));
                crate::__tracing::debug!(
                    target: "sasspile::color",
                    fn = "darken",
                    result = %result,
                    "darken result"
                );
                Ok(result)
            }
            _ => Err(SassError::Eval("darken requires (color, amount) arguments".into())),
        }
    }

    pub(crate) fn builtin_lighten(args: &[Value]) -> Result<Value> {
        match args {
            [Value::Color(c), Value::Number(amount, _)] => {
                crate::__tracing::debug!(
                    target: "sasspile::color",
                    fn = "lighten",
                    input_r = c.r, input_g = c.g, input_b = c.b, input_a = c.a,
                    amount = *amount,
                    "lighten input"
                );
                // Sass lighten = HSL lightness 增加
                let (h, s, l) = Evaluator::rgb_to_hsl(c.r, c.g, c.b);
                let new_l = (l + *amount / 100.0).min(1.0);
                let new_c = Evaluator::hsl_to_rgb(h, s, new_l);
                let result = Value::Color(Color::rgba_fmt(new_c.r, new_c.g, new_c.b, c.a, ColorFormat::RgbPercent(h, s, new_l)));
                crate::__tracing::debug!(
                    target: "sasspile::color",
                    fn = "lighten",
                    result = %result,
                    "lighten result"
                );
                Ok(result)
            }
            _ => Err(SassError::Eval("lighten requires (color, amount) arguments".into())),
        }
    }

    pub(crate) fn builtin_mix(args: &[Value]) -> Result<Value> {
        match args {
            [Value::Color(a), Value::Color(b)] => {
                crate::__tracing::debug!(
                    target: "sasspile::color",
                    fn = "mix",
                    color_a = ?a, color_b = ?b, weight = 50.0_f64,
                    "mix 2-arg input"
                );
                Ok(Value::Color(Color::rgba(
                    ((a.r as u16 + b.r as u16) / 2) as u8,
                    ((a.g as u16 + b.g as u16) / 2) as u8,
                    ((a.b as u16 + b.b as u16) / 2) as u8,
                    (a.a + b.a) / 2.0,
                )))
            }
            [Value::Color(a), Value::Color(b), Value::Number(w, _)] => {
                crate::__tracing::debug!(
                    target: "sasspile::color",
                    fn = "mix",
                    color_a = ?a, color_b = ?b, weight = *w,
                    "mix 3-arg input"
                );
                let weight = *w / 100.0;
                Ok(Value::Color(Color::rgba(
                    (a.r as f64 * (1.0 - weight) + b.r as f64 * weight) as u8,
                    (a.g as f64 * (1.0 - weight) + b.g as f64 * weight) as u8,
                    (a.b as f64 * (1.0 - weight) + b.b as f64 * weight) as u8,
                    a.a * (1.0 - weight) + b.a * weight,
                )))
            }
            _ => {
                crate::__tracing::debug!(
                    target: "sasspile::color",
                    fn = "mix",
                    n_args = args.len(),
                    arg_types = ?args.iter().map(std::mem::discriminant).collect::<Vec<_>>(),
                    args_debug = ?args.iter().map(|a| format!("{a}")).collect::<Vec<_>>(),
                    "mix argument mismatch"
                );
                Err(SassError::Eval("mix requires 2-3 arguments".into()))
            }
        }
    }
}
