//! Bootstrap SCSS 全量解析统计。

use std::path::PathBuf;
use sasspile::{tokenize, parse};

fn collect_scss(dir: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_scss(path.to_str().unwrap_or("")));
            } else if path.extension().and_then(|e| e.to_str()) == Some("scss") {
                files.push(path);
            }
        }
    }
    files
}

fn validate_file(path: &PathBuf) -> Result<(), String> {
    let source = std::fs::read_to_string(path).map_err(|e| format!("read: {e}"))?;
    let (_tokens, lex_diags) = tokenize(&source);
    let lex_e = lex_diags.errors().len();
    if lex_e > 0 {
        let detail: Vec<String> = lex_diags.errors().iter().take(3).map(|d| d.message.clone()).collect();
        return Err(format!("lexer: {lex_e} errors — {}", detail.join("; ")));
    }
    let (_stylesheet, parse_diags) = parse(&source);
    let p_e = parse_diags.errors().len();
    if p_e > 0 {
        let detail: Vec<String> = parse_diags.errors().iter().take(3).map(|d| d.message.clone()).collect();
        return Err(format!("parser: {p_e} errors — {}", detail.join("; ")));
    }
    Ok(())
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .with_level(true)
        .try_init();
}

#[test]
fn count_bs_pass_rate() {
    init_tracing();
    let dir = "/Users/pauljohn/rust/sasslipe-next/bs/scss";
    let files = collect_scss(dir);
    let mut success = 0;
    let mut failed = 0;
    let mut failures: Vec<(String, String)> = Vec::new();
    for path in &files {
        let rel = path.strip_prefix(dir).unwrap_or(path);
        match validate_file(path) {
            Ok(()) => success += 1,
            Err(e) => {
                failed += 1;
                failures.push((rel.display().to_string(), e));
            }
        }
    }
    let mut error_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (_, err) in &failures {
        let key = err.split(" — ").next().unwrap_or(err).to_string();
        *error_counts.entry(key).or_insert(0) += 1;
    }
    let mut sorted_errors: Vec<_> = error_counts.iter().collect();
    sorted_errors.sort_by(|a, b| b.1.cmp(a.1));

    let pass_rate = 100.0 * success as f64 / files.len().max(1) as f64;
    tracing::info!(
        total = files.len(),
        success,
        failed,
        pass_rate = format!("{:.1}%", pass_rate),
        "BS batch complete"
    );
    tracing::info!("--- BS Error Patterns ---");
    for (k, v) in sorted_errors.iter().take(15) {
        tracing::info!("  [{:3}x] {}", v, k);
    }
    tracing::info!("--- BS Failure Samples (first 15) ---");
    for (p, e) in failures.iter().take(15) {
        tracing::info!("  {} -> {}", p, e);
    }

    assert!(pass_rate >= 60.0, "BS pass rate too low: {:.1}%", pass_rate);
}
