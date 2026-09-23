//! @use/@forward 解析指令消费测试

use sasspile::compile;

#[test]
fn use_does_not_panic() {
    let input = "@use './base.scss' as *;";
    let _result = compile(input);
}

#[test]
fn use_with_downstream_tokens() {
    let input = r#"
@use './base.scss' as *;
.button { color: red; }
"#;
    let _result = compile(input);
}

#[test]
fn forward_does_not_panic() {
    let input = r#"
@forward './colors.scss';
.button { color: red; }
"#;
    let _result = compile(input);
}

#[test]
fn use_and_forward_together() {
    let input = r#"
@use './base.scss' as *;
@forward './colors.scss';
.button { color: red; }
"#;
    let _result = compile(input);
}

#[test]
fn use_with_clause() {
    let input = r#"
@use './a.scss' with ($x: 2);
.button { width: $x; }
"#;
    let _result = compile(input);
}
