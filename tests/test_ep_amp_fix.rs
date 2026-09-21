//! 验证修复后 EP 文件的 `&` 展开情况。
use sasspile::{Reactor, OutputStyle};
use std::path::PathBuf;

fn compile(rel: &str) -> String {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("element-plus/packages/theme-chalk/src");
    let src_dir = p.clone();
    p.push(rel);
    let scss = std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("read {}: {e:?}", p.display()));
    // 设置 load_paths 让 @import 'mixins/mixins' 能够解析
    Reactor::new(scss)
        .with_load_paths(vec![src_dir])
        .lex().unwrap()
        .parse().unwrap()
        .evaluate().unwrap()
        .serialize(OutputStyle::Expanded)
        .finish()
        .unwrap()
}

#[test]
fn segmented_no_literal_amp() {
    let css = compile("segmented.scss");
    assert!(!css.contains("&."), "Literal '&.' in segmented:\n{}", &css[..css.len().min(2000)]);
    assert!(css.contains(".el-segmented__item-selected"), "Missing combined selector");
    tracing::info!("=== segmented.scss ===\n{}", &css[..css.len().min(4000)]);
}

#[test]
fn timeline_no_literal_amp() {
    let css = compile("timeline.scss");
    assert!(!css.contains("&."), "Literal '&.' in timeline:\n{}", &css[..css.len().min(2000)]);
    tracing::info!("=== timeline.scss ===\n{}", &css[..css.len().min(4000)]);
}

#[test]
fn pagination_no_literal_amp() {
    let css = compile("pagination.scss");
    assert!(!css.contains("&."), "Literal '&.' in pagination:\n{}", &css[..css.len().min(2000)]);
    tracing::info!("=== pagination.scss ===\n{}", &css[..css.len().min(4000)]);
}

#[test]
fn tree_no_literal_amp() {
    let css = compile("tree.scss");
    assert!(!css.contains("&."), "Literal '&.' in tree:\n{}", &css[..css.len().min(2000)]);
    tracing::info!("=== tree.scss ===\n{}", &css[..css.len().min(4000)]);
}

#[test]
fn popover_no_literal_amp() {
    let css = compile("popover.scss");
    assert!(!css.contains("&."), "Literal '&.' in popover:\n{}", &css[..css.len().min(2000)]);
    tracing::info!("=== popover.scss ===\n{}", &css[..css.len().min(4000)]);
}
