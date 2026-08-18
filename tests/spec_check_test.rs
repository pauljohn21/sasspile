//! Spec dataset conformance test — runs test cases from `spec_dataset.json`
//! through the sasspile compiler using VFS, with OTel Metrics + Trace.
//!
//! This test replaces the external `spec_check.rs` script for CI purposes.
//! Instead of spawning a compiler binary per case, it uses `compile_with_resolver`
//! + `VfsResolver` for in-process compilation — faster and fully traced.
//!
//! Usage:
//! ```sh
//! # Run a specific domain (fast):
//! SPEC_DOMAINS=operators cargo test --test spec_check_test -- --nocapture --ignored
//!
//! # Run all domains (slow, full 20504 cases):
//! cargo test --test spec_check_test -- --nocapture --ignored
//! ```

#[path = "hrx_parser.rs"]
mod hrx_parser;

#[path = "hrx_vfs.rs"]
mod hrx_vfs;

#[path = "spec_runner.rs"]
mod spec_runner;

#[path = "spec_otel_runner.rs"]
mod spec_otel_runner;

#[path = "spec_check_helpers.rs"]
mod spec_check_helpers;

mod tracing_init;

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use spec_check_helpers::{
    load_dataset, run_case_vfs, CheckReport, CheckResult, DomainReport, SpecTestCase,
};

/// Run spec check for one or more domains (or all).
///
/// Control via environment variable `SPEC_DOMAINS`:
/// ```sh
/// # Run specific domains:
/// SPEC_DOMAINS=operators,variables cargo test --test spec_check_test -- --nocapture --ignored
///
/// # Run all domains (slow, full 20504 cases):
/// cargo test --test spec_check_test -- --nocapture --ignored
/// ```
#[test]
#[ignore]
fn spec_check_dataset() {
    let domains_filter: Vec<String> = std::env::var("SPEC_DOMAINS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let label = "spec_check_dataset";
    tracing_init::init_otel(label);
    tracing_init::init_metrics(label);

    let span = tracing::info_span!(
        "spec_check_main",
        stage = "spec_check",
        label = %label,
        domains = ?domains_filter,
    );
    let _enter = span.enter();

    let dataset = load_dataset();

    let run_all = domains_filter.is_empty()
        || domains_filter.iter().any(|d| d == "all");

    let cases: Vec<&SpecTestCase> = if run_all {
        dataset.test_cases.iter().collect()
    } else {
        dataset
            .test_cases
            .iter()
            .filter(|c| domains_filter.contains(&c.domain))
            .collect()
    };

    tracing::info!(
        stage = "spec_check",
        total_in_dataset = dataset.total_cases,
        cases_to_run = cases.len(),
        domains = ?domains_filter,
        "starting spec check"
    );

    let start = Instant::now();
    let mut results: Vec<CheckResult> = Vec::with_capacity(cases.len());
    let mut domain_map: HashMap<String, (usize, usize)> = HashMap::new();

    for (idx, case) in cases.iter().enumerate() {
        if idx % 200 == 0 {
            tracing::info!(
                stage = "spec_check",
                progress = idx,
                total = cases.len(),
                "progress update"
            );
        }

        let result = run_case_vfs(case);

        let entry = domain_map
            .entry(case.domain.clone())
            .or_insert((0, 0));
        entry.0 += 1;
        if result.passed {
            entry.1 += 1;
        }

        results.push(result);
    }

    let elapsed = start.elapsed();

    // Build report
    let total_passed = results.iter().filter(|r| r.passed).count();
    let total_failed = results.iter().filter(|r| !r.passed).count();
    let pass_rate = if !results.is_empty() {
        total_passed as f64 / results.len() as f64
    } else {
        0.0
    };

    let per_domain: Vec<DomainReport> = domain_map
        .iter()
        .map(|(name, (total, passed))| DomainReport {
            domain: name.clone(),
            total: *total,
            passed: *passed,
            failed: total - passed,
            pass_rate: if *total > 0 {
                *passed as f64 / *total as f64
            } else {
                0.0
            },
        })
        .collect();

    let failures: Vec<CheckResult> = results
        .iter()
        .filter(|r| !r.passed)
        .cloned()
        .collect();

    let report = CheckReport {
        timestamp: format!(
            "{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0)
        ),
        total_cases: results.len(),
        total_passed,
        total_failed,
        pass_rate,
        per_domain,
        failures,
    };

    // Write report
    let report_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("spec_check_{}.json", label));
    let json = serde_json::to_string_pretty(&report).unwrap_or_default();
    let _ = std::fs::write(&report_path, &json);

    tracing::info!(
        stage = "spec_check",
        total_cases = report.total_cases,
        total_passed = report.total_passed,
        total_failed = report.total_failed,
        pass_rate = format!("{:.4}", report.pass_rate),
        elapsed_ms = elapsed.as_millis(),
        report_path = %report_path.display(),
        "spec check complete"
    );

    // Log per-domain summary
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

    tracing_init::shutdown_metrics();
    tracing_init::shutdown_otel();

    // RecordOnly mode — no assert, just produce baseline
}
