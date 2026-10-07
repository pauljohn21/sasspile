//! Bootstrap 5.3.x 全量验证测试
//!
//! 这些测试使用 `#[ignore]` 标记，因为需要 Bootstrap submodule 存在。
//! 运行: `cargo test --test bootstrap_test -- --ignored`

mod common;

use rx_scss::builder::CompileBuilder;
use rx_scss::serialize::Options;

/// 逐字节比对辅助函数
#[allow(dead_code)]
fn assert_css_eq(actual: &str, expected: &str) {
    if actual != expected {
        let actual_bytes = actual.len();
        let expected_bytes = expected.len();
        if let Ok(path) = std::env::var("BOOTSTRAP_DUMP_PATH") {
            std::fs::write(&path, actual).ok();
        }
        panic!(
            "CSS mismatch: actual={} bytes, expected={} bytes, diff={} bytes",
            actual_bytes,
            expected_bytes,
            (actual_bytes as i64 - expected_bytes as i64).abs()
        );
    }
}

#[test]
#[ignore = "requires Bootstrap submodule"]
fn compile_bootstrap_full() {
    let source = std::fs::read_to_string("bootstrap/scss/bootstrap.scss")
        .expect("Bootstrap SCSS not found. Run: git submodule update --init --depth 1 bootstrap");

    // Bootstrap compiles with deep recursion — use larger stack
    let handle = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            CompileBuilder::new()
                .include_path("bootstrap/scss/")
                .compile_string(&source)
        })
        .expect("failed to spawn thread");

    // Use a timeout to diagnose slow compilations
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while std::time::Instant::now() < deadline {
        if handle.is_finished() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    if !handle.is_finished() {
        panic!("Bootstrap compilation timed out after 30s — likely incomplete feature set");
    }
    let result = handle.join().expect("thread panicked");

    match result {
        Ok(css) => {
            tracing::info!(bytes = css.len(), "Bootstrap compiled");
            std::fs::write("/tmp/bootstrap_dump.css", &css).ok();
            assert!(css.len() > 100_000, "Output too small: {} bytes", css.len());
            assert!(css.contains("btn"), "Missing 'btn' selector");
            assert!(css.contains("container"), "Missing 'container' selector");
            assert!(css.contains("modal"), "Missing 'modal' selector");
            assert!(css.contains("navbar"), "Missing 'navbar' selector");
        }
        Err(e) => {
            panic!("Bootstrap compilation failed: {}", e);
        }
    }
}

#[test]
#[ignore = "requires Bootstrap submodule"]
fn compile_bootstrap_compressed() {
    let source = std::fs::read_to_string("bootstrap/scss/bootstrap.scss")
        .expect("Bootstrap SCSS not found. Run: git submodule update --init --depth 1 bootstrap");

    let result = CompileBuilder::new()
        .include_path("bootstrap/scss/")
        .compressed()
        .compile_string(&source);

    match result {
        Ok(css) => {
            assert!(css.len() > 50_000, "Compressed output too small: {} bytes", css.len());
            assert!(!css.contains('\n'), "Compressed CSS should not contain newlines");
        }
        Err(e) => {
            panic!("Bootstrap compressed compilation failed: {}", e);
        }
    }
}

#[test]
#[ignore = "requires Bootstrap submodule"]
fn include_path_resolution() {
    let scss = "@use \"sass:meta\";\n$x: 1;\n";
    let _result = CompileBuilder::new()
        .include_path("bootstrap/scss/")
        .compile_string(scss);
}

/// Bootstrap dist alignment test — compares compiled output against dist CSS.
/// Will be enabled (remove #[ignore]) once coverage >= 99% is achieved (task 6.1).
#[test]
#[ignore = "bootstrap dist alignment coverage < 99% — enabled after all features implemented"]
fn test_bootstrap_dist_alignment() {
    let check = crate::common::bootstrap_dist::bootstrap_dist_check()
        .expect("Bootstrap submodule not found — run: git submodule update --init --depth 1");

    let coverage = if check.reference_line_count > 0 {
        (check.reference_line_count - check.missing_count) as f64 / check.reference_line_count as f64
    } else {
        0.0
    };

    tracing::info!(
        coverage_pct = coverage * 100.0,
        missing = check.missing_count,
        total = check.reference_line_count,
        "Bootstrap dist alignment"
    );

    assert!(
        coverage >= 0.99,
        "Bootstrap dist alignment coverage {:.2}% < 99% (missing {} lines)",
        coverage * 100.0,
        check.missing_count
    );
}

#[test]
#[ignore = "requires Bootstrap submodule"]
fn diff_context_on_mismatch() {
    let actual = "body{color:red;}";
    let expected = "body{color:blue;}";

    let diff = similar::TextDiff::from_lines(actual, expected);
    let changes: Vec<_> = diff.iter_all_changes().collect();
    assert!(!changes.is_empty(), "Diff should show changes");
}
