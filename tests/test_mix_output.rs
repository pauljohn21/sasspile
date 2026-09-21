//! Test color.mix output format

#[test]
fn test_mix_output_format() {
    sasspile::init_tracing();
    // Test color.mix(#409eff, #ffffff, 30%) — should produce rgb(%) format
    let input = "
$x: color.mix(#409eff, #ffffff, 30%);
.test { color: $x; }
";
    let css = sasspile::compile_expanded(input).unwrap();
    tracing::warn!(css = %css, "MIX OUTPUT");
}
