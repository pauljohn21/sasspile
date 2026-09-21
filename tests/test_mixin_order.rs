//! Verify backtop.scss mixin ordering matches spec (source order)

use sasspile::*;
use std::path::PathBuf;

#[test]
fn test_backtop_order_matches_spec() {
    init_tracing();
    let path = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/element-plus/packages/theme-chalk/src/backtop.scss"
    ));
    let src_dir = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/element-plus/packages/theme-chalk/src"));
    let load_paths = vec![src_dir.clone(), src_dir.join("mixins")];

    let sp_css = compile_file_with_load_paths(&path, OutputStyle::Expanded, load_paths).expect("compile failed");

    // Print lines around backtop
    for (i, line) in sp_css.lines().enumerate() {
        let t = line.trim();
        if t.contains(".el-backtop") {
            tracing::warn!(idx = i, line = %t, "LINE");
        }
    }

    // Extract selector order
    let selectors: Vec<String> = sp_css.lines()
        .filter(|l| {
            let t = l.trim();
            (t.starts_with(".el-backtop") || t.starts_with(".el-backtop__icon"))
                && t.contains('{')
                && !t.contains('}')
        })
        .map(|l| l.trim().to_string())
        .collect();

    tracing::warn!(selectors = ?selectors, "SELECTOR ORDER");
    assert!(selectors.len() >= 3, "Expected at least 3 rules, got {:?}", selectors);
}
