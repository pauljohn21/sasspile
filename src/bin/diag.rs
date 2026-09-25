use sasspile::compile;

#[tokio::main]
async fn main() {
    let _ = tracing_subscriber::fmt::try_init();

    let cases: Vec<(&str, &str)> = vec![
        // 多行 placeholder extend
        (
            "%spacer {\n  margin: 0;\n  padding: 0;\n}\n.box { @extend %spacer; }",
            "extend_placeholder_multi_line",
        ),
        // !optional + undefined placeholder
        (
            ".bar { @extend %undefined !optional; }",
            "extend_optional_safe",
        ),
        // 选择器级 extend 跨行
        (
            "a {b: c}\nd {\n  @extend\n  a\n}",
            "extend_multiline_selector",
        ),
        // 选择器级 extend + !optional 跨行
        (
            "a {@extend b\n  !optional}",
            "extend_multiline_optional",
        ),
        // 选择器级 extend 目标在下一行
        (
            "a {b: c}\nd {@extend\n  a}",
            "extend_before_arg_scss",
        ),
    ];

    for (input, desc) in cases {
        let output = compile(input);
        eprintln!("=== {desc} ===");
        eprintln!("INPUT:\n{input}");
        eprintln!("OUTPUT:\n<{output}>");
        eprintln!("---");
    }
}
