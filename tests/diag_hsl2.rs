//! HSL 失败数统计（不输出具体 diff）。
mod hrx_support;
use hrx_support::{HrxArchive, parse_hrx as hrx_parse};
use std::path::{Path, PathBuf};
use tracing;

struct HrxCase {
    files: Vec<(String, String)>,
    input_path: String,
    expected_output: String,
}

fn parse_hrx<'a>(content: &'a str) -> Vec<HrxCase> {
    let Ok(archive) = hrx_parse(content) else { return Vec::new(); };
    let groups: Vec<Vec<hrx_support::HrxEntry>> = {
        let mut groups: Vec<Vec<hrx_support::HrxEntry>> = Vec::new();
        let mut current: Vec<hrx_support::HrxEntry> = Vec::new();
        for entry in archive.entries {
            if entry.path.is_empty() { if !current.is_empty() { groups.push(std::mem::take(&mut current)); } }
            else { current.push(entry); }
        }
        if !current.is_empty() { groups.push(current); }
        groups
    };
    let mut cases = Vec::new();
    for group_entries in &groups {
        let group_archive = HrxArchive { entries: group_entries.clone() };
        let vfs = hrx_support::Vfs::from_archive(&group_archive);
        let dirs = vfs.walk();
        let all_files: Vec<(String, String)> = dirs.iter().flat_map(|(dp, files)| {
            files.iter().map(move |(f, c)| {
                if dp == "." { (f.clone(), c.clone()) } else { (format!("{dp}/{f}"), c.clone()) }
            })
        }).filter(|(p, _)| (p.ends_with(".scss") || p.ends_with(".css")) && !p.contains("/sass/")).collect();
        for (dp, files) in &dirs {
            let Some(input_file) = files.iter().find(|(f, _)| f == "input.scss") else { continue; };
            let (input_name, _) = input_file;
            let input_path = if dp == "." { input_name.clone() } else { format!("{dp}/{input_name}") };
            let expected_output = files.iter().find(|(f, _)| f == "output.css").map(|(_, c)| c.clone()).unwrap_or_default();
            cases.push(HrxCase { files: all_files.clone(), input_path, expected_output });
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

fn compile_case(case: &HrxCase, spec_root: &Path, hrx_stem: &str) -> Result<String, String> {
    let tmp = std::env::temp_dir().join(format!("dhsl2-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();
    std::fs::create_dir_all(tmp.join(hrx_stem)).ok();
    for (path, content) in &case.files {
        let target = if path.starts_with(&format!("{hrx_stem}/")) { tmp.join(path) } else { tmp.join(hrx_stem).join(path) };
        if let Some(parent) = target.parent() { std::fs::create_dir_all(parent).ok(); }
        std::fs::write(&target, content).ok();
    }
    let input_file = if case.input_path.starts_with(&format!("{hrx_stem}/")) { tmp.join(&case.input_path) } else { tmp.join(hrx_stem).join(&case.input_path) };
    let r = sasspile::compile_file_with_load_paths(&input_file, sasspile::OutputStyle::Expanded, vec![spec_root.to_path_buf()]);
    let _ = std::fs::remove_dir_all(&tmp);
    r.map_err(|e| format!("{e}"))
}

fn count_dir(subdir: &str) -> (usize, usize) {
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let mut files = Vec::new();
    collect_hrx(&spec_root.join(subdir), &mut files);
    let mut pass = 0;
    let mut fail = 0;
    for file in &files {
        if let Ok(content) = std::fs::read_to_string(file) {
            let stem = file.file_stem().expect("unexpected failure in test").to_string_lossy().to_string();
            for case in &parse_hrx(&content) {
                if case.expected_output.is_empty() { continue; }
                match compile_case(case, &spec_root, &stem) {
                    Ok(r) if r.trim() == case.expected_output.trim() => pass += 1,
                    _ => fail += 1,
                }
            }
        }
    }
    tracing::error!("{subdir}: pass={pass} fail={fail}");
    (pass, fail)
}

#[test]
fn hsl_count() {
    let (_, fail_hsl) = count_dir("core_functions/color/hsl");
    let (_, fail_hsla) = count_dir("core_functions/color/hsla");
    tracing::error!("HSL_TOTAL: hsl fail={fail_hsl}, hsla fail={fail_hsla}");
    let _ = (fail_hsl, fail_hsla);
}
