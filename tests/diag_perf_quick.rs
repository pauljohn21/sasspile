//! —— EP 逐文件计时—— 全部打印（不 tail filter）——

use sasspile::*;
use std::path::PathBuf;
use std::time::Instant;

#[test]
fn diag_time_each_file() {
    let dir = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/element-plus/packages/theme-chalk/src"
    ));
    let entries = std::fs::read_dir(&dir).expect("无法读取目录");
    let mut files: Vec<_> = entries
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "scss"))
        .collect();
    files.sort_by_key(std::fs::DirEntry::path);

    let mut cumulative_ms: u128 = 0;
    let mut max_file = String::new();
    let mut max_ms: u128 = 0;

    for (idx, entry) in files.iter().enumerate() {
        let name = entry.file_name().to_string_lossy().to_string();
        let start = Instant::now();
        match compile_file(&entry.path(), OutputStyle::Expanded) {
            Ok(css) => {
                let elapsed = start.elapsed().as_millis();
                cumulative_ms += elapsed;
                if elapsed > max_ms {
                    max_ms = elapsed;
                    max_file = name.clone();
                }
                // use stderr directly to see output immediately
                eprintln!(
                    "[{:3}/{}] {:40} {:6}ms  {}KB",
                    idx + 1,
                    files.len(),
                    name,
                    elapsed,
                    css.len() / 1024
                );
            }
            Err(e) => {
                eprintln!("[FAIL] {}: {}", name, e);
            }
        }
    }

    eprintln!(
        "\n== TOTAL: {}ms | MAX: {} ({}ms) ==",
        cumulative_ms, max_file, max_ms
    );
}
