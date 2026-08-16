//! Element Plus theme-chalk SCSS 批量编译验证。
//!
//! 遍历 ep/packages/theme-chalk/src 下所有 .scss 文件，
//! 用 sasspile 的 lexer + parser 验证解析能力。

use std::path::PathBuf;
use sasspile::{tokenize, parse};
use tracing::{info, info_span};

/// 递归收集目录下所有 .scss 文件
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

/// 验证单个 SCSS 文件能否成功 tokenize + parse
fn validate_file(path: &PathBuf) -> Result<(usize, usize, usize), String> {
    let span = info_span!("validate_file", file = %path.display());
    let _enter = span.enter();

    let source = std::fs::read_to_string(path).map_err(|e| format!("read: {e}"))?;

    // 1. Tokenize
    let (tokens, lex_diags) = tokenize(&source);
    let (lex_e, lex_w, _lex_i) = lex_diags.counts();
    if lex_e > 0 {
        let detail: Vec<String> = lex_diags.errors().iter().take(5).map(|d| d.message.clone()).collect();
        return Err(format!("lexer: {lex_e} errors, {lex_w} warns — {}", detail.join("; ")));
    }

    // 2. Parse
    let (stylesheet, parse_diags) = parse(&source);
    let (p_e, p_w, _p_i) = parse_diags.counts();
    if p_e > 0 {
        let detail: Vec<String> = parse_diags.errors().iter().take(5).map(|d| d.message.clone()).collect();
        return Err(format!("parser: {p_e} errors, {p_w} warns — {}", detail.join("; ")));
    }

    info!(tokens = tokens.len(), nodes = stylesheet.nodes.len(), "file OK");
    Ok((lex_w + p_w, tokens.len(), stylesheet.nodes.len()))
}

#[test]
fn batch_validate_element_plus_theme_chalk() {
    let span = info_span!("batch_validate", theme = "element-plus");
    let _enter = span.enter();

    let dir = "/Users/pauljohn/rust/sasslipe-next/ep/packages/theme-chalk/src";
    let files = collect_scss(dir);
    info!(total_files = files.len(), "found SCSS files");

    let mut success = 0;
    let mut failed = 0;
    let mut total_warnings = 0;
    let mut total_tokens = 0;
    let mut total_nodes = 0;
    let mut failures: Vec<(String, String)> = Vec::new();

    for path in &files {
        let rel = path.strip_prefix(dir).unwrap_or(path);
        let file_span = info_span!("process_file", file = %rel.display());
        let _file_enter = file_span.enter();

        match validate_file(path) {
            Ok((warnings, tok_count, node_count)) => {
                success += 1;
                total_warnings += warnings;
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
        total_warnings,
        total_tokens,
        total_nodes,
        pass_rate = format!("{:.1}%", pass_rate),
        "batch complete"
    );

    if !failures.is_empty() {
        let detail: Vec<String> = failures
            .iter()
            .take(40)
            .map(|(p, e)| format!("{p}: {e}"))
            .collect();
        panic!(
            "EP batch {success}/{} ({:.1}%) | FAIL:\n{}",
            files.len(),
            pass_rate,
            detail.join("\n")
        );
    } else {
        info!("EP batch {success}/{} ({:.1}%) — ALL PASS", files.len(), pass_rate);
    }
}

#[test]
fn validate_button_scss_deep() {
    let span = info_span!("deep_validate", file = "button.scss");
    let _enter = span.enter();

    let path = PathBuf::from("/Users/pauljohn/rust/sasslipe-next/ep/packages/theme-chalk/src/button.scss");
    let source = std::fs::read_to_string(&path).expect("read button.scss");

    let (tokens, lex_diags) = tokenize(&source);
    let (lex_e, lex_w, _) = lex_diags.counts();
    if lex_e > 0 {
        for d in lex_diags.errors().iter().take(5) {
            info!(error = %d.message, "lexer error");
        }
    }

    let (stylesheet, parse_diags) = parse(&source);
    let (p_e, p_w, _) = parse_diags.counts();
    if p_e > 0 {
        for d in parse_diags.errors().iter().take(20) {
            info!(error = %d.message, "parser error");
        }
    }

    info!(
        tokens = tokens.len(),
        nodes = stylesheet.nodes.len(),
        lex_errors = lex_e,
        parse_errors = p_e,
        lex_warnings = lex_w,
        parse_warnings = p_w,
        "button.scss analysis complete"
    );
}

#[test]
fn validate_mixins_dir() {
    let span = info_span!("validate_mixins");
    let _enter = span.enter();

    let dir = "/Users/pauljohn/rust/sasslipe-next/ep/packages/theme-chalk/src/mixins";
    let files = collect_scss(dir);
    info!(total_files = files.len(), "found SCSS files in mixins/");

    let mut passed = 0;
    for path in &files {
        let rel = path.strip_prefix(dir).unwrap_or(path);
        match validate_file(path) {
            Ok(_) => {
                passed += 1;
                info!(file = %rel.display(), "passed");
            }
            Err(e) => {
                info!(file = %rel.display(), error = %e, "failed");
            }
        }
    }

    let pass_rate = 100.0 * passed as f64 / files.len().max(1) as f64;
    info!(
        passed = passed,
        total = files.len(),
        pass_rate = format!("{:.1}%", pass_rate),
        "mixins complete"
    );
}
