//! Smoke test for Phase 1: interpolation `#{}` + variable bootstrap pattern.
#[test]
fn interpolation_basic_in_property() {
    let scss = r#"$theme: "dark";
body {
  data-theme: $theme;
}"#;
    let css = sasspile_rx::compile(scss).expect("should compile");
    assert!(
        css.contains("data-theme: dark"),
        "expected interpolation output, got:\n---\n{css}"
    );
}

#[test]
fn interpolation_in_selector() {
    let scss = r#"$prefix: "xx";
.#{$prefix}-box { color: red; }
"#;
    let css = sasspile_rx::compile(scss).expect("should compile");
    assert!(
        css.contains(".xx-box"),
        "expected .xx-box in output, got:\n---\n{css}"
    );
}

#[test]
fn bootstrap_size_pattern() {
    // Bootstrap style: $spacer * 0.25 -> uses variable interpolation
    let scss = r#"$spacer: 1rem; $prefix: bs;
.#{$prefix}-gutter-x { --#{$prefix}-gutter-x: #{$spacer}; }
"#;
    let css = sasspile_rx::compile(scss).expect("should compile");
    assert!(
        css.contains(".bs-gutter-x"),
        "expected .bs-gutter-x, got:\n---\n{css}"
    );
    assert!(
        css.contains("--bs-gutter-x"),
        "expected --bs-gutter-x, got:\n---\n{css}"
    );
}

#[test]
fn interpolation_only_defined_vars_replace() {
    // 测试未定义变量保留字面 — 用单引号字符串避开 r# 界限
    let scss = ".thing { content: '#{$undefined}-suffix'; }";
    let css = sasspile_rx::compile(scss).expect("should compile");
    // If $undefined is not in context, the token stays literal
    assert!(
        css.contains("#{$undefined}-suffix") || css.contains("__undefined__-suffix"),
        "undefined should remain literal or substitute, got:\n---\n{css}"
    );
}
