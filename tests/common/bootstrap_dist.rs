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
/// Outputs missing/extra counts and first 50 differences via eprintln.
pub fn bootstrap_dist_check() -> Option<BootstrapDistCheck> {
    use std::collections::HashSet;
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let bootstrap_dir = std::path::PathBuf::from(manifest_dir).join("bootstrap/scss");
    let reference_path = format!("{}/bootstrap/dist/css/bootstrap.css", manifest_dir);

    let css = CompileBuilder::new()
        .expanded()
        .include_path(&bootstrap_dir)
        .compile_file(bootstrap_dir.join("bootstrap.scss"))
        .ok()?;

    let reference = std::fs::read_to_string(&reference_path).ok()?;

    let actual_lines: HashSet<&str> = css.lines().collect();
    let reference_lines: HashSet<&str> = reference.lines().collect();

    let missing: Vec<String> = reference_lines.difference(&actual_lines)
        .map(|l| l.to_string())
        .collect();
    let extra: Vec<String> = actual_lines.difference(&reference_lines)
        .map(|l| l.to_string())
        .collect();

    eprintln!(
        "Bootstrap dist check: actual={} lines, reference={} lines, missing={}, extra={}",
        actual_lines.len(),
        reference_lines.len(),
        missing.len(),
        extra.len()
    );

    let show_count = 50.min(missing.len());
    for (i, line) in missing.iter().take(show_count).enumerate() {
        eprintln!("  missing[{}]: {}", i, line.trim());
    }

    Some(BootstrapDistCheck {
        actual_line_count: actual_lines.len(),
        reference_line_count: reference_lines.len(),
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
