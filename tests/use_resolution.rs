use sasspile_rx::compile;

#[tokio::test]
async fn test_use_does_not_panic() {
    let input = "@use './base.scss' as *;";
    let _result = compile(input).await;
}

#[tokio::test]
async fn test_use_with_downstream_tokens() {
    let input = r#"
@use './base.scss' as *;
.button { color: red; }
"#;
    let _result = compile(input).await;
}

#[tokio::test]
async fn test_forward_does_not_panic() {
    let input = r#"
@forward './colors.scss';
.button { color: red; }
"#;
    let _result = compile(input).await;
}

#[tokio::test]
async fn test_use_and_forward_together() {
    let input = r#"
@use './base.scss' as *;
@forward './colors.scss';
.button { color: red; }
"#;
    let _result = compile(input).await;
}

#[tokio::test]
async fn test_use_with_clause() {
    let input = r#"
@use './a.scss' with ($x: 2);
.button { width: $x; }
"#;
    let _result = compile(input).await;
}
