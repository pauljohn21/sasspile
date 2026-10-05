//! 诊断 selector 嵌套内 `&.class` 和 `>` 组合子丢失问题
#![allow(clippy::unwrap_used)]

#[test]
fn test_multiple_parent_ampersand_gt() {
    sasspile::init_tracing();
    let scss = r#"
        .el-drawer {
            &.ltr, &.rtl {
                > .el-drawer__dragger {
                    height: 100%;
                    width: 3px;
                    top: 0;
                    bottom: 0;
                    cursor: ew-resize;

                    &::before {
                        top: 0;
                        bottom: 0;
                        width: 3px;
                    }
                }
            }
        }
    "#;
    let css = sasspile::compile_expanded(scss).unwrap();
    tracing::info!("OUTPUT1:\n{css}");

    // Expected: .el-drawer.ltr > .el-drawer__dragger, .el-drawer.rtl > .el-drawer__dragger { ... }
    // Expected: .el-drawer.ltr > .el-drawer__dragger::before, .el-drawer.rtl > .el-drawer__dragger::before { ... }
    assert!(
        css.contains(".el-drawer.ltr > .el-drawer__dragger"),
        "Expected `.el-drawer.ltr > .el-drawer__dragger`. Got: {css}"
    );
    assert!(
        css.contains(".el-drawer.ltr > .el-drawer__dragger::before"),
        "Expected `el-drawer.ltr > .el-drawer__dragger::before`. Got: {css}"
    );
    assert!(
        !css.contains(".el-drawer.ltr .el-drawer.ltr"),
        "Should NOT have duplicate parent descendant. Got: {css}"
    );
}

#[test]
fn test_bem_compound_before_pseudo() {
    sasspile::init_tracing();
    let scss = r#"
        .drawer {
            &__dragger {
                position: absolute;
                background-color: transparent;
                transition: all 0.2s;

                &::before {
                    content: '';
                    position: absolute;
                    background-color: transparent;
                    transition: all 0.2s;
                }
            }
        }
    "#;
    let css = sasspile::compile_expanded(scss).unwrap();
    tracing::info!("OUTPUT2:\n{css}");

    assert!(
        css.contains(".drawer__dragger::before"),
        "Expected compound .drawer__dragger::before. Got: {css}"
    );
    assert!(
        !css.contains(".drawer .drawer__dragger"),
        "Should NOT have descendant .drawer .drawer__dragger. Got: {css}"
    );
}

#[test]
fn test_when_modifier_inside_parent() {
    sasspile::init_tracing();
    // Simulating `.el-drawer { &.ltr > .el-drawer__dragger { ... } }`
    let scss = r#"
        .parent {
            &.ltr, &.rtl {
                color: red;
                > .child {
                    height: 100%;
                    &::before {
                        top: 0;
                    }
                }
            }
        }
    "#;
    let css = sasspile::compile_expanded(scss).unwrap();
    tracing::info!("OUTPUT3:\n{css}");

    // Expected: .parent.ltr, .parent.rtl { color: red; }
    // Expected: .parent.ltr > .child, .parent.rtl > .child { height: 100%; }
    // Expected: .parent.ltr > .child::before, .parent.rtl > .child::before { top: 0; }
    assert!(
        css.contains(".parent.ltr") && css.contains(".parent.rtl"),
        "Expected .parent.ltr and .parent.rtl. Got: {css}"
    );
    assert!(
        css.contains(".parent.ltr > .child") || css.contains(".parent.rtl > .child"),
        "Expected `.parent.ltr > .child` combinator. Got: {css}"
    );
}
