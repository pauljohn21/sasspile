//! Verification test: HRX parser on real sass-spec files.
//!
//! This test verifies that the HRX parser can correctly parse
//! actual `.hrx` files from the sass-spec test suite.

#[path = "hrx_parser.rs"]
mod hrx_parser;

use std::path::PathBuf;

fn spec_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("sass-spec")
        .join("spec")
}

#[test]
fn verify_hrx_parser_on_boolean_operations() {
    let hrx_path = spec_root().join("css/plain/boolean_operations.hrx");
    assert!(hrx_path.exists(), "HRX file not found: {:?}", hrx_path);

    let content = std::fs::read_to_string(&hrx_path).expect("Failed to read HRX file");
    let files = hrx_parser::parse_hrx(&content);

    assert!(!files.is_empty(), "No files parsed from HRX");

    // Should have input.scss and output.css
    let has_input = files.iter().any(|(p, _)| p.ends_with("input.scss"));
    assert!(has_input, "input.scss not found in parsed HRX");

    let has_output = files.iter().any(|(p, _)| p.ends_with("output.css"));
    assert!(has_output, "output.css not found in parsed HRX");
}

#[test]
fn verify_hrx_parser_on_slash() {
    let hrx_path = spec_root().join("css/plain/slash.hrx");
    assert!(hrx_path.exists(), "HRX file not found: {:?}", hrx_path);

    let content = std::fs::read_to_string(&hrx_path).expect("Failed to read HRX file");
    let files = hrx_parser::parse_hrx(&content);

    // slash.hrx has sub-directories, so multiple test cases
    assert!(files.len() >= 4, "Expected at least 4 files, got {}", files.len());
}

#[test]
fn verify_extract_test_cases_boolean_operations() {
    let hrx_path = spec_root().join("css/plain/boolean_operations.hrx");
    assert!(hrx_path.exists(), "HRX file not found: {:?}", hrx_path);

    let cases = hrx_parser::extract_test_cases(&hrx_path);
    assert!(!cases.is_empty(), "No test cases extracted");

    let case = &cases[0];
    assert!(case.input.is_some(), "Test case has no input");
    assert!(case.output.is_some(), "Test case has no output");
    assert!(!case.input.as_ref().unwrap().is_empty(), "Input is empty");
    assert!(!case.output.as_ref().unwrap().is_empty(), "Output is empty");
}

#[test]
fn verify_extract_test_cases_slash() {
    let hrx_path = spec_root().join("css/plain/slash.hrx");
    assert!(hrx_path.exists(), "HRX file not found: {:?}", hrx_path);

    let cases = hrx_parser::extract_test_cases(&hrx_path);
    assert!(cases.len() >= 2, "Expected at least 2 test cases, got {}", cases.len());
}

#[test]
fn verify_find_hrx_files() {
    let plain_dir = spec_root().join("css/plain");
    assert!(plain_dir.exists(), "css/plain directory not found");

    let hrx_files = hrx_parser::find_hrx_files(&plain_dir);
    assert!(!hrx_files.is_empty(), "No HRX files found in css/plain");
    assert!(hrx_files.len() >= 5, "Expected at least 5 HRX files, got {}", hrx_files.len());
}
