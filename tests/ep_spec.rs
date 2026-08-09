//! element-plus 编译验证测试。

use sasspile::*;
use std::path::PathBuf;

#[test]
fn test_ep_button() {
    init_tracing();
    let path = PathBuf::from("/Users/pauljohn/rust/element-plus-dev/packages/theme-chalk/src/button.scss");
    match compile_file(&path, OutputStyle::Expanded) {
        Ok(css) => tracing::info!(bytes = css.len(), "编译成功"),
        Err(e) => tracing::error!(error = %e, "button.scss 编译失败"),
    }
}

#[test]
fn test_ep_tag() {
    init_tracing();
    let path = PathBuf::from("/Users/pauljohn/rust/element-plus-dev/packages/theme-chalk/src/tag.scss");
    match compile_file(&path, OutputStyle::Expanded) {
        Ok(css) => tracing::info!(bytes = css.len(), "编译成功"),
        Err(e) => tracing::error!(error = %e, "tag.scss 编译失败"),
    }
}

#[test]
fn test_ep_icon() {
    init_tracing();
    let path = PathBuf::from("/Users/pauljohn/rust/element-plus-dev/packages/theme-chalk/src/icon.scss");
    match compile_file(&path, OutputStyle::Expanded) {
        Ok(css) => tracing::info!(bytes = css.len(), "编译成功"),
        Err(e) => tracing::error!(error = %e, "icon.scss 编译失败"),
    }
}

#[test]
fn test_ep_function() {
    init_tracing();
    let path = PathBuf::from("/Users/pauljohn/rust/element-plus-dev/packages/theme-chalk/src/mixins/function.scss");
    match compile_file(&path, OutputStyle::Expanded) {
        Ok(css) => tracing::info!(bytes = css.len(), "function.scss 编译成功"),
        Err(e) => tracing::error!(error = %e, "function.scss 编译失败"),
    }
}

#[test]
fn test_ep_var() {
    init_tracing();
    let path = PathBuf::from("/Users/pauljohn/rust/element-plus-dev/packages/theme-chalk/src/common/var.scss");
    match compile_file(&path, OutputStyle::Expanded) {
        Ok(css) => tracing::info!(bytes = css.len(), "var.scss 编译成功"),
        Err(e) => tracing::error!(error = %e, "var.scss 编译失败"),
    }
}
