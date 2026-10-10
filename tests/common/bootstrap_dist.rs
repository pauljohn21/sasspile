//! Shared Bootstrap dist alignment helpers
//!
//! Used by both `integration_test.rs` and `bootstrap_test.rs`.

use rx_scss::builder::CompileBuilder;

/// Result of Bootstrap dist alignment check
pub struct BootstrapDistCheck {
    pub actual_line_count: usize,
    pub reference_line_count: usize,
    pub missing_count: usize,
    pub extra_count: usize,
    pub missing_lines: Vec<String>,
}

/// Compile bootstrap.scss and compare against dist CSS reference.
/// Emits structured tracing spans and categorized breakdown.
pub fn bootstrap_dist_check() -> Option<BootstrapDistCheck> {
    let _span = tracing::info_span!("bootstrap_dist_check").entered();
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let bootstrap_dir = std::path::PathBuf::from(manifest_dir).join("bootstrap/scss");
    let reference_path = format!("{}/bootstrap/dist/css/bootstrap.css", manifest_dir);

    let css = CompileBuilder::new()
        .expanded()
        .include_path(&bootstrap_dir)
        .compile_file(bootstrap_dir.join("bootstrap.scss"))
        .ok()?;

    let reference = std::fs::read_to_string(&reference_path).ok()?;

    let actual_lines: Vec<&str> = css.lines().collect();
    let reference_lines_vec: Vec<&str> = reference.lines().collect();

    // Count non-empty, non-comment lines for meaningful coverage
    let actual_set: std::collections::HashSet<&str> = actual_lines.iter().copied().collect();
    let reference_set: std::collections::HashSet<&str> = reference_lines_vec.iter().copied().collect();

    let missing: Vec<String> = reference_set.difference(&actual_set)
        .map(|l| l.to_string())
        .collect();
    let extra: Vec<String> = actual_set.difference(&reference_set)
        .map(|l| l.to_string())
        .collect();

    // Categorize missing lines for diagnosis
    let mut selectors_missing = 0;
    let mut bs_vars_missing = 0;
    let mut webkit_missing = 0;
    let mut moz_missing = 0;
    let mut o_missing = 0;
    let mut ms_missing = 0;
    let mut decl_missing = 0;

    for line in &missing {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        } else if trimmed == "{" || trimmed.starts_with('.') || trimmed.starts_with('#')
            || trimmed.starts_with('@') || trimmed.starts_with('[')
            || trimmed.contains(":not(") || trimmed.contains("::") {
            selectors_missing += 1;
        } else if trimmed.starts_with("--bs-") {
            bs_vars_missing += 1;
        } else if trimmed.starts_with("-webkit-") {
            webkit_missing += 1;
        } else if trimmed.starts_with("-moz-") {
            moz_missing += 1;
        } else if trimmed.starts_with("-o-") {
            o_missing += 1;
        } else if trimmed.starts_with("-ms-") {
            ms_missing += 1;
        } else if !trimmed.starts_with(':') && !trimmed.starts_with('*') {
            decl_missing += 1;
        }
    }

    let total_ref_nonblank = reference.lines().filter(|l| !l.trim().is_empty()).count();
    let total_actual_nonblank = css.lines().filter(|l| !l.trim().is_empty()).count();

    tracing::info!(
        actual_lines = actual_set.len(),
        reference_lines = reference_set.len(),
        actual_nonblank = total_actual_nonblank,
        reference_nonblank = total_ref_nonblank,
        missing = missing.len(),
        extra = extra.len(),
        coverage_pct = (reference_set.len() - missing.len()) as f64 / reference_set.len().max(1) as f64 * 100.0,
        "bootstrap_dist_summary"
    );

    tracing::info!(
        selectors_missing = selectors_missing,
        bs_vars_missing = bs_vars_missing,
        webkit_missing = webkit_missing,
        moz_missing = moz_missing,
        o_missing = o_missing,
        ms_missing = ms_missing,
        decl_other_missing = decl_missing,
        "bootstrap_dist_missing_breakdown"
    );

    // Emit first 30 missing selectors via debug
    let missing_selectors: Vec<&str> = missing.iter()
        .map(|s| s.trim())
        .filter(|t| !t.is_empty() && (t.starts_with('.') || t.starts_with('#') || t.starts_with('@') || *t == "{"))
        .take(30)
        .collect();
    if !missing_selectors.is_empty() {
        tracing::debug!(?missing_selectors, "missing_selectors_sample");
    }

    // Emit first 20 missing declarations via debug
    let missing_decls: Vec<&str> = missing.iter()
        .map(|s| s.trim())
        .filter(|t| {
            !t.is_empty() && !t.starts_with('.') && !t.starts_with('#') && !t.starts_with('@')
                && *t != "{" && !t.starts_with(':') && !t.starts_with('*')
        })
        .take(20)
        .collect();
    if !missing_decls.is_empty() {
        tracing::debug!(?missing_decls, "missing_declarations_sample");
    }

    Some(BootstrapDistCheck {
        actual_line_count: actual_set.len(),
        reference_line_count: reference_set.len(),
        missing_count: missing.len(),
        extra_count: extra.len(),
        missing_lines: missing,
    })
}

/// Calculate coverage percentage (0.0-1.0) of actual lines vs reference lines.
pub fn bootstrap_dist_coverage() -> Option<f64> {
    let check = bootstrap_dist_check()?;
    if check.reference_line_count == 0 {
        return Some(0.0);
    }
    let covered = check.reference_line_count.saturating_sub(check.missing_count);
    Some(covered as f64 / check.reference_line_count as f64)
}
