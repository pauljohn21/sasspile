//! 诊断 rate.scss & 泄漏问题
#[test]
fn test_rate_compilation() {
    sasspile::init_tracing();
    let path = std::path::PathBuf::from("element-plus/packages/theme-chalk/src/rate.scss");
    let css = sasspile::compile_file(
        &path,
        sasspile::OutputStyle::Expanded,
    )
    .expect("rate compile");
    
    // 逐行查找 & 泄漏
    for (i, line) in css.lines().enumerate() {
        if line.starts_with('&') {
            tracing::error!("LINE {}: LEAKED AMPERSAND: {}", i + 1, line.trim());
        }
    }
}
