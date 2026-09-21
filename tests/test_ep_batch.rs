//! Batch diagnose EP files with high similarity to find quick wins

use sasspile::*;
use std::path::PathBuf;
use std::process::Command;

fn compile_sasspile(path: &PathBuf, src_dir: &PathBuf) -> String {
    let load_paths = vec![src_dir.clone(), src_dir.join("mixins")];
    compile_file_with_load_paths(path, OutputStyle::Expanded, load_paths).expect("compile failed")
}

fn compile_dart(path: &PathBuf, src_dir: &PathBuf) -> String {
    let output = Command::new("/opt/homebrew/bin/sass")
        .arg("--style=expanded").arg("--no-source-map")
        .arg("--load-path").arg(src_dir.clone())
        .arg("--load-path").arg(src_dir.join("mixins"))
        .arg(path)
        .output()
        .expect("dart-sass failed");
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn collect_selectors(css: &str) -> Vec<String> {
    css.lines()
        .filter(|l| l.contains('{') && !l.contains('}'))
        .map(|l| {
            let t = l.trim();
            // Extract just the selector part
            t.split('{').next().unwrap_or(t).trim().to_string()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

#[test]
#[ignore]
fn diag_segmented() {
    init_tracing();
    let src = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/element-plus/packages/theme-chalk/src"));
    let path = src.join("segmented.scss");
    let sp = compile_sasspile(&path, &src);
    let dart = compile_dart(&path, &src);

    let sp_sel = collect_selectors(&sp);
    let dart_sel = collect_selectors(&dart);

    tracing::warn!("=== SEGMENTED SELECTORS ===");
    let max = sp_sel.len().max(dart_sel.len());
    for i in 0..max {
        let s = sp_sel.get(i).map(|s| s.as_str()).unwrap_or("<MISSING>");
        let d = dart_sel.get(i).map(|s| s.as_str()).unwrap_or("<MISSING>");
        let marker = if s != d { " <<<DIFF" } else { "" };
        tracing::warn!(i, sp = %s, dart = %d, "CMP{}", marker);
    }
}

#[test]
#[ignore]
fn diag_pagination() {
    init_tracing();
    let src = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/element-plus/packages/theme-chalk/src"));
    let path = src.join("pagination.scss");
    let sp = compile_sasspile(&path, &src);
    let dart = compile_dart(&path, &src);

    let sp_sel = collect_selectors(&sp);
    let dart_sel = collect_selectors(&dart);

    tracing::warn!("=== PAGINATION SELECTORS ===");
    let max = sp_sel.len().max(dart_sel.len());
    for i in 0..max {
        let s = sp_sel.get(i).map(|s| s.as_str()).unwrap_or("<MISSING>");
        let d = dart_sel.get(i).map(|s| s.as_str()).unwrap_or("<MISSING>");
        let marker = if s != d { " <<<DIFF" } else { "" };
        tracing::warn!(i, sp = %s, dart = %d, "CMP{}", marker);
    }
}

#[test]
#[ignore]
fn diag_popover() {
    init_tracing();
    let src = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/element-plus/packages/theme-chalk/src"));
    let path = src.join("popover.scss");
    let sp = compile_sasspile(&path, &src);
    let dart = compile_dart(&path, &src);

    let sp_sel = collect_selectors(&sp);
    let dart_sel = collect_selectors(&dart);

    tracing::warn!("=== POPOVER SELECTORS ===");
    let max = sp_sel.len().max(dart_sel.len());
    for i in 0..max {
        let s = sp_sel.get(i).map(|s| s.as_str()).unwrap_or("<MISSING>");
        let d = dart_sel.get(i).map(|s| s.as_str()).unwrap_or("<MISSING>");
        let marker = if s != d { " <<<DIFF" } else { "" };
        tracing::warn!(i, sp = %s, dart = %d, "CMP{}", marker);
    }
}
