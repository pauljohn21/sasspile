//! Shared types and helpers for spec_check_test.
//!
//! Extracted to keep spec_check_test.rs under 500 lines.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use sasspile::compile_with_resolver;

#[path = "hrx_vfs.rs"]
mod hrx_vfs;

// ─── Dataset Types ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SpecFile {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct SpecTestCase {
    pub id: String,
    pub domain: String,
    #[allow(dead_code)]
    pub hrx_file: String,
    #[allow(dead_code)]
    pub case_name: String,
    pub files: Vec<SpecFile>,
    pub entry: String,
    pub expected_output: Option<String>,
    pub expected_error: Option<String>,
    #[allow(dead_code)]
    pub options: Option<String>,
    #[allow(dead_code)]
    pub is_multi_file: bool,
}

#[derive(Debug, Deserialize)]
pub struct SpecDataset {
    #[allow(dead_code)]
    pub version: String,
    pub total_cases: usize,
    #[allow(dead_code)]
    pub total_hrx: usize,
    #[allow(dead_code)]
    pub domains: Vec<SpecDomain>,
    pub test_cases: Vec<SpecTestCase>,
}

#[derive(Debug, Deserialize)]
pub struct SpecDomain {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub total_cases: usize,
    #[allow(dead_code)]
    pub total_hrx: usize,
}

// ─── Result Types ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct CheckResult {
    pub id: String,
    pub domain: String,
    pub passed: bool,
    pub result_type: String,
    pub message: String,
    pub elapsed_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct CheckReport {
    pub timestamp: String,
    pub total_cases: usize,
    pub total_passed: usize,
    pub total_failed: usize,
    pub pass_rate: f64,
    pub per_domain: Vec<DomainReport>,
    pub failures: Vec<CheckResult>,
}

#[derive(Debug, Serialize)]
pub struct DomainReport {
    pub domain: String,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub pass_rate: f64,
}

// ─── Dataset Loading ────────────────────────────────────────────────────────

pub fn load_dataset() -> SpecDataset {
    let span = tracing::info_span!("load_dataset", stage = "spec_check");
    let _enter = span.enter();

    let dataset_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec_dataset.json");
    let json = std::fs::read_to_string(&dataset_path).unwrap_or_else(|e| {
        tracing::error!(
            stage = "spec_check",
            error = %e,
            path = %dataset_path.display(),
            "failed to read spec_dataset.json"
        );
        panic!("spec_dataset.json not found at {}", dataset_path.display());
    });

    serde_json::from_str(&json).unwrap_or_else(|e| {
        tracing::error!(stage = "spec_check", error = %e, "failed to parse dataset JSON");
        panic!("invalid spec_dataset.json: {}", e);
    })
}

// ─── Normalize CSS ──────────────────────────────────────────────────────────

pub fn normalize_css(css: &str) -> String {
    let mut result = String::new();
    for line in css.lines() {
        let trimmed = line.trim_end();
        if !trimmed.is_empty() {
            result.push_str(trimmed);
            result.push('\n');
        }
    }
    result.trim_end().to_string()
}

// ─── Run Single Test Case via VFS ───────────────────────────────────────────

pub fn run_case_vfs(case: &SpecTestCase) -> CheckResult {
    let span = tracing::info_span!(
        "spec_check_case",
        stage = "spec_check",
        test_id = %case.id,
        domain = %case.domain,
        result = tracing::field::Empty,
    );
    let _enter = span.enter();

    let start = Instant::now();

    // Build VFS from case files
    let entry_dir = if let Some(idx) = case.entry.rfind('/') {
        &case.entry[..idx]
    } else {
        ""
    };

    let mut vfs_map: HashMap<String, String> = HashMap::new();
    for file in &case.files {
        let normalized = if entry_dir.is_empty() {
            file.path.clone()
        } else if file.path.starts_with(&format!("{}/", entry_dir)) {
            file.path
                .strip_prefix(&format!("{}/", entry_dir))
                .unwrap_or(&file.path)
                .to_string()
        } else {
            file.path.clone()
        };
        vfs_map.insert(normalized, file.content.clone());
    }

    let entry_input = case
        .files
        .iter()
        .find(|f| f.path == case.entry)
        .map(|f| f.content.as_str())
        .unwrap_or("");

    let vfs = hrx_vfs::HrxVfs {
        files: vfs_map,
        input_path: "input.scss".to_string(),
    };
    let mut resolver = hrx_vfs::VfsResolver::new(vfs);

    let compile_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        compile_with_resolver(entry_input, &mut resolver)
    }));

    let elapsed_ms = start.elapsed().as_millis() as u64;

    let result = match compile_result {
        Err(panic_val) => {
            let panic_msg = panic_val
                .downcast_ref::<String>()
                .map(|s| s.as_str())
                .unwrap_or("(non-string panic)");
            tracing::error!(
                stage = "spec_check",
                test_id = %case.id,
                domain = %case.domain,
                panic = %panic_msg,
                "COMPILER PANIC"
            );
            CheckResult {
                id: case.id.clone(),
                domain: case.domain.clone(),
                passed: false,
                result_type: "panic".to_string(),
                message: format!("Compiler panicked: {}", panic_msg),
                elapsed_ms,
            }
        }
        Ok(Err(e)) => {
            let error_str = e.to_string();
            if let Some(expected_err) = &case.expected_error {
                if error_str.contains(expected_err) || expected_err.contains(&error_str) {
                    tracing::info!(
                        stage = "spec_check",
                        test_id = %case.id,
                        domain = %case.domain,
                        result = "pass",
                        "passed (error match)"
                    );
                    CheckResult {
                        id: case.id.clone(),
                        domain: case.domain.clone(),
                        passed: true,
                        result_type: "pass".to_string(),
                        message: String::new(),
                        elapsed_ms,
                    }
                } else {
                    tracing::error!(
                        stage = "spec_check",
                        test_id = %case.id,
                        domain = %case.domain,
                        result = "fail_error",
                        "error mismatch"
                    );
                    CheckResult {
                        id: case.id.clone(),
                        domain: case.domain.clone(),
                        passed: false,
                        result_type: "fail_error".to_string(),
                        message: format!("Expected error: {}\nGot: {}", expected_err, error_str),
                        elapsed_ms,
                    }
                }
            } else {
                tracing::error!(
                    stage = "spec_check",
                    test_id = %case.id,
                    domain = %case.domain,
                    result = "fail_compile",
                    error = %error_str,
                    "unexpected compile error"
                );
                CheckResult {
                    id: case.id.clone(),
                    domain: case.domain.clone(),
                    passed: false,
                    result_type: "fail_compile".to_string(),
                    message: format!("Compile error: {}", error_str),
                    elapsed_ms,
                }
            }
        }
        Ok(Ok(actual_output)) => {
            if let Some(expected) = &case.expected_output {
                let expected_norm = normalize_css(expected);
                let actual_norm = normalize_css(&actual_output);
                if expected_norm == actual_norm {
                    tracing::info!(
                        stage = "spec_check",
                        test_id = %case.id,
                        domain = %case.domain,
                        result = "pass",
                        "passed (output match)"
                    );
                    CheckResult {
                        id: case.id.clone(),
                        domain: case.domain.clone(),
                        passed: true,
                        result_type: "pass".to_string(),
                        message: String::new(),
                        elapsed_ms,
                    }
                } else {
                    tracing::error!(
                        stage = "spec_check",
                        test_id = %case.id,
                        domain = %case.domain,
                        result = "fail_output",
                        "output mismatch"
                    );
                    CheckResult {
                        id: case.id.clone(),
                        domain: case.domain.clone(),
                        passed: false,
                        result_type: "fail_output".to_string(),
                        message: format!("Expected:\n{}\nGot:\n{}", expected_norm, actual_norm),
                        elapsed_ms,
                    }
                }
            } else if case.expected_error.is_some() {
                tracing::error!(
                    stage = "spec_check",
                    test_id = %case.id,
                    domain = %case.domain,
                    result = "fail_error",
                    "expected error but got success"
                );
                CheckResult {
                    id: case.id.clone(),
                    domain: case.domain.clone(),
                    passed: false,
                    result_type: "fail_error".to_string(),
                    message: "Expected error but compilation succeeded".to_string(),
                    elapsed_ms,
                }
            } else {
                tracing::warn!(
                    stage = "spec_check",
                    test_id = %case.id,
                    domain = %case.domain,
                    "no expected output/error, assuming pass"
                );
                CheckResult {
                    id: case.id.clone(),
                    domain: case.domain.clone(),
                    passed: true,
                    result_type: "pass".to_string(),
                    message: "No expected output/error".to_string(),
                    elapsed_ms,
                }
            }
        }
    };

    tracing::Span::current().record("result", &result.result_type);
    result
}
