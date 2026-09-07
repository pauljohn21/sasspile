//! change/scale 子目录诊断——显示失败详情。
mod hrx_support;
use hrx_support::{HrxArchive, HrxEntry, Vfs, parse_hrx as hrx_parse};
use std::path::{Path, PathBuf};

struct HrxCase {
    files: Vec<(String, String)>,
    input_path: String,
    expected_output: String,
    expect_error: bool,
}

fn parse_hrx(content: &str) -> Vec<HrxCase> {
    let Ok(archive) = hrx_parse(content) else { return Vec::new(); };
    let groups: Vec<Vec<HrxEntry>> = {
        let mut groups: Vec<Vec<HrxEntry>> = Vec::new();
        let mut current: Vec<HrxEntry> = Vec::new();
        for entry in archive.entries {
            if entry.path.is_empty() {
                if !current.is_empty() { groups.push(std::mem::take(&mut current)); }
            } else { current.push(entry); }
        }
        if !current.is_empty() { groups.push(current); }
        groups
    };
    let mut cases = Vec::new();
    for group_entries in &groups {
        let group_archive = HrxArchive { entries: group_entries.clone() };
        let vfs = Vfs::from_archive(&group_archive);
        let dirs = vfs.walk();
        let all_files: Vec<(String, String)> = dirs.iter().flat_map(|(dp, files)| {
            files.iter().map(move |(f, c)| {
                if dp == "." { (f.clone(), c.clone()) } else { (format!("{dp}/{f}"), c.clone()) }
            })
        }).filter(|(p, _)| (p.ends_with(".scss") || p.ends_with(".css") || p.ends_with(".sass")) && !p.contains("/sass/")).collect();
        for (dp, files) in &dirs {
            let Some(input_file) = files.iter().find(|(f, _)| f == "input.scss") else { continue; };
            let (input_name, _) = input_file;
            let input_path = if dp == "." { input_name.clone() } else { format!("{dp}/{input_name}") };
            let expected_output = files.iter().find(|(f, _)| f == "output.css").map(|(_, c)| c.clone()).unwrap_or_default();
            let expect_error = files.iter().any(|(f, _)| f == "error");
            cases.push(HrxCase { files: all_files.clone(), input_path, expected_output, expect_error });
        }
    }
    cases
}

fn collect_hrx(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() { collect_hrx(&path, files); }
            else if path.extension().and_then(|s| s.to_str()) == Some("hrx") && let Ok(meta) = std::fs::metadata(&path) && meta.len() < 50_000 {
                files.push(path);
            }
        }
    }
}

fn compile_case(case: &HrxCase, spec_root: &Path, hrx_dir: &Path, hrx_stem: &str) -> Result<String, String> {
    let tmp_dir = std::env::temp_dir().join(format!("cfs-diag-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).ok();
    let case_subdir = tmp_dir.join(hrx_stem);
    std::fs::create_dir_all(&case_subdir).ok();
    for (path, content) in &case.files {
        let target = if path.starts_with(&format!("{hrx_stem}/")) { tmp_dir.join(path) } else { case_subdir.join(path) };
        if let Some(parent) = target.parent() { std::fs::create_dir_all(parent).ok(); }
        std::fs::write(&target, content).ok();
    }
    if let Ok(entries) = std::fs::read_dir(hrx_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if (p.extension().and_then(|s| s.to_str()) == Some("scss") || p.extension().and_then(|s| s.to_str()) == Some("css")) && let Ok(content) = std::fs::read_to_string(&p) {
                let filename = p.file_name().unwrap().to_string_lossy().to_string();
                std::fs::write(tmp_dir.join(&filename), content).ok();
            }
        }
    }
    let input_file = if case.input_path.starts_with(&format!("{hrx_stem}/")) { tmp_dir.join(&case.input_path) } else { case_subdir.join(&case.input_path) };
    let result = sasspile::compile_file_with_load_paths(&input_file, sasspile::OutputStyle::Expanded, vec![spec_root.to_path_buf()]);
    let _ = std::fs::remove_dir_all(&tmp_dir);
    result.map_err(|e| format!("{e}"))
}

fn main_diag(subdir: &str, max: usize) {
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let dir = spec_root.join(subdir);
    let mut files = Vec::new();
    collect_hrx(&dir, &mut files);
    let mut shown = 0;
    for file in &files {
        if let Ok(content) = std::fs::read_to_string(file) {
            let stem = file.file_stem().unwrap().to_string_lossy().to_string();
            for case in &parse_hrx(&content) {
                if shown >= max { break; }
                if case.expected_output.is_empty() && !case.expect_error { continue; }
                let name = case.input_path.strip_suffix("input.scss").unwrap_or(&case.input_path).trim_end_matches('/').to_string();
                match compile_case(case, &spec_root, file.parent().unwrap_or(Path::new(".")), &stem) {
                    Ok(actual) => {
                        let a = actual.trim();
                        let e = case.expected_output.trim();
                        if case.expect_error {
                            shown += 1;
                            eprintln!("ERR_EXP_OK: {stem}/{name}\n  actual: {a}");
                        } else if a != e {
                            shown += 1;
                            eprintln!("DIFF: {stem}/{name}\n  exp: {e}\n  got: {a}");
                        }
                    }
                    Err(err_str) => {
                        if !case.expect_error {
                            shown += 1;
                            eprintln!("ERR: {stem}/{name}\n  {err_str}");
                        }
                    }
                }
            }
        }
        if shown >= max { break; }
    }
}

#[test]
fn change_details() { main_diag("core_functions/color/change", 120); }
#[test]
fn scale_details() { main_diag("core_functions/color/scale", 120); }
