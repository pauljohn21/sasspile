//! 最小复现 — 确认 &__xxx 的选择器解析
#[test]
fn test_compound_ampersand() {
    sasspile::init_tracing();
    let scss = r#"
        .parent {
            &__child:focus { outline: none; }
        }
    "#;
    let css = sasspile::compile_expanded(scss).unwrap();
    tracing::info!("OUTPUT:\n{css}");
    assert!(
        css.contains(".parent__child:focus"),
        "Expected .parent__child:focus (compound). Got: {css}"
    );
    assert!(
        !css.contains(".parent .parent__child:focus"),
        "Should NOT have descendant .parent .parent__child:focus. Got: {css}"
    );
}

#[test]
fn test_b_em_mixin_compound() {
    sasspile::init_tracing();
    let scss = r#"
        @mixin b($block) { .#{$block} { @content; } }
        @include b(drawer) {
            position: absolute;
            &__sr-focus:focus { outline: none !important; }
            &__header { color: red; }
        }
    "#;
    let css = sasspile::compile_expanded(scss).unwrap();
    tracing::info!("OUTPUT:\n{css}");
    assert!(
        css.contains(".drawer__sr-focus:focus"),
        "Expected .drawer__sr-focus:focus (compound). Got: {css}"
    );
    assert!(
        !css.contains(".drawer .drawer__sr-focus:focus"),
        "Should NOT have descendant form. Got: {css}"
    );
}
