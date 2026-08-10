//! core_functions/color 诊断——显示错误模式统计。

use std::collections::HashMap;
use std::path::Path;

fn parse_hrx(content: &str) -> Vec<(String, String, String)> {
    let mut files: Vec<(String, String)> = Vec::new();
    let mut path = String::new();
    let mut content_buf = String::new();
    for line in content.lines() {
        if line.starts_with("<===>") {
            if !path.is_empty() {
                files.push((path.clone(), content_buf));
            }
            path = line.trim_start_matches("<===>").trim().to_string();
            content_buf = String::new();
        } else {
            content_buf.push_str(line);
            content_buf.push('\n');
        }
    }
    if !path.is_empty() {
        files.push((path, content_buf));
    }
    let mut cases = Vec::new();
    for (p, input) in &files {
        if p.ends_with("input.scss") {
            let base = p.strip_suffix("input.scss").unwrap_or(p).to_string();
            let out_path = format!("{base}output.css");
            let err_path = format!("{base}error");
            let output = files
                .iter()
                .find(|(pp, _)| pp == &out_path)
                .map(|(_, c)| c.clone())
                .unwrap_or_default();
            let has_error = files.iter().any(|(pp, _)| pp == &err_path);
            if !has_error && !output.is_empty() {
                cases.push((
                    base.trim_end_matches('/').to_string(),
                    input.clone(),
                    output,
                ));
            }
        }
    }
    cases
}

fn collect_hrx(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_hrx(&path, files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("hrx") {
                if let Ok(meta) = std::fs::metadata(&path) {
                    if meta.len() < 100_000 {
                        files.push(path);
                    }
                }
            }
        }
    }
}

#[test]
fn color_error_patterns() {
    let dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../sass-spec-main/spec/core_functions/color");
    let mut files = Vec::new();
    collect_hrx(&dir, &mut files);

    let mut pass = 0;
    let mut fail = 0;
    let mut patterns: HashMap<String, usize> = HashMap::new();

    for file in &files {
        if let Ok(content) = std::fs::read_to_string(file) {
            let stem = file.file_stem().unwrap().to_string_lossy().to_string();
            for (name, input, expected) in &parse_hrx(&content) {
                match sasspile::compile_expanded(input) {
                    Ok(actual) => {
                        if actual.trim() == expected.trim() {
                            pass += 1;
                        } else {
                            fail += 1;
                            let a = actual.trim();
                            let e = expected.trim();
                            let key = if a.is_empty() {
                                "empty".to_string()
                            } else if a.lines().next() != e.lines().next() {
                                "first_line".to_string()
                            } else {
                                "other".to_string()
                            };
                            *patterns.entry(format!("diff/{stem}")).or_default() += 1;
                        }
                    }
                    Err(err) => {
                        fail += 1;
                        let err_str = format!("{err}");
                        let prefix = if err_str.contains("未定义") {
                            let func = err_str.split("未定义函数: ").nth(1).unwrap_or("?");
                            format!("undef/{func}")
                        } else if err_str.contains("语法错误") {
                            "syntax".to_string()
                        } else if err_str.contains("求值错误") {
                            "eval".to_string()
                        } else {
                            "other_err".to_string()
                        };
                        *patterns.entry(prefix).or_default() += 1;
                    }
                }
            }
        }
    }

    println!("color: {pass} pass / {fail} fail");
    println!("\n错误模式 (top 20):");
    let mut sorted: Vec<_> = patterns.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    for (k, v) in sorted.iter().take(20) {
        println!("  {v:5} {k}");
    }
}
