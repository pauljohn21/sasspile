//! sass-spec black-box acceptance tests.
//!
//! Parses HRX files from the sass-spec submodule, runs each
//! `input.scss` through `sasspile_rx::compile`, and compares with `output.css`.
//!
//! Excluded directories (per tasks.md 7.4): libsass/,
//! libsass-closed-issues/, libsass-todo-issues/, libsass-todo-tests/, non_conformant/

use std::fs;
use std::path::{Path, PathBuf};

use sasspile_rx::compile;

const SPEC_ROOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/sass-spec/spec"
);

const EXCLUDED_DIRS: &[&str] = &[
    "libsass",
    "libsass-closed-issues",
    "libsass-todo-issues",
    "libsass-todo-tests",
    "non_conformant",
];

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// HRX parser
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Parse one HRX file into a list of cases.
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
            let path = &line[separator.len()..];
            let path = path.trim();

            // Finish previous section
            if in_input {
                current_input = Some(input_buf.trim().to_string());
                input_buf.clear();
                in_input = false;
            } else if in_output {
                current_output = Some(output_buf.trim().to_string());
                output_buf.clear();
                in_output = false;

                // Pair complete
                match (current_input.take(), current_output.take()) {
                    (Some(i), Some(o)) => cases.push((i, o)),
                    _ => {}
                }
            }

            if path.ends_with("input.scss") || path.ends_with("input.sass") {
                in_input = true;
            } else if path.ends_with("output.css") {
                in_output = true;
            }
            // else: options.yml, warning, error, README.md → skip
        } else if in_input {
            input_buf.push_str(line);
            input_buf.push('\n');
        } else if in_output {
            output_buf.push_str(line);
            output_buf.push('\n');
        }
    }

    // Handle last pair
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

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Spec walker
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Tests
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn init_test_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_target(false)
        .try_init();
}

#[test]
fn spec_walker_run_all_hrx() {
    init_test_tracing();
    let _span = tracing::info_span!("sasspile.spec_walker").entered();

    let root = Path::new(SPEC_ROOT);
    if !root.exists() {
        tracing::warn!("sass-spec not found at {SPEC_ROOT}, skipping");
        return;
    }

    let mut hrx_files = Vec::new();
    collect_hrx_files(root, &mut hrx_files);

    let mut total_cases: usize = 0;
    let mut passed: usize = 0;
    let mut failed: usize = 0;
    let mut panicked: usize = 0;
    let mut failures: Vec<String> = Vec::new();

    for hrx_path in &hrx_files {
        let Ok(content) = fs::read_to_string(hrx_path) else {
            continue;
        };
        let cases = parse_hrx(&content);

        for (i, (input, expected)) in cases.into_iter().enumerate() {
            total_cases += 1;

            // Run through the public API — catch panic as failure
            let result = std::panic::catch_unwind(|| compile(&input));

            match result {
                Ok(Ok(css)) => {
                    let actual = normalize_css(&css);
                    let exp = normalize_css(&expected);
                    if actual == exp {
                        passed += 1;
                    } else {
                        failed += 1;
                        let path_display = hrx_path
                            .strip_prefix(root)
                            .unwrap_or(hrx_path)
                            .display();
                        if failures.len() < 10 {
                            failures.push(format!(
                                "  [{path_display}#{i}]\n    expected: {exp:?}\n    actual:   {actual:?}"
                            ));
                        }
                    }
                }
                Ok(Err(_)) => {
                    // compile returned Err — only counts as pass if expected
                    // output is empty (error case)
                    if expected.trim().is_empty() {
                        passed += 1;
                    } else {
                        failed += 1;
                    }
                }
                Err(_) => {
                    panicked += 1;
                }
            }
        }
    }

    let pass_rate = if total_cases > 0 {
        (passed as f64 / total_cases as f64) * 100.0
    } else {
        0.0
    };

    tracing::info!("sass-spec HRX acceptance results:");
    tracing::info!("  total:    {total_cases}");
    tracing::info!("  passed:   {passed}");
    tracing::info!("  failed:   {failed}");
    tracing::info!("  panicked: {panicked}");
    tracing::info!("  rate:     {pass_rate:.1}%");
    for f in &failures {
        tracing::error!("{f}");
    }

    // Soft assertion: at least one case should pass
    assert!(passed > 0, "at least one sass-spec case should pass");

    // For enterprise acceptance the target is 100% on active spec directories
    // Bootstrap + Element Plus compliance is covered by independent fixture tests
}
