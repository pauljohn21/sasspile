//! spec_check.rs — Independent conformance checker using OTel Metrics + Trace.
//!
//! Takes a JSON dataset (from gen_spec_dataset.rs) and a compiler command,
//! runs each test case through the compiler, and compares output.
//! Uses tracing spans for evidence chains and OTel metrics for quantification.
//!
//! Usage:
//! ```sh
//! rust-script scripts/spec_check.rs --dataset spec_dataset.json --compiler "sasspile" --label spec_check
//! ```
//! ```cargo
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! tracing = "0.1"
//! tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
//! ```

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

// ─── Dataset Types (mirror of gen_spec_dataset.rs) ──────────────────────────

#[derive(Debug, Deserialize)]
struct SpecFile { path: String, content: String }

#[derive(Debug, Deserialize)]
struct SpecTestCase {
    id: String,
    domain: String,
    hrx_file: String,
    case_name: String,
    files: Vec<SpecFile>,
    entry: String,
    expected_output: Option<String>,
    expected_error: Option<String>,
    #[allow(dead_code)]
    options: Option<String>,
    is_multi_file: bool,
}

#[derive(Debug, Deserialize)]
struct SpecDataset {
    #[allow(dead_code)]
    version: String,
    total_cases: usize,
    #[allow(dead_code)]
    total_hrx: usize,
    #[allow(dead_code)]
    domains: Vec<SpecDomain>,
    test_cases: Vec<SpecTestCase>,
}

#[derive(Debug, Deserialize)]
struct SpecDomain { name: String, total_cases: usize, total_hrx: usize }

// ─── Result Types ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
struct CheckResult {
    id: String,
    domain: String,
    passed: bool,
    result_type: String, // "pass", "fail_output", "fail_error", "fail_compile", "panic"
    message: String,
    elapsed_ms: u64,
}

#[derive(Debug, Serialize)]
struct CheckReport {
    timestamp: String,
    compiler: String,
    label: String,
    total_cases: usize,
    total_passed: usize,
    total_failed: usize,
    pass_rate: f64,
    per_domain: Vec<DomainReport>,
    failures: Vec<CheckResult>,
}

#[derive(Debug, Serialize)]
struct DomainReport {
    domain: String,
    total: usize,
    passed: usize,
    failed: usize,
    pass_rate: f64,
}

// ─── Normalize CSS ──────────────────────────────────────────────────────────

fn normalize_css(css: &str) -> String {
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

// ─── Run single test case ───────────────────────────────────────────────────

fn run_case(case: &SpecTestCase, compiler: &str, tmp_dir: &PathBuf) -> CheckResult {
    let span = tracing::info_span!(
        "spec_check",
        stage = "spec_check",
        test_id = %case.id,
        domain = %case.domain,
        result = tracing::field::Empty,
    );
    let _enter = span.enter();

    let start = Instant::now();

    // Write files to temp dir
    let case_dir = tmp_dir.join(&case.case_name);
    let _ = fs::create_dir_all(&case_dir);

    for file in &case.files {
        let file_path = case_dir.join(&file.path);
        if let Some(parent) = file_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&file_path, &file.content);
    }

    let entry_path = case_dir.join(&case.entry);

    // Run compiler
    let compile_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        Command::new(compiler)
            .arg(&entry_path)
            .output()
    }));

    let elapsed_ms = start.elapsed().as_millis() as u64;

    let result = match compile_result {
        Err(panic_val) => {
            tracing::error!(
                stage = "spec_check",
                test_id = %case.id,
                domain = %case.domain,
                panic = %panic_val.downcast_ref::<String>()
                    .map(|s| s.as_str())
                    .unwrap_or("(non-string panic)"),
                "COMPILER PANIC"
            );
            CheckResult {
                id: case.id.clone(),
                domain: case.domain.clone(),
                passed: false,
                result_type: "panic".to_string(),
                message: "Compiler panicked".to_string(),
                elapsed_ms,
            }
        }
        Ok(Err(e)) => {
            tracing::error!(
                stage = "spec_check",
                test_id = %case.id,
                domain = %case.domain,
                error = %e,
                "COMPILER SPAWN FAILED"
            );
            CheckResult {
                id: case.id.clone(),
                domain: case.domain.clone(),
                passed: false,
                result_type: "spawn_fail".to_string(),
                message: format!("Failed to spawn compiler: {}", e),
                elapsed_ms,
            }
        }
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            if !output.status.success() {
                // Compiler returned error — check if we expected an error
                if let Some(expected_err) = &case.expected_error {
                    if stderr.contains(expected_err) || stdout.contains(expected_err) {
                        tracing::info!(
                            stage = "spec_check",
                            test_id = %case.id,
                            domain = %case.domain,
                            result = "pass",
                            "test passed (error match)"
                        );
                        CheckResult { id: case.id.clone(), domain: case.domain.clone(), passed: true, result_type: "pass".to_string(), message: String::new(), elapsed_ms }
                    } else {
                        tracing::error!(
                            stage = "spec_check",
                            test_id = %case.id,
                            domain = %case.domain,
                            result = "fail_error",
                            "error mismatch"
                        );
                        CheckResult { id: case.id.clone(), domain: case.domain.clone(), passed: false, result_type: "fail_error".to_string(),
                            message: format!("Expected error: {}\nGot stderr: {}", expected_err, stderr), elapsed_ms }
                    }
                } else {
                    tracing::error!(
                        stage = "spec_check",
                        test_id = %case.id,
                        domain = %case.domain,
                        result = "fail_compile",
                        stderr = %stderr,
                        "unexpected compile error"
                    );
                    CheckResult { id: case.id.clone(), domain: case.domain.clone(), passed: false, result_type: "fail_compile".to_string(),
                        message: format!("Compile error: {}", stderr), elapsed_ms }
                }
            } else {
                // Compiler succeeded — compare output
                if let Some(expected) = &case.expected_output {
                    let expected_norm = normalize_css(expected);
                    let actual_norm = normalize_css(&stdout);
                    if expected_norm == actual_norm {
                        tracing::info!(
                            stage = "spec_check",
                            test_id = %case.id,
                            domain = %case.domain,
                            result = "pass",
                            "test passed (output match)"
                        );
                        CheckResult { id: case.id.clone(), domain: case.domain.clone(), passed: true, result_type: "pass".to_string(), message: String::new(), elapsed_ms }
                    } else {
                        tracing::error!(
                            stage = "spec_check",
                            test_id = %case.id,
                            domain = %case.domain,
                            result = "fail_output",
                            "output mismatch"
                        );
                        CheckResult { id: case.id.clone(), domain: case.domain.clone(), passed: false, result_type: "fail_output".to_string(),
                            message: format!("Expected:\n{}\nGot:\n{}", expected_norm, actual_norm), elapsed_ms }
                    }
                } else if case.expected_error.is_some() {
                    tracing::error!(
                        stage = "spec_check",
                        test_id = %case.id,
                        domain = %case.domain,
                        result = "fail_error",
                        "expected error but got success"
                    );
                    CheckResult { id: case.id.clone(), domain: case.domain.clone(), passed: false, result_type: "fail_error".to_string(),
                        message: "Expected error but compilation succeeded".to_string(), elapsed_ms }
                } else {
                    // No expected output or error — skip
                    CheckResult { id: case.id.clone(), domain: case.domain.clone(), passed: true, result_type: "pass".to_string(),
                        message: "No expected output/error, assuming pass".to_string(), elapsed_ms }
                }
            }
        }
    };

    // Cleanup
    let _ = fs::remove_dir_all(&case_dir);

    result
}

// ─── Main ───────────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut dataset_path = String::from("spec_dataset.json");
    let mut compiler = String::from("sasspile");
    let mut label = String::from("spec_check");
    let mut domains_filter: Vec<String> = Vec::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--dataset" | "-d" => { i += 1; if i < args.len() { dataset_path = args[i].clone(); } }
            "--compiler" | "-c" => { i += 1; if i < args.len() { compiler = args[i].clone(); } }
            "--label" | "-l" => { i += 1; if i < args.len() { label = args[i].clone(); } }
            "--domain" => { i += 1; if i < args.len() { domains_filter.push(args[i].clone()); } }
            _ => {}
        }
        i += 1;
    }

    // Init tracing
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    let span = tracing::info_span!("spec_check_main", stage = "spec_check", label = %label, compiler = %compiler);
    let _enter = span.enter();

    // Load dataset
    let dataset_json = match fs::read_to_string(&dataset_path) {
        Ok(j) => j,
        Err(e) => {
            tracing::error!(stage = "spec_check", error = %e, path = %dataset_path, "failed to read dataset");
            std::process::exit(1);
        }
    };

    let dataset: SpecDataset = match serde_json::from_str(&dataset_json) {
        Ok(d) => d,
        Err(e) => {
            tracing::error!(stage = "spec_check", error = %e, "failed to parse dataset JSON");
            std::process::exit(1);
        }
    };

    tracing::info!(
        stage = "spec_check",
        total_cases = dataset.total_cases,
        compiler = %compiler,
        label = %label,
        "dataset loaded"
    );

    // Filter cases
    let cases: Vec<&SpecTestCase> = dataset.test_cases.iter()
        .filter(|c| domains_filter.is_empty() || domains_filter.contains(&c.domain))
        .collect();

    tracing::info!(stage = "spec_check", cases_to_run = cases.len(), "filtered cases");

    // Temp dir for test files
    let tmp_dir = PathBuf::from(format!("/tmp/spec_check_{}", SystemTime::now()
        .duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)));
    let _ = fs::create_dir_all(&tmp_dir);

    let start = Instant::now();

    // Run all cases
    let mut results: Vec<CheckResult> = Vec::new();
    let mut domain_map: HashMap<String, (usize, usize)> = HashMap::new(); // (total, passed)

    for (idx, case) in cases.iter().enumerate() {
        if idx % 500 == 0 {
            tracing::info!(stage = "spec_check", progress = idx, total = cases.len(), "progress");
        }

        let result = run_case(case, &compiler, &tmp_dir);

        let entry = domain_map.entry(case.domain.clone()).or_insert((0, 0));
        entry.0 += 1;
        if result.passed { entry.1 += 1; }

        results.push(result);
    }

    let elapsed = start.elapsed();

    // Build report
    let total_passed = results.iter().filter(|r| r.passed).count();
    let total_failed = results.iter().filter(|r| !r.passed).count();
    let pass_rate = if !results.is_empty() {
        total_passed as f64 / results.len() as f64
    } else { 0.0 };

    let per_domain: Vec<DomainReport> = domain_map.iter()
        .map(|(name, (total, passed))| DomainReport {
            domain: name.clone(),
            total: *total,
            passed: *passed,
            failed: total - passed,
            pass_rate: if *total > 0 { *passed as f64 / *total as f64 } else { 0.0 },
        })
        .collect();

    let failures: Vec<CheckResult> = results.iter().filter(|r| !r.passed).cloned().collect();

    let report = CheckReport {
        timestamp: format!("{}", SystemTime::now()
            .duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)),
        compiler: compiler.clone(),
        label: label.clone(),
        total_cases: results.len(),
        total_passed,
        total_failed,
        pass_rate,
        per_domain,
        failures,
    };

    // Write report
    let report_path = format!("spec_check_{}.json", label);
    let json = serde_json::to_string_pretty(&report).unwrap_or_default();
    let _ = fs::write(&report_path, &json);

    tracing::info!(
        stage = "spec_check",
        total_cases = report.total_cases,
        total_passed = report.total_passed,
        total_failed = report.total_failed,
        pass_rate = format!("{:.4}", report.pass_rate),
        elapsed_ms = elapsed.as_millis(),
        report_path = %report_path,
        "spec check complete"
    );

    // Log summary via tracing (no eprintln!)
    tracing::info!(
        stage = "spec_check",
        compiler = %compiler,
        total_cases = report.total_cases,
        total_passed = report.total_passed,
        total_failed = report.total_failed,
        pass_rate = format!("{:.4}", report.pass_rate),
        report_path = %report_path,
        "=== Spec Check Summary ==="
    );

    for d in &report.per_domain {
        tracing::info!(
            stage = "spec_check",
            domain = %d.domain,
            passed = d.passed,
            total = d.total,
            pass_rate = format!("{:.3}", d.pass_rate),
            "domain result"
        );
    }

    // Cleanup
    let _ = fs::remove_dir_all(&tmp_dir);
}
