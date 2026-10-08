use rx_scss::telemetry::init_test_tracing;

#[test]
fn test_tracing_initialization() {
    init_test_tracing();

    // Verify tracing is working
    tracing::info!("Telemetry initialized successfully");

    let input = r#"
        $type: "primary";
        @if $type == "primary" {
            .first { color: red; }
        }
        @else if $type == "secondary" {
            .second { color: blue; }
        }
    "#;

    let result = rx_scss::from_string(input, &rx_scss::Options::default());
    tracing::info!(output = ?result, "Compilation completed");
    assert!(result.is_ok());

    let css = result.unwrap();
    assert!(css.contains("color: red"), "Expected 'color: red' in output:\n{}", css);
}
