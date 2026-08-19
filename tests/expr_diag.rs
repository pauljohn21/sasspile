//! expressions 目录诊断——显示前 N 个失败的摘要。

mod spec_manifest;
use spec_manifest::collect_hrx_files;

use std::path::{Path, PathBuf};

struct HrxCase {
    files: Vec<(String, String)>,
    input_path: String,
    expected_output: String,
    expect_error: bool,
    name: String,
}

fn parse_hrx(content: &str) -> Vec<HrxCase> {
    let mut files: Vec<(String, String)> = Vec::new();
    let mut current_path = String::new();
    let mut current_content = String::new();

    for line in content.lines() {
        if line.starts_with("<===>") {
            if !current_path.is_empty() {
                files.push((current_path.clone(), current_content));
            }
            current_path = line.trim_start_matches("<===>").trim().to_string();
            current_content = String::new();
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }
    if !current_path.is_empty() {
        files.push((current_path, current_content));
    }

    let mut cases = Vec::new();
    for (path, _) in &files {
        if path.ends_with("input.scss") {
            let base = path.strip_suffix("input.scss").unwrap_or(path).to_string();
            let output_path = format!("{base}output.css");
            let error_path = format!("{base}error");

            let expected_output = files
                .iter()
                .find(|(p, _)| p == &output_path)
                .map(|(_, c)| c.clone())
                .unwrap_or_default();
            let expect_error = files.iter().any(|(p, _)| p == &error_path);

            let case_files: Vec<(String, String)> = files
                .iter()
                .filter(|(p, _)| p.ends_with(".scss") || p.ends_with(".css"))
                .map(|(p, c)| (p.clone(), c.clone()))
                .collect();

            let name = base.trim_end_matches('/').to_string();
            cases.push(HrxCase { files: case_files, input_path: path.clone(), expected_output, expect_error, name });
        }
    }
    cases
}

fn run_case(case: &HrxCase, load_paths: &[PathBuf]) -> Option<String> {
    if case.expected_output.is_empty() && !case.expect_error { return None; }
    let total_size: usize = case.files.iter().map(|(_, c)| c.len()).sum();
    if total_size > 50_000 { return Some("TOO_LARGE".to_string()); }

    let tmp_dir = std::env::temp_dir().join(format!("expr-diag-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).ok();
    for (path, content) in &case.files {
        let file_path = tmp_dir.join(path);
        if let Some(parent) = file_path.parent() { std::fs::create_dir_all(parent).ok(); }
        std::fs::write(&file_path, content).ok();
    }

    let input_file = tmp_dir.join(&case.input_path);
    let result = sasspile::compile_file_with_load_paths(&input_file, sasspile::OutputStyle::Expanded, load_paths.to_vec());
    let _ = std::fs::remove_dir_all(&tmp_dir);

    match result {
        Ok(actual) => {
            if actual.trim() == case.expected_output.trim() { None }
            else { Some(format!("--- FAIL: {} ---\nEXPECTED:\n{}\nACTUAL:\n{}\n", case.name, case.expected_output.trim(), actual.trim())) }
        }
        Err(e) => {
            if case.expect_error { None }
            else { Some(format!("--- FAIL: {} ---\nEXPECTED:\n{}\nERROR:\n{}\n", case.name, case.expected_output.trim(), e)) }
        }
    }
}

#[test]
fn expr_fail_details() {
    sasspile::init_tracing();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let expr_dir = spec_root.join("expressions");

    let (files, _) = collect_hrx_files(&expr_dir, &spec_root);

    let mut fail_count = 0;
    let mut shown = 0;
    for file in &files {
        if let Ok(content) = std::fs::read_to_string(file) {
            for case in &parse_hrx(&content) {
                if let Some(diff) = run_case(case, std::slice::from_ref(&spec_root)) {
                    fail_count += 1;
                    if shown < 40 {
                        tracing::info!("\n{diff}");
                        shown += 1;
                    }
                }
            }
        }
    }
    tracing::info!(total_fails = fail_count, shown = shown, "expressions fail summary");
}
