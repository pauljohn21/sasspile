//! core_functions 诊断——显示前 N 个失败的摘要。

use std::path::Path;

fn parse_hrx(content: &str) -> Vec<(String, String, String)> {
    let mut files: Vec<(String, String)> = Vec::new();
    let mut path = String::new();
    let mut content_buf = String::new();
    for line in content.lines() {
        if line.starts_with("<===>") {
            if !path.is_empty() { files.push((path.clone(), content_buf)); }
            path = line.trim_start_matches("<===>").trim().to_string();
            content_buf = String::new();
        } else {
            content_buf.push_str(line);
            content_buf.push('\n');
        }
    }
    if !path.is_empty() { files.push((path, content_buf)); }
    let mut cases = Vec::new();
    for (p, input) in &files {
        if p.ends_with("input.scss") {
            let base = p.strip_suffix("input.scss").unwrap_or(p).to_string();
            let out_path = format!("{base}output.css");
            let err_path = format!("{base}error");
            let output = files.iter().find(|(pp,_)| pp==&out_path).map(|(_,c)|c.clone()).unwrap_or_default();
            let has_error = files.iter().any(|(pp,_)| pp==&err_path);
            if !has_error && !output.is_empty() {
                cases.push((base.trim_end_matches('/').to_string(), input.clone(), output));
            }
        }
    }
    cases
}

fn collect_hrx(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() { collect_hrx(&path, files); }
            else if path.extension().and_then(|s| s.to_str()) == Some("hrx") {
                if let Ok(meta) = std::fs::metadata(&path) {
                    if meta.len() < 50_000 { files.push(path); }
                }
            }
        }
    }
}

fn diag(subdir: &str, max_show: usize) {
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sass-spec-main/spec");
    let dir = spec_root.join(subdir);
    let mut files = Vec::new();
    collect_hrx(&dir, &mut files);

    let mut shown = 0;
    let mut err_types: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for file in &files {
        if shown >= max_show { break; }
        if let Ok(content) = std::fs::read_to_string(file) {
            let stem = file.file_stem().unwrap().to_string_lossy().to_string();
            for (name, input, expected) in &parse_hrx(&content) {
                if shown >= max_show { break; }
                match sasspile::compile_expanded(input) {
                    Ok(actual) => {
                        if actual.trim() != expected.trim() {
                            shown += 1;
                            let a = actual.trim();
                            let e = expected.trim();
                            let a_line = a.lines().next().unwrap_or("");
                            let e_line = e.lines().next().unwrap_or("");
                            let key = if a.is_empty() { "empty_output".to_string() }
                                else if a_line != e_line { "first_line_diff".to_string() }
                                else { "other_diff".to_string() };
                            *err_types.entry(key.clone()).or_default() += 1;
                            println!("FAIL {stem}/{name}: {key}");
                            if key == "first_line_diff" {
                                println!("  exp: {e_line}");
                                println!("  act: {a_line}");
                            }
                        }
                    }
                    Err(err) => {
                        shown += 1;
                        let err_str = format!("{err}");
                        let key = if err_str.contains("未定义") { "undefined".to_string() }
                            else if err_str.contains("语法错误") { "syntax".to_string() }
                            else if err_str.contains("求值错误") { "eval".to_string() }
                            else { "other_err".to_string() };
                        *err_types.entry(key.clone()).or_default() += 1;
                        println!("ERROR {stem}/{name}: {key} [{err_str}]");
                    }
                }
            }
        }
    }

    println!("\n错误类型统计 ({subdir}):");
    for (k, v) in &err_types {
        println!("  {k}: {v}");
    }
}

#[test]
fn diag_list() { diag("core_functions/list", 15); }

#[test]
fn diag_selector() { diag("core_functions/selector", 15); }

#[test]
fn diag_color() { diag("core_functions/color", 15); }

#[test]
fn diag_math() { diag("core_functions/math", 15); }

#[test]
fn diag_expressions() { diag("expressions", 15); }

#[test]
fn diag_meta() { diag("core_functions/meta", 15); }

#[test]
fn diag_import() { diag("directives/import", 15); }

#[test]
fn diag_use() { diag("directives/use", 15); }

/// 只统计指定子目录的通过/失败/总数。
fn stats_subdir(subdir: &str) {
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sass-spec-main/spec");
    let dir = spec_root.join(subdir);
    let mut files = Vec::new();
    collect_hrx(&dir, &mut files);
    let mut pass = 0;
    let mut fail = 0;
    let mut cases = 0;
    for file in &files {
        if let Ok(content) = std::fs::read_to_string(file) {
            for (_name, input, expected) in &parse_hrx(&content) {
                cases += 1;
                if expected.trim().is_empty() { continue; }
                match sasspile::compile_expanded(input) {
                    Ok(actual) => {
                        if actual.trim() == expected.trim() { pass += 1; }
                        else { fail += 1; }
                    }
                    Err(_) => { fail += 1; }
                }
            }
        }
    }
    let pct = if cases > 0 { pass * 100 / cases } else { 0 };
    println!("{subdir}: {pass}/{cases} ({pct}%) fail={fail}");
}

#[test]
fn stats_list() { stats_subdir("core_functions/list"); }

#[test]
fn stats_math() { stats_subdir("core_functions/math"); }
