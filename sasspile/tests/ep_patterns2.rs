//! EP 失败模式更精确诊断。

use sasspile::{tokenize, parse};

fn test_parse_detailed(name: &str, src: &str) {
    let (tokens, lex_diags) = tokenize(src);
    let lex_e = lex_diags.errors().len();
    let (_stylesheet, parse_diags) = parse(src);
    let parse_e = parse_diags.errors().len();
    if lex_e == 0 && parse_e == 0 {
        tracing::info!(pattern = %name, "OK");
    } else {
        let lex_msg: Vec<String> = lex_diags.errors().iter().map(|d| d.message.clone()).collect();
        let parse_msg: Vec<String> = parse_diags.errors().iter().map(|d| d.message.clone()).collect();
        // 找到失败的行
        let failing_lines = find_lines(src, &parse_msg);
        tracing::info!(pattern = %name, lex_e, parse_e, lex_msg = ?lex_msg, parse_msg = ?parse_msg, failing_lines = ?failing_lines, "FAIL");
    }
}

fn find_lines(src: &str, _errors: &[String]) -> Vec<usize> {
    // 简单返回可能导致问题的行（包含特定模式的行）
    src.lines()
        .enumerate()
        .filter(|(_, line)| {
            line.contains("box-shadow")
                || line.contains("@at-root")
                || line.contains("::before")
                || line.contains("::after")
                || line.contains("getProperty")
                || line.contains("indent:")
        })
        .map(|(i, _)| i + 1)
        .collect()
}

fn init() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .try_init();
}

#[test]
fn test_box_shadow_simple() {
    init();

    // 最简单的多行 box-shadow
    test_parse_detailed("box_shadow_1", ".foo {\n  box-shadow: 1px 2px red, 3px 4px blue;\n}");

    // 换行后逗号分隔
    test_parse_detailed("box_shadow_2", ".foo {\n  box-shadow:\n    1px 2px red,\n    3px 4px blue;\n}");

    // 更少的多行
    test_parse_detailed("box_shadow_3", ".foo {\n  box-shadow:\n    1px 2px red;\n}");

    // 单行 box-shadow with comma
    test_parse_detailed("box_shadow_single_line", ".foo { box-shadow: 1px 2px red, 3px 4px blue; }");

    // 逗号在行尾
    test_parse_detailed("comma_at_end", ".foo {\n  color: red,\n  background: blue;\n}");

    // 更复杂的逗号值
    test_parse_detailed("multi_value_comma", ".foo {\n  margin: 1px 2px, 3px 4px;\n}");
}
