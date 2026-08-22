//! sass-spec 全量统计——使用 manifest 跳过不支持的目录。

mod spec_manifest;

use hrx_auditor::parser::{parse_hrx, HrxArchive, HrxEntry};
use hrx_auditor::vfs::Vfs;
use spec_manifest::SKIP_DIRS;
use std::path::{Path, PathBuf};
use tracing::info;

struct HrxCase {
    files: Vec<(String, String)>,
    input_path: String,
    expected_output: String,
    expect_error: bool,
}

fn parse_hrx_to_cases(content: &str) -> Vec<HrxCase> {
    let archive = match parse_hrx(content) {
        Ok(a) => a,
        Err(_) => return Vec::new(),
    };

    let groups: Vec<Vec<HrxEntry>> = {
        let mut groups: Vec<Vec<HrxEntry>> = Vec::new();
        let mut current: Vec<HrxEntry> = Vec::new();
        for entry in archive.entries {
            if entry.path.is_empty() {
                if !current.is_empty() {
                    groups.push(std::mem::take(&mut current));
                }
            } else {
                current.push(entry);
            }
        }
        if !current.is_empty() {
            groups.push(current);
        }
        groups
    };

    let mut cases = Vec::new();
    for group_entries in &groups {
        let group_archive = HrxArchive {
            entries: group_entries.clone(),
        };
        let vfs = Vfs::from_archive(&group_archive);
        let dirs = vfs.walk();

        let all_files: Vec<(String, String)> = dirs
            .iter()
            .flat_map(|(dir_path, files)| {
                files.iter().map(move |(f, c)| {
                    if dir_path == "." {
                        (f.clone(), c.clone())
                    } else {
                        (format!("{dir_path}/{f}"), c.clone())
                    }
                })
            })
            .filter(|(p, _)| p.ends_with(".scss") || p.ends_with(".css"))
            .collect();

        for (dir_path, files) in &dirs {
            let input_file = files.iter().find(|(f, _)| f == "input.scss");
            if input_file.is_none() {
                continue;
            }

            let (input_name, _) = input_file.unwrap();

            let input_path = if dir_path == "." {
                input_name.clone()
            } else {
                format!("{dir_path}/{input_name}")
            };

            let expected_output = files
                .iter()
                .find(|(f, _)| f == "output.css")
                .map(|(_, c)| c.clone())
                .unwrap_or_default();
            let expect_error = files.iter().any(|(f, _)| f == "error");

            cases.push(HrxCase {
                files: all_files.clone(),
                input_path,
                expected_output,
                expect_error,
            });
        }
    }
    cases
}

fn run_case(case: &HrxCase, load_paths: &[PathBuf]) -> bool {
    if case.expected_output.is_empty() && !case.expect_error {
        return true;
    }

    let total_size: usize = case.files.iter().map(|(_, c)| c.len()).sum();
    if total_size > 50_000 {
        return false;
    }

    let tmp_dir = std::env::temp_dir().join(format!(
        "sass-spec-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).ok();

    for (path, content) in &case.files {
        let file_path = tmp_dir.join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&file_path, content).ok();
    }

    let input_file = tmp_dir.join(&case.input_path);
    let result = scss_rs::compile_file_with_paths(
        &input_file,
        load_paths,
        scss_rs::OutputStyle::Expanded,
    );
    let _ = std::fs::remove_dir_all(&tmp_dir);

    if case.expect_error {
        result.is_err()
    } else {
        match result {
            Ok(actual) => actual.trim() == case.expected_output.trim(),
            Err(_) => false,
        }
    }
}

fn collect_hrx_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_recursive(dir, &mut files);
    files
}

fn collect_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_recursive(&path, files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("hrx")
                && let Ok(meta) = std::fs::metadata(&path)
                    && meta.len() < 100_000 {
                        files.push(path);
                    }
        }
    }
}

fn collect_hrx_files_with_manifest(dir: &Path, spec_root: &Path) -> (Vec<PathBuf>, usize) {
    let all = collect_hrx_files(dir);
    let mut kept = Vec::new();
    let mut skipped = 0;
    for path in all {
        let rel = path
            .strip_prefix(spec_root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
        if SKIP_DIRS.iter().any(|skip| rel.starts_with(skip) || rel == *skip) {
            skipped += 1;
            continue;
        }
        kept.push(path);
    }
    (kept, skipped)
}

fn run_spec_dir(spec_root: &Path, dir_name: &str) -> (usize, usize, usize, usize) {
    let dir = spec_root.join(dir_name);
    if !dir.exists() {
        return (0, 0, 0, 0);
    }

    let (files, skipped) = collect_hrx_files_with_manifest(&dir, spec_root);

    let (mut pass, mut fail, mut skip, mut cases) = (0, 0, 0, 0);
    for file in &files {
        if let Ok(content) = std::fs::read_to_string(file) {
            for case in &parse_hrx_to_cases(&content) {
                cases += 1;
                if case.expected_output.is_empty() && !case.expect_error {
                    skip += 1;
                    continue;
                }
                if run_case(case, &[spec_root.to_path_buf()]) {
                    pass += 1;
                } else {
                    fail += 1;
                }
            }
        }
    }

    let evaluated = cases - skip;
    let pct = pass * 100 / evaluated.max(1);
    info!(
        dir = dir_name,
        pass = pass,
        fail = fail,
        skip = skip,
        skipped_dirs = skipped,
        total = cases,
        pct = pct,
        "sass-spec 目录"
    );
    (pass, fail, skip, cases)
}

#[test]
fn test_sass_spec_full_stats() {
    scss_rs::init_tracing();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");

    let dirs = [
        "variables",
        "values",
        "css",
        "operators",
        "expressions",
        "directives",
        "core_functions",
        "parser",
        "callable",
    ];

    let (mut total_pass, mut total_fail, mut total_skip, mut total_cases) = (0, 0, 0, 0);

    for dir in &dirs {
        let (pass, fail, skip, cases) = run_spec_dir(&spec_root, dir);
        let eval = cases - skip;
        let pct = pass * 100 / eval.max(1);
        info!(dir, pass, fail, skip, total = cases, evaluated = eval, pct, "sass-spec 目录");
        total_pass += pass;
        total_fail += fail;
        total_skip += skip;
        total_cases += cases;
    }

    let evaluated = total_cases - total_skip;
    let overall_pct = total_pass * 100 / evaluated.max(1);
    info!(
        pass = total_pass,
        fail = total_fail,
        skip = total_skip,
        total = total_cases,
        evaluated = evaluated,
        pct = overall_pct,
        "sass-spec 全量统计（已跳过不支持的目录）"
    );
}
