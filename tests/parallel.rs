use sasspile_rx::compile_parallel;

#[tokio::test]
async fn test_compile_parallel_empty() {
    assert_eq!(compile_parallel("").await, "");
}

#[tokio::test]
async fn test_compile_parallel_with_delimiter() {
    assert_eq!(compile_parallel("a;b").await, "a\n;\nb\n");
}
