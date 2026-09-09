//! HSL/HWB 手动 spec 验证
use sasspile::compile_expanded;

#[test]
fn spec_hsl_units() {
    let pass = vec![
        ("a { c: hsl(0deg, 100%, 50%) }", "hsl(0, 100%, 50%)"),
        ("a { c: hsl(60, 100%, 50%) }", "hsl(60, 100%, 50%)"),
        ("a { c: hsl(60in, 100%, 50%) }", "hsl(60, 100%, 50%)"),
        ("a { c: hsl(60rad, 100%, 50%) }", "hsl(197.7467707849, 100%, 50%)"),
    ];
    for (input, expected) in pass {
        let r = compile_expanded(input).expect("unexpected failure in test");
        assert!(r.contains(expected), "FAIL: input={input}\n  expected={expected}\n  got={r}");
    }
}

#[test]
fn spec_hsl_clamped() {
    let pass = vec![
        ("a { c: hsl(0, -100%, 50%) }", "hsl(0, 0%, 50%)"),
        ("a { c: hsl(0, 500%, 50%) }", "hsl(0, 500%, 50%)"),
        ("a { c: hsl(0, 100%, -100%) }", "hsl(0, 100%, -100%)"),
        ("a { c: hsl(0, 100%, 500%) }", "hsl(0, 100%, 500%)"),
    ];
    for (input, expected) in pass {
        let r = compile_expanded(input).expect("unexpected failure in test");
        assert!(r.contains(expected), "FAIL: input={input}\n  expected={expected}\n  got={r}");
    }
}

#[test]
fn spec_hsl_hue_wrap() {
    let pass = vec![
        ("a { c: hsl(390, 100%, 50%) }", "hsl(30, 100%, 50%)"),
        ("a { c: hsl(-30, 100%, 50%) }", "hsl(330, 100%, 50%)"),
        ("a { c: hsl(360, 100%, 50%) }", "hsl(0, 100%, 50%)"),
        ("a { c: hsl(-360, 100%, 50%) }", "hsl(0, 100%, 50%)"),
    ];
    for (input, expected) in pass {
        let r = compile_expanded(input).expect("unexpected failure in test");
        assert!(r.contains(expected), "FAIL: input={input}\n  expected={expected}\n  got={r}");
    }
}

#[test]
fn spec_hsl_missing_channels() {
    let pass = vec![
        ("a { c: hsl(none, 100%, 50%) }", "hsl(none 100% 50%)"),
        ("a { c: hsl(0, none, 50%) }", "hsl(0deg none 50%)"),
        ("a { c: hsl(0, 100%, none) }", "hsl(0deg 100% none)"),
    ];
    for (input, expected) in pass {
        let r = compile_expanded(input).expect("unexpected failure in test");
        assert!(r.contains(expected), "FAIL: input={input}\n  expected={expected}\n  got={r}");
    }
}

#[test]
fn spec_hwb_basic() {
    let cases = vec![
        ("a { c: hwb(0deg 30% 40%) }", "hsl(0, 33.3333333333%, 45%)"),
        ("a { c: hwb(1rad 30% 40%) }", "hsl(57.2957795131, 33.3333333333%, 45%)"),
        ("a { c: hwb(180 30% 40% / 0.5) }", "hsla(180, 33.3333333333%, 45%, 0.5)"),
        ("a { c: hwb(180 30% 40% / 0) }", "hsla(180, 33.3333333333%, 45%, 0)"),
        ("a { c: hwb(0 30% 40% / 1.1) }", "hsl(0, 33.3333333333%, 45%)"), // alpha clamp to 1
        ("a { c: hwb(0 30% 40% / -0.1) }", "hsla(0, 33.3333333333%, 45%, 0)"), // alpha clamp to 0
    ];
    for (input, expected) in cases {
        let r = compile_expanded(input).unwrap_or_else(|e| format!("ERR: {e}"));
        assert!(r.contains(expected), "FAIL: input={input}\n  expected={expected}\n  got={r}");
    }
}

#[test]
fn spec_hwb_missing() {
    let cases = vec![
        ("a { c: hwb(none 30% 40%) }", "hwb(none 30% 40%)"),
        ("a { c: hwb(0 none 40%) }", "hwb(0deg none 40%)"),
        ("a { c: hwb(0 30% none) }", "hwb(0deg 30% none)"),
        ("a { c: hwb(0 30% 40% / none) }", "hwb(0deg 30% 40% / none)"),
    ];
    for (input, expected) in cases {
        let r = compile_expanded(input).unwrap_or_else(|e| format!("ERR: {e}"));
        assert!(r.contains(expected), "FAIL: input={input}\n  expected={expected}\n  got={r}");
    }
}

#[test]
fn spec_is_missing_channel() {
    let pass = vec![
        ("a { c: color.is-missing(hsl(none, 100%, 50%), hue) }", "true"),
        ("a { c: color.is-missing(hsl(none, 100%, 50%), saturation) }", "false"),
        ("a { c: color.is-missing(hsl(120, 50%, 50%), hue) }", "false"),
    ];
    for (input, expected) in pass {
        let r = compile_expanded(input).expect("unexpected failure in test");
        assert!(r.contains(expected), "FAIL: input={input}\n  expected={expected}\n  got={r}");
    }
}

#[test]
fn spec_hwb_global() {
    // hwb 不带 @use 的调用
    let cases = vec![
        ("a { c: hwb(180 30% 40%) }", "hsl(180, 33.3333333333%, 45%)"),
    ];
    for (input, expected) in cases {
        let r = compile_expanded(input).unwrap_or_else(|e| format!("ERR: {e}"));
        assert!(r.contains(expected), "FAIL: input={input}\n  expected={expected}\n  got={r}");
    }
}
