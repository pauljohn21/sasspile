use sasspile_rx::compile;

#[tokio::test]
async fn test_compile_empty() {
    assert_eq!(compile("").await, "");
}

#[tokio::test]
async fn test_compile_with_delimiter() {
    assert_eq!(compile("a;b").await, "a\n;\nb\n");
}
