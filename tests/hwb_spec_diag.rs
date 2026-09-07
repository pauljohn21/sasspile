//! HWB spec 诊断测试——验证 sass-spec HWB 目录关键案例。

use sasspile::compile_expanded;

#[test]
fn hwb_degenerate_hue() {
    let cases = vec![
        ("a {b: color.hwb(calc(infinity), 30%, 40%, 0.5)}", "hsla(0, 33.3333333333%, 45%, 0.5)"),
        ("a {b: color.hwb(calc(-infinity), 30%, 40%, 0.5)}", "hsla(0, 33.3333333333%, 45%, 0.5)"),
        ("a {b: color.hwb(calc(NaN), 30%, 40%, 0.5)}", "hsla(0, 33.3333333333%, 45%, 0.5)"),
        ("a {b: color.hwb(-0, 30%, 40%, 0.5)}", "hsla(0, 33.3333333333%, 45%, 0.5)"),
    ];
    for (input, expected) in cases {
        let r = compile_expanded(input).unwrap_or_else(|e| format!("ERR: {e}"));
        assert!(r.contains(expected), "FAIL: input={input}\n  expected={expected}\n  got={r}");
    }
}

#[test]
fn hwb_alpha_percent() {
    let cases = vec![
        ("a {b: color.hwb(0, 0%, 0%, 100%)}", "red"),
        ("a {b: color.hwb(0, 0%, 0%, 250%)}", "red"),
        ("a {b: color.hwb(0, 0%, 0%, 250)}", "red"),
    ];
    for (input, expected) in cases {
        let r = compile_expanded(input).unwrap_or_else(|e| format!("ERR: {e}"));
        assert!(r.contains(expected), "FAIL: input={input}\n  expected={expected}\n  got={r}");
    }
}

#[test]
fn hwb_whiteness_nan_infinity() {
    let cases = vec![
        ("a {b: color.hwb(0, calc(infinity * 1%), 40%, 0.5)}", "hsla(0, 100%, 50%, 0.5)"),
        ("a {b: color.hwb(0, calc(-infinity * 1%), 40%, 0.5)}", "hsla(0, 0%, 0%, 0.5)"),
        ("a {b: color.hwb(0, calc(NaN * 1%), 40%, 0.5)}", "hsla(0, 100%, 30%, 0.5)"),
    ];
    for (input, expected) in cases {
        let r = compile_expanded(input).unwrap_or_else(|e| format!("ERR: {e}"));
        assert!(r.contains(expected), "FAIL: input={input}\n  expected={expected}\n  got={r}");
    }
}
