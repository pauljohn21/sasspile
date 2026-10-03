//! Test color.mix output format
#![allow(clippy::unwrap_used)]

#[test]
fn test_mix_output_format() {
    sasspile::init_tracing();
    // Test color.mix(#409eff, #ffffff, 30%) — should produce rgb(%) format
    let input = "
$x: color.mix(#409eff, #ffffff, 30%);
.test { color: $x; }
";
    let css = sasspile::compile_expanded(input).expect("compile color.mix");
    tracing::warn!(css = %css, "MIX OUTPUT");
}
