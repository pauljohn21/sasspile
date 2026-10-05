//! 诊断 pagination mixin &:focus 展开问题
#![allow(clippy::unwrap_used)]

#[test]
fn test_has_descendant_prefix_multi() {
    // Test the function directly
    use sasspile::compile_expanded;
    let scss = r#"
        @mixin btn {
            &:focus { outline: none; }
        }
        .parent {
            .c1,
            .c2 {
                @include btn;
            }
        }
    "#;
    let css = compile_expanded(scss).unwrap();
    tracing::info!("OUTPUT:\n{css}");

    // Print the lines that contain :focus
    for line in css.lines() {
        if line.contains(":focus") && line.contains("{") {
            tracing::info!("LINE: {line}");
        }
    }

    // The EXPECTED output for the &:focus rule should be:
    // .parent .c1:focus, .parent .c2:focus { outline: none; }
    // NOT:
    // .parent .c1:focus, .c2:focus { outline: none; }

    let is_correct = css.contains(".parent .c1:focus, .parent .c2:focus");
    assert!(is_correct, "Should have both prefixed. Got: {css}");
}
