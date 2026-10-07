//! 诊断自定义属性 --#{$prefix}name 的解析问题
//! 运行: `cargo test --test diag_custom_prop -- --nocapture`

use rx_scss::builder::CompileBuilder;
use rx_scss::pipeline::from_string;
use rx_scss::serialize::Options;

#[test]
fn custom_prop_simple_interpolation() {
    let input = r#"$prefix: bs-;
:root {
  --#{$prefix}body-font-family: sans-serif;
}
"#;
    let css = from_string(input, &Options::expanded()).expect("compile failed");
    tracing::info!(css = %css, output = "simple interpolation");
    assert!(css.contains("--bs-body-font-family: sans-serif"),
        "Expected --bs-body-font-family, got: {}", css);
}

#[test]
fn custom_prop_double_dash_literal() {
    let input = r#"$prefix: "bs-";
:root {
  --#{$prefix}body-bg: #fff;
}
"#;
    let css = from_string(input, &Options::expanded()).expect("compile failed");
    tracing::info!(css = %css, output = "string interp");
}

#[test]
fn custom_prop_no_interp_just_ident() {
    let input = r#":root {
  --my-property: 42;
}
"#;
    let css = from_string(input, &Options::expanded()).expect("compile failed");
    tracing::info!(css = %css, output = "plain custom prop");
    assert!(css.contains("--my-property: 42"), "Got: {}", css);
}

#[test]
fn custom_prop_static_bs() {
    let input = r#":root {
  --bs-primary: red;
}
"#;
    let css = from_string(input, &Options::expanded()).expect("compile failed");
    tracing::info!(css = %css, output = "static bs");
    assert!(css.contains("--bs-primary: red"), "Got: {}", css);
}
