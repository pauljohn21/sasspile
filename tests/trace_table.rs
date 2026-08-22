use sasspile::{compile_file_with_load_paths, OutputStyle};
use std::path::PathBuf;

#[test]
fn trace_table() {
    sasspile::init_tracing();
    let dir = PathBuf::from("/Users/pauljohn/rust/element-plus-dev/packages/theme-chalk/src");
    let result = compile_file_with_load_paths(
        &dir.join("table.scss"),
        OutputStyle::Expanded,
        vec![dir.clone()],
    );
    match result {
        Ok(css) => tracing::info!(len = css.len(), "OK"),
        Err(e) => tracing::error!(error = %e, "FAIL"),
    }
}
