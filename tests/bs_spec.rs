//! Bootstrap 5.3.8 编译验证测试。

use sasspile::*;
use std::path::PathBuf;

fn try_compile(name: &str, file: &str) {
    init_tracing();
    let path = PathBuf::from(format!("/Users/pauljohn/rust/bootstrap-5.3.8/scss/{file}"));
    match compile_file(&path, OutputStyle::Expanded) {
        Ok(css) => tracing::info!(component = name, bytes = css.len(), "编译成功"),
        Err(e) => tracing::error!(component = name, error = %e, "编译失败"),
    }
}

#[test]
fn bs_reboot() {
    try_compile("reboot", "_reboot.scss");
}
#[test]
fn bs_alert() {
    try_compile("alert", "_alert.scss");
}
#[test]
fn bs_badge() {
    try_compile("badge", "_badge.scss");
}
#[test]
fn bs_close() {
    try_compile("close", "_close.scss");
}
#[test]
fn bs_containers() {
    try_compile("containers", "_containers.scss");
}
#[test]
fn bs_grid() {
    try_compile("grid", "_grid.scss");
}
#[test]
fn bs_root() {
    try_compile("root", "_root.scss");
}
#[test]
fn bs_type() {
    try_compile("type", "_type.scss");
}
#[test]
fn bs_buttons() {
    try_compile("buttons", "_buttons.scss");
}
#[test]
fn bs_card() {
    try_compile("card", "_card.scss");
}

#[test]
fn bs_full() {
    init_tracing();
    let path = PathBuf::from("/Users/pauljohn/rust/bootstrap-5.3.8/scss/bootstrap.scss");
    match compile_file(&path, OutputStyle::Expanded) {
        Ok(css) => tracing::info!(bytes = css.len(), "bootstrap.scss 编译成功"),
        Err(e) => tracing::error!(error = %e, "bootstrap.scss 编译失败"),
    }
}

#[test]
fn bs_reboot_only() {
    init_tracing();
    let path = PathBuf::from("/Users/pauljohn/rust/bootstrap-5.3.8/scss/bootstrap-reboot.scss");
    match compile_file(&path, OutputStyle::Expanded) {
        Ok(css) => tracing::info!(bytes = css.len(), "bootstrap-reboot.scss 编译成功"),
        Err(e) => tracing::error!(error = %e, "bootstrap-reboot.scss 编译失败"),
    }
}

#[test]
fn bs_functions() {
    init_tracing();
    let path = PathBuf::from("/Users/pauljohn/rust/bootstrap-5.3.8/scss/_functions.scss");
    match compile_file(&path, OutputStyle::Expanded) {
        Ok(css) => tracing::info!(bytes = css.len(), "_functions.scss 编译成功"),
        Err(e) => tracing::error!(error = %e, "_functions.scss 编译失败"),
    }
}

#[test]
fn bs_variables() {
    init_tracing();
    let path = PathBuf::from("/Users/pauljohn/rust/bootstrap-5.3.8/scss/_variables.scss");
    match compile_file(&path, OutputStyle::Expanded) {
        Ok(css) => tracing::info!(bytes = css.len(), "_variables.scss 编译成功"),
        Err(e) => tracing::error!(error = %e, "_variables.scss 编译失败"),
    }
}
