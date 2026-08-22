use hrx_auditor::parser::{parse_hrx as hrx_parse, HrxArchive, HrxEntry};
use hrx_auditor::vfs::Vfs;
use sasspile::{compile_file_with_load_paths, OutputStyle};
use std::path::PathBuf;

/// 从 HRX 内容编译所有 entry，返回结果列表。
fn compile_hrx(hrx: &str) -> Vec<(String, Result<String, String>, Option<String>, bool)> {
    let archive = match hrx_parse(hrx) {
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

    let mut results = Vec::new();
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
            let (input_name, _input_content) = input_file.unwrap();
            let expected_output = files
                .iter()
                .find(|(f, _)| f == "output.css")
                .map(|(_, c)| c.clone());
            let expect_error = files.iter().any(|(f, _)| f == "error");
            let name = if dir_path == "." {
                String::new()
            } else {
                dir_path.clone()
            };

            let tmp_dir = std::env::temp_dir().join("sasspile_diag");
            let _ = std::fs::create_dir_all(&tmp_dir);
            for (path, content) in &all_files {
                let full = tmp_dir.join(path);
                if let Some(parent) = full.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(&full, content);
            }

            let input_path = if dir_path == "." {
                tmp_dir.join(input_name)
            } else {
                tmp_dir.join(format!("{dir_path}/{input_name}"))
            };

            let result = compile_file_with_load_paths(
                &input_path,
                OutputStyle::Expanded,
                vec![tmp_dir.clone()],
            ).map_err(|e| format!("{e}"));

            results.push((name, result, expected_output, expect_error));
        }
    }
    results
}

#[test]
fn diag_forward_extend() {
    sasspile::init_tracing();
    let spec_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");

    // error 版本
    let hrx_file = spec_root.join("directives/forward/error/extend.hrx");
    let content = std::fs::read_to_string(&hrx_file).expect("read hrx");
    let mut pass = 0;
    let mut fail = 0;
    for (name, result, expected, expect_error) in compile_hrx(&content) {
        let ok = if expect_error {
            result.is_err()
        } else {
            result.is_ok() && expected.is_some() && result.as_ref().unwrap().trim() == expected.as_ref().unwrap().trim()
        };
        if ok {
            pass += 1;
        } else {
            fail += 1;
            match (&result, expect_error) {
                (Ok(css), true) => tracing::warn!(name = %name, css = %css, "EXPECTED ERROR but got OK"),
                (Err(e), true) => tracing::warn!(name = %name, error = %e, "wrong error"),
                (Ok(css), false) => tracing::warn!(name = %name, css = %css, expected = ?expected, "CONTENT DIFF"),
                (Err(e), false) => tracing::warn!(name = %name, error = %e, "UNEXPECTED ERROR"),
            }
        }
    }
    tracing::info!(pass = pass, fail = fail, "forward/error/extend summary");

    // 非 error 版本
    let hrx_file2 = spec_root.join("directives/forward/extend.hrx");
    let content2 = std::fs::read_to_string(&hrx_file2).expect("read hrx2");
    let mut pass2 = 0;
    let mut fail2 = 0;
    for (name, result, expected, expect_error) in compile_hrx(&content2) {
        let ok = if expect_error {
            result.is_err()
        } else {
            result.is_ok() && expected.is_some() && result.as_ref().unwrap().trim() == expected.as_ref().unwrap().trim()
        };
        if ok {
            pass2 += 1;
        } else {
            fail2 += 1;
            match (&result, expect_error) {
                (Ok(css), true) => tracing::warn!(name = %name, css = %css, "EXPECTED ERROR but got OK"),
                (Err(e), true) => tracing::warn!(name = %name, error = %e, "wrong error"),
                (Ok(css), false) => tracing::warn!(name = %name, css = %css, expected = ?expected, "CONTENT DIFF"),
                (Err(e), false) => tracing::warn!(name = %name, error = %e, "UNEXPECTED ERROR"),
            }
        }
    }
    tracing::info!(pass = pass2, fail = fail2, "forward/extend summary");
}
