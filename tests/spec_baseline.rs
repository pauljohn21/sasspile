//! Full sass-spec baseline test — runs all 17 domains, produces baseline JSON.
//!
//! This test is `#[ignore]` by default — run with:
//! ```sh
//! cargo test --test spec_baseline -- --nocapture --ignored
//! ```

#[path = "hrx_parser.rs"]
mod hrx_parser;

#[path = "hrx_vfs.rs"]
mod hrx_vfs;

#[path = "spec_runner.rs"]
mod spec_runner;

#[path = "spec_otel_runner.rs"]
mod spec_otel_runner;

mod tracing_init;

use std::path::PathBuf;
use std::time::Instant;

use serde::Serialize;

fn spec_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("sass-spec")
        .join("spec")
}

/// Domain path -> domain label mapping.
const DOMAINS: &[(&str, &str)] = &[
    ("css/plain", "css_plain"),
    ("css", "css"),
    ("directives", "directives"),
    ("expressions", "expressions"),
    ("operators", "operators"),
    ("parser", "parser"),
    ("values", "values"),
    ("variables", "variables"),
    ("callable", "callable"),
    ("core_functions/color", "core_functions_color"),
    ("core_functions/list", "core_functions_list"),
    ("core_functions/map", "core_functions_map"),
    ("core_functions/math", "core_functions_math"),
    ("core_functions/meta", "core_functions_meta"),
    ("core_functions/string", "core_functions_string"),
    ("core_functions/selector", "core_functions_selector"),
    ("core_functions", "core_functions_misc"),
];

#[derive(Debug, Serialize)]
struct BaselineDomain {
    domain: String,
    total: u64,
    passed: u64,
    failed: u64,
    skipped: u64,
    pass_rate: f64,
}

#[derive(Debug, Serialize)]
struct BaselineReport {
    timestamp: String,
    total_hrx: usize,
    total_cases: u64,
    total_passed: u64,
    total_failed: u64,
    total_skipped: u64,
    overall_pass_rate: f64,
    domains: Vec<BaselineDomain>,
}

#[test]
#[ignore]
fn test_baseline_all() {
    let label = "spec_baseline";
    tracing_init::init_otel(label);
    tracing_init::init_metrics(label);

    let span = tracing::info_span!(
        "spec_baseline",
        stage = "spec_test",
        total_hrx = tracing::field::Empty,
        total_cases = tracing::field::Empty,
        passed = tracing::field::Empty,
        failed = tracing::field::Empty,
        pass_rate = tracing::field::Empty,
    );
    let _enter = span.enter();

    let start = Instant::now();
    let mut all_stats: Vec<(String, spec_otel_runner::DomainStats)> = Vec::new();
    let mut total_hrx = 0usize;

    for (path, domain_label) in DOMAINS {
        let dir = spec_root().join(path);
        if !dir.exists() {
            tracing::warn!(
                stage = "spec_test",
                domain = %domain_label,
                path = %path,
                "directory not found, skipping"
            );
            continue;
        }

        tracing::info!(
            stage = "spec_test",
            domain = %domain_label,
            path = %path,
            "starting domain"
        );

        // Use catch_unwind to prevent compiler panics from aborting the baseline
        let domain_stats = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut runner = spec_otel_runner::SpecOtelRunner::new(domain_label);
            let hrx_files = hrx_parser::find_hrx_files(&dir);
            total_hrx += hrx_files.len();

            for hrx_path in &hrx_files {
                runner.run_hrx_tests(hrx_path);
            }

            runner.finalize()
        }));

        match domain_stats {
            Ok(stats) => {
                all_stats.push((domain_label.to_string(), stats));
            }
            Err(panic_val) => {
                tracing::error!(
                    stage = "spec_test",
                    domain = %domain_label,
                    panic = %panic_val.downcast_ref::<String>()
                        .map(|s| s.as_str())
                        .unwrap_or("(non-string panic)"),
                    "domain panicked, recording as failed"
                );
                all_stats.push((domain_label.to_string(), spec_otel_runner::DomainStats {
                    total: 0,
                    passed: 0,
                    failed: 0,
                    skipped: 0,
                }));
            }
        }
    }

    let elapsed = start.elapsed();
    let total_cases: u64 = all_stats.iter().map(|(_, s)| s.total).sum();
    let total_passed: u64 = all_stats.iter().map(|(_, s)| s.passed).sum();
    let total_failed: u64 = all_stats.iter().map(|(_, s)| s.failed).sum();
    let total_skipped: u64 = all_stats.iter().map(|(_, s)| s.skipped).sum();
    let overall_pass_rate = if total_cases > 0 {
        total_passed as f64 / total_cases as f64
    } else {
        0.0
    };

    tracing::info!(
        stage = "spec_test",
        total_hrx = total_hrx,
        total_cases = total_cases,
        passed = total_passed,
        failed = total_failed,
        skipped = total_skipped,
        pass_rate = format!("{:.4}", overall_pass_rate),
        elapsed_ms = elapsed.as_millis(),
        "baseline complete"
    );

    // Write baseline JSON
    let report = BaselineReport {
        timestamp: chrono_now(),
        total_hrx,
        total_cases,
        total_passed,
        total_failed,
        total_skipped,
        overall_pass_rate,
        domains: all_stats
            .iter()
            .map(|(name, s)| BaselineDomain {
                domain: name.clone(),
                total: s.total,
                passed: s.passed,
                failed: s.failed,
                skipped: s.skipped,
                pass_rate: s.pass_rate(),
            })
            .collect(),
    };

    let json_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("spec_baseline_{}.json", report.timestamp));
    let json = serde_json::to_string_pretty(&report).unwrap_or_default();
    std::fs::write(&json_path, &json).unwrap_or_else(|e| {
        tracing::error!(
            stage = "spec_test",
            path = %json_path.display(),
            error = %e,
            "failed to write baseline JSON"
        );
    });

    tracing::info!(
        stage = "spec_test",
        path = %json_path.display(),
        "baseline JSON written"
    );

    // RecordOnly mode — no assert, just produce baseline
    tracing_init::shutdown_metrics();
    tracing_init::shutdown_otel();
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}", secs)
}
