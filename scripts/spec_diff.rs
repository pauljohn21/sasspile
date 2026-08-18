//! spec_diff.rs — Compare two baseline JSON files and report progress/regression.
//!
//! Usage:
//! ```sh
//! RUST_LOG=info rust-script scripts/spec_diff.rs --old spec_baseline_old.json --new spec_baseline_new.json
//! ```
//! ```cargo
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! tracing = "0.1"
//! tracing-subscriber = { version = "0.3", features = ["env-filter"] }
//! ```

use std::env;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BaselineDomain {
    domain: String,
    total: u64,
    passed: u64,
    failed: u64,
    skipped: u64,
    pass_rate: f64,
}

#[derive(Debug, Deserialize)]
struct BaselineReport {
    #[allow(dead_code)]
    timestamp: String,
    #[allow(dead_code)]
    total_hrx: usize,
    total_cases: u64,
    total_passed: u64,
    total_failed: u64,
    total_skipped: u64,
    overall_pass_rate: f64,
    domains: Vec<BaselineDomain>,
}

fn main() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    let span = tracing::info_span!("spec_diff", stage = "spec_diff");
    let _enter = span.enter();

    let args: Vec<String> = env::args().collect();
    let mut old_path = String::new();
    let mut new_path = String::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--old" => {
                i += 1;
                if i < args.len() {
                    old_path = args[i].clone();
                }
            }
            "--new" => {
                i += 1;
                if i < args.len() {
                    new_path = args[i].clone();
                }
            }
            _ => {}
        }
        i += 1;
    }

    if old_path.is_empty() || new_path.is_empty() {
        tracing::error!("Usage: spec_diff.rs --old <old.json> --new <new.json>");
        std::process::exit(1);
    }

    let old_json = fs::read_to_string(&old_path).unwrap_or_else(|e| {
        tracing::error!(path = %old_path, error = %e, "failed to read old baseline");
        std::process::exit(1);
    });
    let new_json = fs::read_to_string(&new_path).unwrap_or_else(|e| {
        tracing::error!(path = %new_path, error = %e, "failed to read new baseline");
        std::process::exit(1);
    });

    let old: BaselineReport = serde_json::from_str(&old_json).unwrap_or_else(|e| {
        tracing::error!(error = %e, "failed to parse old JSON");
        std::process::exit(1);
    });
    let new: BaselineReport = serde_json::from_str(&new_json).unwrap_or_else(|e| {
        tracing::error!(error = %e, "failed to parse new JSON");
        std::process::exit(1);
    });

    // Overall metrics
    tracing::info!(
        stage = "spec_diff",
        old_cases = old.total_cases,
        new_cases = new.total_cases,
        old_passed = old.total_passed,
        new_passed = new.total_passed,
        old_failed = old.total_failed,
        new_failed = new.total_failed,
        old_skipped = old.total_skipped,
        new_skipped = new.total_skipped,
        old_pass_rate = format!("{:.4}", old.overall_pass_rate),
        new_pass_rate = format!("{:.4}", new.overall_pass_rate),
        delta_pass_rate = format!("{:+.4}", new.overall_pass_rate - old.overall_pass_rate),
        "overall comparison"
    );

    // Per-domain breakdown
    for new_domain in &new.domains {
        let old_domain = old.domains.iter().find(|d| d.domain == new_domain.domain);
        let (old_pass, old_fail) = match old_domain {
            Some(d) => (d.passed, d.failed),
            None => (0, 0),
        };
        let delta = new_domain.passed as i64 - old_pass as i64;

        tracing::info!(
            stage = "spec_diff",
            domain = %new_domain.domain,
            old_pass = old_pass,
            new_pass = new_domain.passed,
            old_fail = old_fail,
            new_fail = new_domain.failed,
            delta = delta,
            "domain comparison"
        );
    }

    // Detect improvements and regressions
    let improvements: Vec<&BaselineDomain> = new
        .domains
        .iter()
        .filter(|nd| {
            old.domains
                .iter()
                .find(|od| od.domain == nd.domain)
                .map(|od| nd.passed > od.passed)
                .unwrap_or(true)
        })
        .collect();

    let regressions: Vec<&BaselineDomain> = new
        .domains
        .iter()
        .filter(|nd| {
            old.domains
                .iter()
                .find(|od| od.domain == nd.domain)
                .map(|od| nd.passed < od.passed)
                .unwrap_or(false)
        })
        .collect();

    if !improvements.is_empty() {
        tracing::info!(stage = "spec_diff", count = improvements.len(), "improvements detected");
        for d in &improvements {
            let old_p = old
                .domains
                .iter()
                .find(|od| od.domain == d.domain)
                .map(|od| od.passed)
                .unwrap_or(0);
            tracing::info!(
                stage = "spec_diff",
                domain = %d.domain,
                old_passed = old_p,
                new_passed = d.passed,
                "improved"
            );
        }
    }

    if !regressions.is_empty() {
        tracing::warn!(stage = "spec_diff", count = regressions.len(), "regressions detected");
        for d in &regressions {
            let old_p = old
                .domains
                .iter()
                .find(|od| od.domain == d.domain)
                .map(|od| od.passed)
                .unwrap_or(0);
            tracing::warn!(
                stage = "spec_diff",
                domain = %d.domain,
                old_passed = old_p,
                new_passed = d.passed,
                "regressed"
            );
        }
    }

    if improvements.is_empty() && regressions.is_empty() {
        tracing::info!(stage = "spec_diff", "no changes detected");
    }
}
