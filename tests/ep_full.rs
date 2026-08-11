//! element-plus 全量编译验证测试。

use sasspile::*;
use std::path::PathBuf;

#[test]
fn test_ep_full_stats() {
    init_tracing();
    let dir = PathBuf::from("/Users/pauljohn/rust/element-plus-dev/packages/theme-chalk/src");
    let mut ok = 0;
    let mut fail = 0;
    let mut errors: Vec<(String, String)> = vec![];

    let entries = std::fs::read_dir(&dir).expect("无法读取 element-plus 目录");
    let mut files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "scss"))
        .collect();
    files.sort_by_key(|e| e.path());

    for entry in files {
        let name = entry.file_name().to_string_lossy().to_string();
        match compile_file(&entry.path(), OutputStyle::Expanded) {
            Ok(css) => {
                ok += 1;
                eprintln!("OK   {} ({} bytes)", name, css.len());
            }
            Err(e) => {
                fail += 1;
                let msg = format!("{e}");
                errors.push((name.clone(), msg.clone()));
                eprintln!("FAIL {} : {}", name, msg);
            }
        }
    }

    eprintln!("\n=== 统计 ===");
    eprintln!("通过: {} / {}", ok, ok + fail);
    eprintln!("失败: {}", fail);

    eprintln!("\n=== 错误分类 ===");
    let mut categories: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for (name, err) in &errors {
        let cat = if err.contains("未定义函数") {
            "未定义函数".to_string()
        } else if err.contains("未定义") || err.contains("Undefined") {
            "未定义".to_string()
        } else if err.contains("不是") || err.contains("is not") {
            "类型错误".to_string()
        } else if err.contains("参数") || err.contains("argument") {
            "参数错误".to_string()
        } else if err.contains("解析") || err.contains("Parse") {
            "解析错误".to_string()
        } else if err.contains("求值") || err.contains("eval") {
            "求值错误".to_string()
        } else {
            "其他".to_string()
        };
        categories.entry(cat).or_default().push(name.clone());
    }
    for (cat, files) in &categories {
        eprintln!("{} ({}): {}", cat, files.len(), files.join(", "));
    }

    println!("EP 通过: {}/{}", ok, ok + fail);
}
