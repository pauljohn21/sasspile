//! @include 多行 + using 子句 + content 块测试

use sasspile::compile;

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_test_writer()
        .try_init();
}

#[test]
fn include_with_inline_content_block() {
    init_tracing();
    // @include 同行有 { 但无 } → Multi 模式
    let input = "@mixin box {\n  @content;\n}\n@include box() {\n  color: red;\n}";
    let output = compile(input);
    assert!(output.contains("color: red"), "content block should replace @content, got: {output}");
}

#[test]
fn include_single_line_with_semi() {
    let input = "@mixin pad($x) { padding: $x; }\n@include pad(8px);";
    let output = compile(input);
    assert!(output.contains("padding: 8px"), "normal single-line @include still works, got: {output}");
}

#[test]
fn include_single_line_no_semi() {
    let input = "@mixin pad($x) { padding: $x; }\n@include pad(8px)";
    let output = compile(input);
    assert!(output.contains("padding: 8px"), "no-semicolon @include still works, got: {output}");
}

#[test]
fn include_empty_content_block() {
    let input = "@mixin box {\n  @content;\n}\n@include box() {\n}";
    let output = compile(input);
    // empty content → @content replaced with nothing → no output
    assert!(!output.contains("@content"), "@content should be consumed, got: {output}");
}

#[test]
fn include_with_args_and_content() {
    let input = "@mixin styled($c) {\n  color: $c;\n  @content;\n}\n@include styled(blue) {\n  font-size: 14px;\n}";
    let output = compile(input);
    assert!(output.contains("color: blue"), "mixin arg substitution, got: {output}");
    assert!(output.contains("font-size: 14px"), "content block appended, got: {output}");
}
