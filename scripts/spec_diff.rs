//! spec_diff.rs — Compare two baseline JSON files and report progress/regression.
//!
//! Usage:
//! ```sh
//! rust-script scripts/spec_diff.rs --old spec_baseline_old.json --new spec_baseline_new.json
//! ```
//! ```cargo
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```

use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, serde::Deserialize)]
struct BaselineDomain {
    domain: String,
    total: u64,
    passed: u64,
    failed: u64,
    skipped: u64,
    pass_rate: f64,
}

#[derive(Debug, serde::Deserialize)]
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

fn main() {
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
        eprintln!("Usage: spec_diff.rs --old <old.json> --new <new.json>");
        std::process::exit(1);
    }

    let old_json = fs::read_to_string(&old_path).unwrap_or_else(|e| {
        eprintln!("Failed to read {}: {}", old_path, e);
        std::process::exit(1);
    });
    let new_json = fs::read_to_string(&new_path).unwrap_or_else(|e| {
        eprintln!("Failed to read {}: {}", new_path, e);
        std::process::exit(1);
    });

    let old: BaselineReport = serde_json::from_str(&old_json).unwrap_or_else(|e| {
        eprintln!("Failed to parse old JSON: {}", e);
        std::process::exit(1);
    });
    let new: BaselineReport = serde_json::from_str(&new_json).unwrap_or_else(|e| {
        eprintln!("Failed to parse new JSON: {}", e);
        std::process::exit(1);
    });

    println!("# Spec Baseline Diff Report\n");
    println!("| Metric | Old | New | Delta |");
    println!("|--------|-----|-----|-------|");
    println!(
        "| Total cases | {} | {} | {} |",
        old.total_cases,
        new.total_cases,
        diff_str(old.total_cases as i64, new.total_cases as i64)
    );
    println!(
        "| Passed | {} | {} | {} |",
        old.total_passed,
        new.total_passed,
        diff_str(old.total_passed as i64, new.total_passed as i64)
    );
    println!(
        "| Failed | {} | {} | {} |",
        old.total_failed,
        new.total_failed,
        diff_str(old.total_failed as i64, new.total_failed as i64)
    );
    println!(
        "| Skipped | {} | {} | {} |",
        old.total_skipped,
        new.total_skipped,
        diff_str(old.total_skipped as i64, new.total_skipped as i64)
    );
    println!(
        "| Pass rate | {:.4} | {:.4} | {:.4} |",
        old.overall_pass_rate, new.overall_pass_rate, new.overall_pass_rate - old.overall_pass_rate
    );

    println!("\n## Per-Domain Breakdown\n");
    println!("| Domain | Old Pass | New Pass | Old Fail | New Fail | Delta |");
    println!("|--------|----------|----------|----------|----------|-------|");

    for new_domain in &new.domains {
        let old_domain = old.domains.iter().find(|d| d.domain == new_domain.domain);
        let (old_pass, old_fail) = match old_domain {
            Some(d) => (d.passed, d.failed),
            None => (0, 0),
        };
        let delta = new_domain.passed as i64 - old_pass as i64;
        let delta_label = if delta > 0 {
            format!("+{}", delta)
        } else {
            format!("{}", delta)
        };
        println!(
            "| {} | {} | {} | {} | {} | {} |",
            new_domain.domain, old_pass, new_domain.passed, old_fail, new_domain.failed, delta_label
        );
    }

    // Summary
    let new_passed: Vec<&BaselineDomain> = new
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

    let regressed: Vec<&BaselineDomain> = new
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

    println!("\n## Summary\n");
    if !new_passed.is_empty() {
        println!("### New passes:");
        for d in &new_passed {
            let old_p = old
                .domains
                .iter()
                .find(|od| od.domain == d.domain)
                .map(|od| od.passed)
                .unwrap_or(0);
            println!("- {} ({} → {})", d.domain, old_p, d.passed);
        }
    }
    if !regressed.is_empty() {
        println!("\n### Regressions:");
        for d in &regressed {
            let old_p = old
                .domains
                .iter()
                .find(|od| od.domain == d.domain)
                .map(|od| od.passed)
                .unwrap_or(0);
            println!("- {} ({} → {})", d.domain, old_p, d.passed);
        }
    }
}

fn diff_str(old: i64, new: i64) -> String {
    let d = new - old;
    if d > 0 {
        format!("+{}", d)
    } else {
        format!("{}", d)
    }
}
