//! 逗号值失败的精确诊断。

use sasspile::{tokenize, parse};

fn test_parse_debug(name: &str, src: &str) {
    let (tokens, lex_diags) = tokenize(src);
    let lex_e = lex_diags.errors().len();
    let (_stylesheet, parse_diags) = parse(src);
    let parse_e = parse_diags.errors().len();
    if lex_e == 0 && parse_e == 0 {
        tracing::info!(pattern = %name, "OK");
    } else {
        let lex_msg: Vec<String> = lex_diags.errors().iter().map(|d| d.message.clone()).collect();
        let parse_msg: Vec<String> = parse_diags.errors().iter().map(|d| d.message.clone()).collect();
        tracing::info!(pattern = %name, lex_e, parse_e, lex_msg = ?lex_msg, parse_msg = ?parse_msg, "FAIL");
    }
}

fn init() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .try_init();
}

#[test]
fn test_comma_value_patterns() {
    init();

    // 测试 1: 最简单逗号值
    test_parse_debug("simple_comma", ".foo { color: red, blue; }");

    // 测试 2: 数值逗号
    test_parse_debug("number_comma", ".foo { margin: 1px, 2px; }");

    // 测试 3: 三值逗号
    test_parse_debug("three_comma", ".foo { margin: 1px 2px, 3px 4px; }");

    // 测试 4: 带单位的逗号
    test_parse_debug("unit_comma", ".foo { margin: 1px 2px, 3px 4px, 5px 6px; }");

    // 测试 5: box-shadow 单行逗号
    test_parse_debug("box_shadow_inline", ".foo { box-shadow: 1px 2px red, 3px 4px blue; }");

    // 测试 6: 减法表达式
    test_parse_debug("subtraction", ".foo { width: 5px - 2px; }");

    // 测试 7: 加法表达式
    test_parse_debug("addition", ".foo { width: 5px + 2px; }");

    // 测试 8: 无空格的逗号
    test_parse_debug("no_space_comma", ".foo { color: red,blue; }");

    // 测试 9: 末尾逗号（无效但测试）
    test_parse_debug("trailing_comma", ".foo { color: red,; }");

    // 测试 10: 函数调用的逗号
    test_parse_debug("fn_call_comma", ".foo { color: rgb(0,0,0), blue; }");

    // 测试 11: 多行 box-shadow（精确复制 input.scss）
    test_parse_debug("box_shadow_multiline", ".foo {\n  box-shadow:\n    1px 0 0 0 red inset,\n    0 1px 0 0 red inset;\n}");

    // 测试 12: 逗号后的声明 (font-family 模式)
    test_parse_debug("comma_then_decl", ".foo {\n  font-family: a, b;\n  color: red;\n}");
}
