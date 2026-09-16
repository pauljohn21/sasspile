//! Built-in implementations (Math / Type / Selector / Misc).

use crate::shared::context::CompilerContext;

// ═══════════════════════════════════════════════════════════════════════════════
// Math helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// 解析数值,一并返回剥离的尾部 unit (如 "px", "%", "em")
pub(crate) fn parse_number_with_unit(s: &str) -> (f64, String) {
    let s = s.trim();
    let unit_start = s
        .char_indices()
        .find(|(_, c)| !c.is_ascii_digit() && *c != '.' && *c != '-' && *c != '+')
        .map(|(i, _)| i)
        .unwrap_or(s.len());
    let num_part = &s[..unit_start];
    let unit = s[unit_start..].trim().to_string();
    // 剥离 unit 末尾非字母部分 (如 "10px," → "10px")
    let unit = unit.trim_end_matches(|c: char| !c.is_alphabetic()).to_string();
    let num = num_part.parse::<f64>().unwrap_or(0.0);
    (num, unit)
}

pub(crate) fn parse_number(s: &str) -> f64 {
    parse_number_with_unit(s).0
}

pub(crate) fn builtin_percentage(args: &[String], _ctx: &CompilerContext) -> String {
    if args.is_empty() {
        return String::new();
    }
    let v = parse_number(&args[0]);
    format!("{}%", (v * 100.0).round() as i64)
}

pub(crate) fn builtin_math(name: &str, args: &[String], _ctx: &CompilerContext) -> String {
    if args.is_empty() {
        return "0".to_string();
    }
    let (v, unit) = parse_number_with_unit(&args[0]);
    let result = match name {
        "abs" => v.abs(),
        "ceil" => v.ceil(),
        "floor" => v.floor(),
        "round" => v.round(),
        _ => v,
    };
    if result.fract() == 0.0 {
        format!("{result:.0}{unit}")
    } else {
        format!("{result}{unit}")
    }
}

pub(crate) fn builtin_min_max(name: &str, args: &[String], ctx: &CompilerContext) -> String {
    if args.is_empty() {
        return "0".to_string();
    }
    let values: Vec<f64> = args
        .iter()
        .map(|a| parse_number(&super::substitute_vars(ctx, a)))
        .collect();
    let result = match name {
        "min" => values.iter().cloned().fold(f64::INFINITY, f64::min),
        "max" => values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        _ => 0.0,
    };
    if result.fract() == 0.0 {
        format!("{result:.0}")
    } else {
        result.to_string()
    }
}

pub(crate) fn builtin_unit(args: &[String], _ctx: &CompilerContext) -> String {
    if args.is_empty() {
        return String::new();
    }
    let s = args[0].trim();
    let num_end = s
        .char_indices()
        .find(|(_, c)| !c.is_ascii_digit() && *c != '.' && *c != '-')
        .map(|(i, _)| i)
        .unwrap_or(s.len());
    s[num_end..].to_string()
}

pub(crate) fn builtin_unitless(args: &[String], _ctx: &CompilerContext) -> String {
    if args.is_empty() {
        return "true".to_string();
    }
    let unit = builtin_unit(args, _ctx);
    if unit.is_empty() {
        "true".to_string()
    } else {
        "false".to_string()
    }
}

fn comparable_unit(a: &str, b: &str) -> bool {
    let ua = builtin_unit(&[a.to_string()], &crate::shared::context::CompilerContext::new());
    let ub = builtin_unit(&[b.to_string()], &crate::shared::context::CompilerContext::new());
    ua.is_empty() && ub.is_empty() || ua == ub
}

pub(crate) fn builtin_comparable(args: &[String], _ctx: &CompilerContext) -> String {
    if args.len() < 2 {
        return "true".to_string();
    }
    if comparable_unit(&args[0], &args[1]) {
        "true".to_string()
    } else {
        "false".to_string()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Type / Meta builtins
// ═══════════════════════════════════════════════════════════════════════════════

pub(crate) fn builtin_type_of(args: &[String], ctx: &CompilerContext) -> String {
    if args.is_empty() {
        return "null".to_string();
    }
    let vars = ctx.global_variables.borrow();
    let s = super::resolve_vars_only(&vars, &args[0]);
    drop(vars);
    let s = s.trim();
    if s.starts_with('(') && s.ends_with(')') && s.contains(':') {
        "map".to_string()
    } else if s.starts_with('[') || (s.starts_with('(') && s.contains(',')) {
        "list".to_string()
    } else if s.starts_with('"') || s.starts_with('\'') {
        "string".to_string()
    } else if s.starts_with('#') || crate::evaluate_dst::builtins::color_name_to_rgb(s).is_some() {
        "color".to_string()
    } else if s.parse::<f64>().is_ok() {
        "number".to_string()
    } else if s == "true" || s == "false" {
        "bool".to_string()
    } else {
        "string".to_string()
    }
}

pub(crate) fn builtin_variable_exists(args: &[String], ctx: &CompilerContext) -> String {
    if args.is_empty() {
        return "false".to_string();
    }
    let name = args[0].trim().trim_start_matches('$').to_string();
    if ctx.global_variables.borrow().contains_key(&name) {
        "true".to_string()
    } else {
        "false".to_string()
    }
}

pub(crate) fn builtin_global_variable_exists(args: &[String], ctx: &CompilerContext) -> String {
    builtin_variable_exists(args, ctx)
}

pub(crate) fn builtin_inspect(args: &[String], _ctx: &CompilerContext) -> String {
    if args.is_empty() {
        return "null".to_string();
    }
    args[0].trim().to_string()
}

pub(crate) fn builtin_not(args: &[String], _ctx: &CompilerContext) -> String {
    if args.is_empty() {
        return "true".to_string();
    }
    match args[0].trim() {
        "true" => "false".to_string(),
        "false" => "true".to_string(),
        "()" | "null" | "0" => "true".to_string(),
        _ => "false".to_string(),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Selector builtins
// ═══════════════════════════════════════════════════════════════════════════════

fn split_selector_list(s: &str) -> Vec<String> {
    super::split_args(s)
}

pub(crate) fn builtin_selector_nest(args: &[String], _ctx: &CompilerContext) -> String {
    if args.is_empty() {
        return String::new();
    }
    let selectors = split_selector_list(&args[0]);
    if args.len() == 1 {
        return selectors.join(", ");
    }
    let parent = &args[1];
    let parent_parts: Vec<&str> = parent.split(',').map(|s| s.trim()).collect();
    let mut result = Vec::new();
    for p in &parent_parts {
        for s in &selectors {
            result.push(format!("{p} {s}"));
        }
    }
    result.join(", ")
}

pub(crate) fn builtin_selector_append(args: &[String], _ctx: &CompilerContext) -> String {
    if args.is_empty() {
        return String::new();
    }
    let selectors = split_selector_list(&args[0]);
    if args.len() == 1 {
        return selectors.join(", ");
    }
    let suffix = &args[1];
    selectors
        .iter()
        .map(|s| format!("{s}{suffix}"))
        .collect::<Vec<_>>()
        .join(", ")
}

// ═══════════════════════════════════════════════════════════════════════════════
// Misc builtins
// ═══════════════════════════════════════════════════════════════════════════════

pub(crate) fn builtin_call(args: &[String], _ctx: &CompilerContext) -> String {
    let _ = args;
    tracing::warn!("call() not fully implemented");
    String::new()
}

pub(crate) fn builtin_get_function(args: &[String], _ctx: &CompilerContext) -> String {
    let _ = args;
    String::new()
}

pub(crate) fn builtin_if_function(args: &[String], _ctx: &CompilerContext) -> String {
    if args.len() < 3 {
        return String::new();
    }
    let cond = args[0].trim();
    match cond {
        "true" | "1" | "non-null-non-falsy" => args[1].clone(),
        _ => args[2].clone(),
    }
}
