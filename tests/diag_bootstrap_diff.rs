//! Bootstrap dist 差异分析诊断
//!
//! 分类统计缺失行，定位下一个修复瓶颈。
//! 运行: `RUST_LOG=diag_bootstrap_diff=debug cargo test --test diag_bootstrap_diff analyze_bootstrap_diff -- --ignored --nocapture`

use rx_scss::builder::CompileBuilder;
use std::collections::HashSet;

#[test]
#[ignore = "diagnostic only — run manually"]
fn analyze_bootstrap_diff() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("diag_bootstrap_diff=debug")
        .with_writer(std::io::stderr)
        .try_init();

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let bootstrap_dir = std::path::PathBuf::from(manifest_dir).join("bootstrap/scss");
    let reference_path = format!("{}/bootstrap/dist/css/bootstrap.css", manifest_dir);

    let css = CompileBuilder::new()
        .expanded()
        .include_path(&bootstrap_dir)
        .compile_file(bootstrap_dir.join("bootstrap.scss"))
        .expect("compile failed");

    let reference = std::fs::read_to_string(&reference_path).expect("read reference");

    let actual_lines: HashSet<&str> = css.lines().collect();
    let reference_lines: HashSet<&str> = reference.lines().collect();

    let missing: Vec<&str> = reference_lines.difference(&actual_lines).copied().collect();
    let extra: Vec<&str> = actual_lines.difference(&reference_lines).copied().collect();

    tracing::info!(actual = actual_lines.len(), reference = reference_lines.len(), missing = missing.len(), extra = extra.len(), "bootstrap diff summary");

    // 分类统计
    let (at_rules, selectors, declarations, braces) = missing.iter().fold(
        (0usize, 0usize, 0usize, 0usize),
        |(at, sel, decl, br), line| {
            let t = line.trim();
            if t.starts_with('@') { (at + 1, sel, decl, br) }
            else if t == "{" || t == "}" || t.starts_with("}.") || t.starts_with("},") { (at, sel, decl, br + 1) }
            else if t.contains(':') && !t.starts_with("/*") { (at, sel, decl + 1, br) }
            else if !t.is_empty() { (at, sel + 1, decl, br) }
            else { (at, sel, decl, br) }
        },
    );
    tracing::info!(at_rules, selectors, declarations, braces, braces_or_empty = braces, "missing by category");

    // 归一化比较：trim + 多空格合一 + 去掉末尾分号
    let normalize = |s: &str| -> String {
        s.trim()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim_end_matches(';')
            .to_string()
    };
    let ref_normalized: HashSet<String> = reference.lines().map(normalize).filter(|s| !s.is_empty()).collect();
    let act_normalized: HashSet<String> = css.lines().map(normalize).filter(|s| !s.is_empty()).collect();
    let missing_norm: Vec<_> = ref_normalized.difference(&act_normalized).collect();
    let extra_norm: Vec<_> = act_normalized.difference(&ref_normalized).collect();
    let coverage_norm = if !ref_normalized.is_empty() {
        (ref_normalized.len() - missing_norm.len()) as f64 / ref_normalized.len() as f64
    } else { 0.0 };
    tracing::info!(ref_norm = ref_normalized.len(), act_norm = act_normalized.len(), missing_norm = missing_norm.len(), extra_norm = extra_norm.len(), coverage_norm = format!("{:.2}%", coverage_norm * 100.0), "normalized comparison");

    // 输出形状诊断
    let total_lines = css.lines().count();
    let null_lines = css.lines().filter(|l| l.contains("null")).count();
    tracing::info!(total_lines, null_lines, "output shape");

    // 统计 null 行 & non-null 但格式错误的行
    let mut null_custom_prop = 0;  // --bs-xxx: null
    let mut null_value = 0;       // prop: null
    let mut spaced_var = 0;        // var(-- bs-...) 含空格
    for line in css.lines() {
        let t = line.trim();
        if t.contains("null") && t.starts_with("--bs") { null_custom_prop += 1; }
        else if t.contains("null") { null_value += 1; }
        if t.contains("-- ") || t.contains("var(-- ") { spaced_var += 1; }
    }
    tracing::info!(null_custom_prop, null_value, spaced_var, "null analysis");

    // 展示 spaced_var 样本
    let mut spaced_count = 0;
    for line in css.lines() {
        let t = line.trim();
        if t.contains("-- ") || t.contains("var(-- ") {
            if spaced_count < 15 {
                tracing::debug!(line = %t, idx = spaced_count, "spaced var sample");
            }
            spaced_count += 1;
        }
    }

    // 展示歸一化後仍然缺失的样本
    let mut null_count = 0;
    let mut missing_decl_count = 0;
    let mut missing_sel_count = 0;
    for line in &extra_norm {
        if line.contains("null") && null_count < 15 {
            tracing::debug!(category = "null_leak", line = %line, "extra normalized");
            null_count += 1;
        }
    }
    for line in missing_norm.iter().take(40) {
        let t = line.as_str();
        if t.starts_with('.') && !t.contains(':') && !t.contains('&') && missing_sel_count < 20 {
            tracing::debug!(category = "missing_sel", line = %t, "missing normalized");
            missing_sel_count += 1;
        } else if (t.contains(':') || t.contains(':')) && missing_decl_count < 20 {
            tracing::debug!(category = "missing_decl", line = %t, "missing normalized");
            missing_decl_count += 1;
        }
    }

    // 展示 EXTRA 样本
    for (i, line) in extra.iter().take(30).enumerate() {
        tracing::debug!(idx = i, line = %line.trim(), "extra line");
    }

    // 展示 MISSING 样本（含 @media 和选择器）
    let mut at_count = 0;
    let mut sel_count = 0;
    for line in &missing {
        let t = line.trim();
        if t.starts_with("@media") || t.starts_with("@supports") || t.starts_with("@keyframes") || t.starts_with("@font-face") {
            if at_count < 20 {
                tracing::debug!(category = "at_rule", line = %t, "missing sample");
                at_count += 1;
            }
        }
    }
    for line in &missing {
        let t = line.trim();
        if !t.starts_with('@') && !t.is_empty() && !t.starts_with('{') && !t.starts_with('}') && !t.contains(':') {
            if sel_count < 20 {
                tracing::debug!(category = "selector", line = %t, "missing sample");
                sel_count += 1;
            }
        }
    }
}
