//! 调试 placeholder extend 行为
use sasspile::OutputStyle;

#[test]
fn debug_nested_single_extend() {
    // 模拟 image.scss 结构：%size 在顶层，extender 在嵌套规则内
    let css = sasspile::compile(
        "%size { width: 100%; height: 100%; }\n.parent { .child { @extend %size; color: red; } }",
        OutputStyle::Expanded,
    )
    .expect("compile failed");
    tracing::info!("=== nested_single OUTPUT ===\n{css}");
    assert!(!css.contains("%size"), "占位符不应出现: {css}");
    assert!(css.contains(".parent .child"), "应该包含嵌套选择器: {css}");
    assert!(css.contains("width: 100%"), "应该包含 width: 100%: {css}");
}

#[test]
fn debug_nested_multi_extend() {
    // 多个嵌套 extender 扩展同一个 placeholder
    let css = sasspile::compile(
        "%size { width: 100%; height: 100%; }\n.model { .a { @extend %size; } .b { @extend %size; } }",
        OutputStyle::Expanded,
    )
    .expect("compile failed");
    tracing::info!("=== nested_multi OUTPUT ===\n{css}");
    assert!(!css.contains("%size"), "占位符不应出现: {css}");
    assert!(css.contains("width: 100%"), "应该包含 width: 100%: {css}");
}

#[test]
fn debug_mixin_extend() {
    // 模拟 @extend 在 @include 内部的情况
    let css = sasspile::compile(
        "%base { color: red; }\n@mixin my-mixin { @extend %base; font-size: 14px; }\n.foo { @include my-mixin; }",
        OutputStyle::Expanded,
    )
    .expect("compile failed");
    tracing::info!("=== mixin_extend OUTPUT ===\n{css}");
    assert!(!css.contains("%base"), "占位符不应出现: {css}");
    assert!(css.contains(".foo"), "应该包含 .foo: {css}");
    assert!(css.contains("color: red"), "应该包含 color: red: {css}");
}

#[test]
fn debug_image_placeholder() {
    sasspile::init_tracing();
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("element-plus/packages/theme-chalk/src/image.scss");
    let css =
        sasspile::compile_file(&path, OutputStyle::Expanded).expect("compile failed");
    let preview: String = css.chars().take(1500).collect();
    tracing::info!("=== image.scss OUTPUT (first 1500 chars) ===\n{preview}");
    assert!(
        css.contains("width: 100%"),
        "应该包含 width: 100%: {css}"
    );
}

#[test]
fn debug_simple_placeholder() {
    let css = sasspile::compile(
        "%base { color: red; }\n.child { @extend %base; }",
        OutputStyle::Expanded,
    )
    .expect("compile failed");
    tracing::info!("=== simple OUTPUT ===\n{css}");
    assert!(!css.contains("%base"), "占位符不应出现: {css}");
    assert!(css.contains(".child"), "应该包含 .child: {css}");
    assert!(css.contains("color: red"), "应该包含 color: red: {css}");
}

#[test]
fn debug_image_minimal() {
    let css = sasspile::compile(
        "%size { width: 100%; height: 100%; }\n.a { @extend %size; color: red; }",
        OutputStyle::Expanded,
    )
    .expect("compile failed");
    tracing::info!("=== minimal OUTPUT ===\n{css}");
    let a_rule_count = css.matches(".a {").count();
    assert_eq!(a_rule_count, 1, "应该只有一个 .a 规则: {css}");
    assert!(css.contains("width: 100%"), "应该包含 width: 100%");
    assert!(css.contains("color: red"), "应该包含 color: red");
}

#[test]
fn debug_multi_extend() {
    let css = sasspile::compile(
        "%box { display: block; }\n.a { @extend %box; color: red; }\n.b { @extend %box; color: blue; }",
        OutputStyle::Expanded,
    )
    .expect("compile failed");
    tracing::info!("=== multi OUTPUT ===\n{css}");
    assert!(!css.contains("%box"), "占位符不应出现: {css}");
    assert!(css.contains("display: block"), "应该包含 display: block");
    assert!(
        css.contains(".a, .b") || css.contains(".b, .a"),
        "应该包含组合选择器: {css}"
    );
}
