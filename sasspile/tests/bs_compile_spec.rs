//! Bootstrap SCSS 全量编译验证。

use std::path::PathBuf;
use sasspile::{tokenize, parse};
use tracing::{info, info_span};

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

fn validate_file(path: &PathBuf) -> Result<(usize, usize), String> {
    let source = std::fs::read_to_string(path).map_err(|e| format!("read: {e}"))?;
    let (tokens, lex_diags) = tokenize(&source);
    let (lex_e, lex_w, _) = lex_diags.counts();
    if lex_e > 0 {
        let detail: Vec<String> = lex_diags.errors().iter().take(3).map(|d| d.message.clone()).collect();
        return Err(format!("lexer: {lex_e} errors, {lex_w} warns — {}", detail.join("; ")));
    }
    let (stylesheet, parse_diags) = parse(&source);
    let (p_e, p_w, _) = parse_diags.counts();
    if p_e > 0 {
        let detail: Vec<String> = parse_diags.errors().iter().take(3).map(|d| d.message.clone()).collect();
        return Err(format!("parser: {p_e} errors, {p_w} warns — {}", detail.join("; ")));
    }
    Ok((tokens.len(), stylesheet.nodes.len()))
}

#[test]
fn batch_validate_bootstrap_scss() {
    let span = info_span!("batch_validate_bs", theme = "bootstrap");
    let _enter = span.enter();

    let dir = "/Users/pauljohn/rust/sasslipe-next/bs/scss";
    let files = collect_scss(dir);
    info!(total_files = files.len(), "found SCSS files");

    let mut success = 0;
    let mut failed = 0;
    let total_lex_warnings = 0;
    let mut total_tokens = 0;
    let mut total_nodes = 0;
    let mut failures: Vec<(String, String)> = Vec::new();

    for path in &files {
        let rel = path.strip_prefix(dir).unwrap_or(path);
        let file_span = info_span!("process_file", file = %rel.display());
        let _file_enter = file_span.enter();

        match validate_file(path) {
            Ok((tok_count, node_count)) => {
                success += 1;
                total_tokens += tok_count;
                total_nodes += node_count;
                info!(tok_count, node_count, "passed");
            }
            Err(e) => {
                failed += 1;
                failures.push((rel.display().to_string(), e.clone()));
                info!(error = %e, "failed");
            }
        }
    }

    let pass_rate = 100.0 * success as f64 / files.len().max(1) as f64;
    info!(
        total = files.len(),
        success,
        failed,
        total_lex_warnings,
        total_tokens,
        total_nodes,
        pass_rate = format!("{:.1}%", pass_rate),
        "BS batch complete"
    );

    if !failures.is_empty() {
        let detail: Vec<String> = failures.iter().take(30).map(|(p, e)| format!("{p}: {e}")).collect();
        panic!("BS batch {success}/{} ({:.1}%) | FAIL:\n{}", files.len(), pass_rate, detail.join("\n"));
    } else {
        info!("BS batch {success}/{} ({:.1}%) — ALL PASS", files.len(), pass_rate);
    }
}
