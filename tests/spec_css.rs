//! Spec tests for `sass-spec/spec/css/` domain (excluding `plain/`).
//!
//! Covers: selector, media, supports, custom_properties, functions,
//! moz_document, unicode_range, unknown_directive, and root-level CSS.

#[path = "hrx_parser.rs"]
mod hrx_parser;

#[path = "spec_runner.rs"]
mod spec_runner;

use std::path::PathBuf;

fn spec_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("sass-spec")
        .join("spec")
}

#[test]
fn test_css_selector() {
    let dir = spec_root().join("css").join("selector");
    if !dir.exists() {
        return;
    }
    let hrx_files = hrx_parser::find_hrx_files(&dir);
    let mut total_passed = 0;
    let mut total_failed = 0;
    for hrx_path in &hrx_files {
        let results = spec_runner::run_hrx_tests(hrx_path);
        for result in &results {
            if result.passed {
                total_passed += 1;
            } else {
                total_failed += 1;
            }
        }
    }
    tracing::info!(
        stage = "spec_test",
        domain = "css/selector",
        passed = total_passed,
        failed = total_failed,
        "css selector spec tests"
    );
}

#[test]
fn test_css_media() {
    let dir = spec_root().join("css").join("media");
    if !dir.exists() {
        return;
    }
    let hrx_files = hrx_parser::find_hrx_files(&dir);
    let mut total_passed = 0;
    let mut total_failed = 0;
    for hrx_path in &hrx_files {
        let results = spec_runner::run_hrx_tests(hrx_path);
        for result in &results {
            if result.passed {
                total_passed += 1;
            } else {
                total_failed += 1;
            }
        }
    }
    tracing::info!(
        stage = "spec_test",
        domain = "css/media",
        passed = total_passed,
        failed = total_failed,
        "css media spec tests"
    );
}

#[test]
fn test_css_supports() {
    let dir = spec_root().join("css").join("supports");
    if !dir.exists() {
        return;
    }
    let hrx_files = hrx_parser::find_hrx_files(&dir);
    let mut total_passed = 0;
    let mut total_failed = 0;
    for hrx_path in &hrx_files {
        let results = spec_runner::run_hrx_tests(hrx_path);
        for result in &results {
            if result.passed {
                total_passed += 1;
            } else {
                total_failed += 1;
            }
        }
    }
    tracing::info!(
        stage = "spec_test",
        domain = "css/supports",
        passed = total_passed,
        failed = total_failed,
        "css supports spec tests"
    );
}

#[test]
fn test_css_custom_properties() {
    let dir = spec_root().join("css").join("custom_properties");
    if !dir.exists() {
        return;
    }
    let hrx_files = hrx_parser::find_hrx_files(&dir);
    let mut total_passed = 0;
    let mut total_failed = 0;
    for hrx_path in &hrx_files {
        let results = spec_runner::run_hrx_tests(hrx_path);
        for result in &results {
            if result.passed {
                total_passed += 1;
            } else {
                total_failed += 1;
            }
        }
    }
    tracing::info!(
        stage = "spec_test",
        domain = "css/custom_properties",
        passed = total_passed,
        failed = total_failed,
        "css custom_properties spec tests"
    );
}

#[test]
fn test_css_functions() {
    let dir = spec_root().join("css").join("functions");
    if !dir.exists() {
        return;
    }
    let hrx_files = hrx_parser::find_hrx_files(&dir);
    let mut total_passed = 0;
    let mut total_failed = 0;
    for hrx_path in &hrx_files {
        let results = spec_runner::run_hrx_tests(hrx_path);
        for result in &results {
            if result.passed {
                total_passed += 1;
            } else {
                total_failed += 1;
            }
        }
    }
    tracing::info!(
        stage = "spec_test",
        domain = "css/functions",
        passed = total_passed,
        failed = total_failed,
        "css functions spec tests"
    );
}

#[test]
fn test_css_other() {
    let css_dir = spec_root().join("css");
    if !css_dir.exists() {
        return;
    }
    let plain_dir = css_dir.join("plain");
    let known_subdirs = [
        "selector", "media", "supports", "custom_properties", "functions",
    ];
    let hrx_files: Vec<_> = hrx_parser::find_hrx_files(&css_dir)
        .into_iter()
        .filter(|p| {
            if let Some(parent) = p.parent() {
                let is_plain = parent.starts_with(&plain_dir);
                let is_known = known_subdirs.iter().any(|s| parent.ends_with(s));
                !is_plain && !is_known
            } else {
                false
            }
        })
        .collect();
    let mut total_passed = 0;
    let mut total_failed = 0;
    for hrx_path in &hrx_files {
        let results = spec_runner::run_hrx_tests(hrx_path);
        for result in &results {
            if result.passed {
                total_passed += 1;
            } else {
                total_failed += 1;
            }
        }
    }
    tracing::info!(
        stage = "spec_test",
        domain = "css/other",
        passed = total_passed,
        failed = total_failed,
        "css other spec tests"
    );
}
