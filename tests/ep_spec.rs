//! element-plus 编译验证测试。

use sasspile::*;
use std::path::PathBuf;

fn ep_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("ep")
        .join("packages")
        .join("theme-chalk")
        .join("src")
}

#[test]
fn test_ep_button() {
    init_tracing();
    let path = ep_src().join("button.scss");
    match compile_file(&path, OutputStyle::Expanded) {
        Ok(css) => tracing::info!(bytes = css.len(), "编译成功"),
        Err(e) => tracing::error!(error = %e, "button.scss 编译失败"),
    }
}

#[test]
fn test_ep_tag() {
    init_tracing();
    let path = ep_src().join("tag.scss");
    match compile_file(&path, OutputStyle::Expanded) {
        Ok(css) => tracing::info!(bytes = css.len(), "编译成功"),
        Err(e) => tracing::error!(error = %e, "tag.scss 编译失败"),
    }
}

#[test]
fn test_ep_icon() {
    init_tracing();
    let path = ep_src().join("icon.scss");
    match compile_file(&path, OutputStyle::Expanded) {
        Ok(css) => tracing::info!(bytes = css.len(), "编译成功"),
        Err(e) => tracing::error!(error = %e, "icon.scss 编译失败"),
    }
}

#[test]
fn test_ep_function() {
    init_tracing();
    let path = ep_src().join("mixins/function.scss");
    match compile_file(&path, OutputStyle::Expanded) {
        Ok(css) => tracing::info!(bytes = css.len(), "function.scss 编译成功"),
        Err(e) => tracing::error!(error = %e, "function.scss 编译失败"),
    }
}

#[test]
fn test_ep_var() {
    init_tracing();
    let path = ep_src().join("common/var.scss");
    match compile_file(&path, OutputStyle::Expanded) {
        Ok(css) => tracing::info!(bytes = css.len(), "var.scss 编译成功"),
        Err(e) => tracing::error!(error = %e, "var.scss 编译失败"),
    }
}
