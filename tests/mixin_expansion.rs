use sasspile_rx::compile;

#[tokio::test]
async fn test_mixin_default_param() {
    let input = "@mixin pad($x: 8px) { padding: $x; }\n@include pad();\n";
    let result = compile(input).await;
    assert!(result.contains("padding: 8px"), "got: {:?}", result);
}

#[tokio::test]
async fn test_mixin_override_default() {
    let input = "@mixin pad($x: 8px) { padding: $x; }\n@include pad(16px);\n";
    let result = compile(input).await;
    assert!(result.contains("padding: 16px"), "got: {:?}", result);
}

#[tokio::test]
async fn test_mixin_two_params() {
    let input = "@mixin btn($size, $color) { .btn { width: $size; color: $color; } }\n@include btn(16px, blue);\n";
    let result = compile(input).await;
    assert!(
        result.contains("width: 16px") && result.contains("color: blue"),
        "got: {:?}",
        result
    );
}
