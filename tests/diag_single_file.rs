//! —— EP 单文件性能诊断 ——

use sasspile::*;
use std::path::PathBuf;
use std::time::Instant;

#[test]
fn diag_single_drawer() {
    let path = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/element-plus/packages/theme-chalk/src/drawer.scss"
    ));
    let start = Instant::now();
    let css = compile_file(&path, OutputStyle::Expanded).expect("compile failed");
    let elapsed = start.elapsed();
    tracing::info!(file = "drawer.scss", ms = elapsed.as_millis(), bytes = css.len(), "RESULT");
}

#[test]
fn diag_single_cascader() {
    let path = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/element-plus/packages/theme-chalk/src/cascader.scss"
    ));
    let start = Instant::now();
    let css = compile_file(&path, OutputStyle::Expanded).expect("compile failed");
    let elapsed = start.elapsed();
    tracing::info!(file = "cascader.scss", ms = elapsed.as_millis(), bytes = css.len(), "RESULT");
}

#[test]
fn diag_single_date_picker() {
    let path = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/element-plus/packages/theme-chalk/src/date-picker.scss"
    ));
    let start = Instant::now();
    let css = compile_file(&path, OutputStyle::Expanded).expect("compile failed");
    let elapsed = start.elapsed();
    tracing::info!(file = "date-picker.scss", ms = elapsed.as_millis(), bytes = css.len(), "RESULT");
}

#[test]
fn diag_single_menu() {
    let path = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/element-plus/packages/theme-chalk/src/menu.scss"
    ));
    let start = Instant::now();
    let css = compile_file(&path, OutputStyle::Expanded).expect("compile failed");
    let elapsed = start.elapsed();
    tracing::info!(file = "menu.scss", ms = elapsed.as_millis(), bytes = css.len(), "RESULT");
}

#[test]
fn diag_single_button() {
    let path = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/element-plus/packages/theme-chalk/src/button.scss"
    ));
    let start = Instant::now();
    let css = compile_file(&path, OutputStyle::Expanded).expect("compile failed");
    let elapsed = start.elapsed();
    tracing::info!(file = "button.scss", ms = elapsed.as_millis(), bytes = css.len(), "RESULT");
}

#[test]
fn diag_single_transfer() {
    let path = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/element-plus/packages/theme-chalk/src/transfer.scss"
    ));
    let start = Instant::now();
    let css = compile_file(&path, OutputStyle::Expanded).expect("compile failed");
    let elapsed = start.elapsed();
    tracing::info!(file = "transfer.scss", ms = elapsed.as_millis(), bytes = css.len(), "RESULT");
}

#[test]
fn diag_single_descriptions() {
    let path = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/element-plus/packages/theme-chalk/src/descriptions.scss"
    ));
    let start = Instant::now();
    let css = compile_file(&path, OutputStyle::Expanded).expect("compile failed");
    let elapsed = start.elapsed();
    tracing::info!(file = "descriptions.scss", ms = elapsed.as_millis(), bytes = css.len(), "RESULT");
}

#[test]
fn diag_single_pagination() {
    let path = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/element-plus/packages/theme-chalk/src/pagination.scss"
    ));
    let start = Instant::now();
    let css = compile_file(&path, OutputStyle::Expanded).expect("compile failed");
    let elapsed = start.elapsed();
    tracing::info!(file = "pagination.scss", ms = elapsed.as_millis(), bytes = css.len(), "RESULT");
}
