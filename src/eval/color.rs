//! 颜色函数实现 (响应式风格移植自 main color.rs)

use super::{parse_color, rgb_to_hsl, hsl_to_rgb, strip_pct};

pub fn eval_rgb_call(args: &[String]) -> Option<String> {
    let r = args.first()?.trim().parse::<u64>().ok()?.min(255) as u8;
    let g = args.get(1)?.trim().parse::<u64>().ok()?.min(255) as u8;
    let b = args.get(2)?.trim().parse::<u64>().ok()?.min(255) as u8;
    Some(format!("#{r:02x}{g:02x}{b:02x}"))
}

pub fn eval_rgba(args: &[String]) -> Option<String> {
    if args.len() == 4 {
        let r = args.first()?.trim().parse::<u64>().ok()?.min(255) as u8;
        let g = args.get(1)?.trim().parse::<u64>().ok()?.min(255) as u8;
        let b = args.get(2)?.trim().parse::<u64>().ok()?.min(255) as u8;
        let a = args.get(3)?.trim().parse::<f64>().ok()?;
        if a >= 1.0 {
            Some(format!("#{r:02x}{g:02x}{b:02x}"))
        } else {
            Some(format!("#{r:02x}{g:02x}{b:02x}{:02x}", (a * 255.0).round() as u8))
        }
    } else if args.len() == 2 {
        let color = args.first()?.trim();
        let base = color.trim_start_matches('#');
        let a = args.get(1)?.trim().parse::<f64>().ok()?;
        if base.len() == 6 {
            let r = u8::from_str_radix(&base[0..2], 16).ok()?;
            let g = u8::from_str_radix(&base[2..4], 16).ok()?;
            let b = u8::from_str_radix(&base[4..6], 16).ok()?;
            if a >= 1.0 {
                Some(format!("#{r:02x}{g:02x}{b:02x}"))
            } else {
                Some(format!("#{r:02x}{g:02x}{b:02x}{:02x}", (a * 255.0).round() as u8))
            }
        } else {
            None
        }
    } else {
        None
    }
}

pub fn eval_hsl_call(args: &[String]) -> Option<String> {
    let h = args.first()?.trim().parse::<f64>().ok()?;
    let s_pct = strip_pct(args.get(1)?)?;
    let l_pct = strip_pct(args.get(2)?)?;
    let (r, g, b) = hsl_to_rgb(h, s_pct / 100.0, l_pct / 100.0);
    Some(format!("#{r:02x}{g:02x}{b:02x}"))
}

pub fn eval_lighten(args: &[String]) -> Option<String> {
    let color = parse_color(args.first()?)?;
    let amount = strip_pct(args.get(1)?)? / 100.0;
    let (h, s, l) = rgb_to_hsl(color.0, color.1, color.2);
    let new_l = (l + amount).clamp(0.0, 1.0);
    let (r, g, b) = hsl_to_rgb(h, s, new_l);
    Some(format!("#{r:02x}{g:02x}{b:02x}"))
}

pub fn eval_darken(args: &[String]) -> Option<String> {
    let color = parse_color(args.first()?)?;
    let amount = strip_pct(args.get(1)?)? / 100.0;
    let (h, s, l) = rgb_to_hsl(color.0, color.1, color.2);
    let new_l = (l - amount).clamp(0.0, 1.0);
    let (r, g, b) = hsl_to_rgb(h, s, new_l);
    Some(format!("#{r:02x}{g:02x}{b:02x}"))
}

pub fn eval_mix(args: &[String]) -> Option<String> {
    let c1 = parse_color(args.first()?)?;
    let c2 = parse_color(args.get(1)?)?;
    let weight = args.get(2).and_then(|s| s.trim().parse::<f64>().ok()).unwrap_or(50.0) / 100.0;
    let r = ((c1.0 as f64 * (1.0 - weight) + c2.0 as f64 * weight).round() as u8).min(255);
    let g = ((c1.1 as f64 * (1.0 - weight) + c2.1 as f64 * weight).round() as u8).min(255);
    let b = ((c1.2 as f64 * (1.0 - weight) + c2.2 as f64 * weight).round() as u8).min(255);
    Some(format!("#{r:02x}{g:02x}{b:02x}"))
}

pub fn eval_adjust_hue(args: &[String]) -> Option<String> {
    let color = parse_color(args.first()?)?;
    let degrees = args.get(1)?.trim().parse::<f64>().ok()?;
    let (h, s, l) = rgb_to_hsl(color.0, color.1, color.2);
    let new_h = (h + degrees) % 360.0;
    let new_h = if new_h < 0.0 { new_h + 360.0 } else { new_h };
    let (r, g, b) = hsl_to_rgb(new_h, s, l);
    Some(format!("#{r:02x}{g:02x}{b:02x}"))
}
