//! HRX file parser — extracts test cases from `.hrx` files.
//!
//! HRX format uses `<===> filename` as file separators
//! and `===` (70 equals) as sub-directory separators.

// This module is shared across multiple test targets via `#[path = "hrx_parser.rs"]`.
// When compiled as a standalone test target, some public items appear unused.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// A single test case extracted from an HRX file.
#[derive(Debug, Clone)]
pub struct HrxTestCase {
    /// Test name (derived from path or parent directory)
    pub name: String,
    /// Input SCSS source
    pub input: Option<String>,
    /// Expected output CSS
    pub output: Option<String>,
    /// Expected error message (if test expects error)
    pub error: Option<String>,
    /// Options (e.g., `:warning` or `:precision`)
    pub options: Option<String>,
    /// Base path for multi-file tests
    pub base_path: Option<String>,
}

/// Parse an HRX file from its raw text content.
pub fn parse_hrx(content: &str) -> Vec<(String, String)> {
    let mut files = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_content = String::new();

    for line in content.lines() {
        if line.starts_with("<===> ") {
            // Save previous file
            if let Some(name) = current_name.take() {
                // Trim trailing newline from content
                let trimmed = current_content.trim_end_matches('\n').to_string();
                files.push((name, trimmed));
                current_content.clear();
            }
            // Start new file
            let name = line.trim_start_matches("<===> ").trim();
            current_name = Some(name.to_string());
        } else if line.starts_with("=======") {
            // Sub-directory separator — flush current file
            if let Some(name) = current_name.take() {
                let trimmed = current_content.trim_end_matches('\n').to_string();
                files.push((name, trimmed));
                current_content.clear();
            }
            // Skip the separator line
        } else if line.starts_with("<===") && !line.starts_with("<===> ") {
            // Test case separator in HRX — flush and skip
            if let Some(name) = current_name.take() {
                let trimmed = current_content.trim_end_matches('\n').to_string();
                files.push((name, trimmed));
                current_content.clear();
            }
        } else if current_name.is_some() {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    // Flush last file
    if let Some(name) = current_name {
        let trimmed = current_content.trim_end_matches('\n').to_string();
        files.push((name, trimmed));
    }

    files
}

/// Extract test cases from parsed HRX files.
///
/// A test case is identified by the presence of `input.scss`.
/// The expected output is in `output.css`, or `error` for error tests.
pub fn extract_test_cases(hrx_path: &Path) -> Vec<HrxTestCase> {
    let content = match std::fs::read_to_string(hrx_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let files = parse_hrx(&content);

    // Group files by their parent directory
    // Multi-file tests have paths like "subdir/input.scss"
    // Single-file tests just have "input.scss"
    let mut groups: std::collections::HashMap<String, Vec<(String, String)>> =
        std::collections::HashMap::new();

    for (path, content) in &files {
        let parent = if let Some(idx) = path.rfind('/') {
            path[..idx].to_string()
        } else {
            String::new() // root level
        };
        groups.entry(parent).or_default().push((path.clone(), content.clone()));
    }

    let mut test_cases = Vec::new();

    for (dir, group_files) in &groups {
        // Find input.scss
        let input_name = if dir.is_empty() {
            "input.scss".to_string()
        } else {
            format!("{}/input.scss", dir)
        };

        let input = group_files
            .iter()
            .find(|(p, _)| *p == input_name || p.ends_with("/input.scss") || *p == "input.scss")
            .map(|(_, c)| c.clone());

        if input.is_none() {
            continue;
        }

        // Find output.css
        let output_name = if dir.is_empty() {
            "output.css".to_string()
        } else {
            format!("{}/output.css", dir)
        };
        let output = group_files
            .iter()
            .find(|(p, _)| *p == output_name || p.ends_with("/output.css") || *p == "output.css")
            .map(|(_, c)| c.clone());

        // Find error
        let error_name = if dir.is_empty() {
            "error".to_string()
        } else {
            format!("{}/error", dir)
        };
        let error = group_files
            .iter()
            .find(|(p, _)| *p == error_name || p.ends_with("/error") || *p == "error")
            .map(|(_, c)| c.clone());

        // Find options
        let options_name = if dir.is_empty() {
            "options".to_string()
        } else {
            format!("{}/options", dir)
        };
        let options = group_files
            .iter()
            .find(|(p, _)| *p == options_name || p.ends_with("/options") || *p == "options")
            .map(|(_, c)| c.clone());

        let test_name = if dir.is_empty() {
            hrx_path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        } else {
            format!(
                "{}_{}",
                hrx_path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                dir.replace('/', "_")
            )
        };

        test_cases.push(HrxTestCase {
            name: test_name,
            input,
            output,
            error,
            options,
            base_path: if dir.is_empty() {
                None
            } else {
                Some(dir.clone())
            },
        });
    }

    // If no test cases found with the grouping approach, try simple extraction
    if test_cases.is_empty() {
        let input = files.iter().find(|(p, _)| p == "input.scss").map(|(_, c)| c.clone());
        if let Some(input) = input {
            let output = files
                .iter()
                .find(|(p, _)| p == "output.css")
                .map(|(_, c)| c.clone());
            let error = files
                .iter()
                .find(|(p, _)| p == "error")
                .map(|(_, c)| c.clone());
            let options = files
                .iter()
                .find(|(p, _)| p == "options")
                .map(|(_, c)| c.clone());

            let name = hrx_path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            test_cases.push(HrxTestCase {
                name,
                input: Some(input),
                output,
                error,
                options,
                base_path: None,
            });
        }
    }

    test_cases
}

/// Walk a directory and find all `.hrx` files.
pub fn find_hrx_files(dir: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                results.extend(find_hrx_files(&path));
            } else if path.extension().map(|e| e == "hrx").unwrap_or(false) {
                results.push(path);
            }
        }
    }

    results.sort();
    results
}

#[test]
fn test_parse_simple_hrx() {
    let content = "<===> input.scss\na {\n  color: red;\n}\n\n<===> output.css\na {\n  color: red;\n}\n";
    let files = parse_hrx(content);
    assert_eq!(files.len(), 2);
    assert_eq!(files[0].0, "input.scss");
    assert_eq!(files[1].0, "output.css");
}

#[test]
fn test_parse_multifile_hrx() {
    let content = "<===> subdir/input.scss\na { color: red; }\n\n<===> subdir/output.css\na {\n  color: red;\n}\n";
    let files = parse_hrx(content);
    assert_eq!(files.len(), 2);
    assert!(files[0].0.contains("input.scss"));
}

#[test]
fn test_parse_with_error_section() {
    let content = "<===> input.scss\n$a: 1px + \"invalid\";\n\n<===> error\nError: \"invalid\" is not a number.\n";
    let files = parse_hrx(content);
    assert_eq!(files.len(), 2);
    assert_eq!(files[1].0, "error");
}

#[test]
fn test_parse_with_options() {
    let content = "<===> input.scss\na { color: red; }\n\n<===> options\n:warning\n\n<===> output.css\na {\n  color: red;\n}\n";
    let files = parse_hrx(content);
    assert_eq!(files.len(), 3);
}
