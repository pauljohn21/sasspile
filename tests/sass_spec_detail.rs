//! Aggregated sass-spec report — per top-level directory pass rates.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use sasspile_rx::compile;

const SPEC_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/sass-spec/spec");

const EXCLUDED_DIRS: &[&str] = &[
    "libsass",
    "libsass-closed-issues",
    "libsass-todo-issues",
    "libsass-todo-tests",
    "non_conformant",
];

fn parse_hrx(content: &str) -> Vec<(String, String)> {
    let separator = "<===> ";
    let mut cases = Vec::new();

    let mut current_input: Option<String> = None;
    let mut current_output: Option<String> = None;
    let mut in_input = false;
    let mut in_output = false;
    let mut input_buf = String::new();
    let mut output_buf = String::new();

    for line in content.lines() {
        if line.starts_with(separator) {
            let path = line[separator.len()..].trim();

            if in_input {
                current_input = Some(input_buf.trim().to_string());
                input_buf.clear();
                in_input = false;
            } else if in_output {
                current_output = Some(output_buf.trim().to_string());
                output_buf.clear();
                in_output = false;

                if let (Some(i), Some(o)) = (current_input.take(), current_output.take()) {
                    cases.push((i, o));
                }
            }

            if path.ends_with("input.scss") || path.ends_with("input.sass") {
                in_input = true;
            } else if path.ends_with("output.css") {
                in_output = true;
            }
        } else if in_input {
            input_buf.push_str(line);
            input_buf.push('\n');
        } else if in_output {
            output_buf.push_str(line);
            output_buf.push('\n');
        }
    }

    if in_input {
        current_input = Some(input_buf.trim().to_string());
    } else if in_output {
        current_output = Some(output_buf.trim().to_string());
    }
    if let (Some(i), Some(o)) = (current_input, current_output) {
        if !i.is_empty() {
            cases.push((i, o));
        }
    }

    cases
}

fn collect_hrx_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(read_dir) = fs::read_dir(dir) else {
        return;
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if !EXCLUDED_DIRS.contains(&name) {
                    collect_hrx_files(&path, files);
                }
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("hrx") {
            files.push(path);
        }
    }
}

fn normalize_css(css: &str) -> String {
    css.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn spec_analysis_per_directory() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_target(false)
        .try_init();
    let _span = tracing::info_span!("sasspile.spec_analysis").entered();

    let root = Path::new(SPEC_ROOT);
    if !root.exists() {
        tracing::warn!("sass-spec not found at {SPEC_ROOT}, skipping");
        return;
    }

    let mut hrx_files = Vec::new();
    collect_hrx_files(root, &mut hrx_files);

    let mut dir_stats: HashMap<String, (usize, usize)> = HashMap::new();
    let mut total_passed = 0;
    let mut total_cases = 0;

    for hrx_path in &hrx_files {
        let Ok(content) = fs::read_to_string(hrx_path) else {
            continue;
        };
        let cases = parse_hrx(&content);
        if cases.is_empty() {
            continue;
        }

        let rel = hrx_path.strip_prefix(root).unwrap_or(hrx_path);
        let top_dir = rel
            .components()
            .next()
            .and_then(|c| c.as_os_str().to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut passed = 0;
        for (input, expected) in &cases {
            let result = std::panic::catch_unwind(|| compile(input));
            if let Ok(Ok(css)) = result {
                if normalize_css(&css) == normalize_css(expected) {
                    passed += 1;
                }
            }
        }

        total_passed += passed;
        total_cases += cases.len();

        let entry = dir_stats.entry(top_dir).or_insert((0, 0));
        entry.0 += passed;
        entry.1 += cases.len();
    }

    // Sort worst-first by pass rate
    let mut stats_vec: Vec<(String, usize, usize)> = dir_stats
        .into_iter()
        .map(|(d, (p, t))| (d, p, t))
        .collect();
    stats_vec.sort_by(|a, b| {
        let rate_a = a.1 as f64 / (a.2 as f64 + 0.001);
        let rate_b = b.1 as f64 / (b.2 as f64 + 0.001);
        rate_a.partial_cmp(&rate_b).unwrap_or(std::cmp::Ordering::Equal)
    });

    tracing::info!("=== sass-spec per-directory pass rates (worst first) ===");
    for (dir, passed, total) in &stats_vec {
        let rate = if *total > 0 {
            (*passed as f64 / *total as f64) * 100.0
        } else {
            0.0
        };
        let bar = "█".repeat((rate / 5.0) as usize);
        tracing::info!("  {dir:20} {:4}/{:4}  {:5.1}%  {}", passed, total, rate, bar);
    }

    let overall_rate = (total_passed as f64 / total_cases.max(1) as f64) * 100.0;
    tracing::info!(
        "=== OVERALL: {}/{} ({:.1}%) ===",
        total_passed, total_cases, overall_rate
    );
}
