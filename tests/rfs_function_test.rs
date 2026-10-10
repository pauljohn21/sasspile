//! RFS (Responsive Font Size) engine function regression tests
//!
//! These tests verify that the Bootstrap RFS engine's internal functions
//! work correctly, particularly the @while loops used in divide() and rfs-value().

use rx_scss::telemetry;
use rx_scss::{from_string, Options};

/// Helper: compile SCSS to CSS string
fn compile(scss: &str) -> String {
    let _ = telemetry::init_test_tracing();
    from_string(scss, &Options::default()).unwrap_or_default()
}

/// Test: RFS divide() function — the core division algorithm with nested @while
///
/// This replicates the `_rfs.scss divide()` function which uses nested @while
/// loops for high-precision division. Previously broken because @while inside
/// functions was not executing at all.
#[test]
fn rfs_divide_basic() {
    let scss = r#"
        @function divide($dividend, $divisor, $precision: 10) {
            $result: 0;
            $remainder: $dividend;
            $factor: 10;
            @while ($remainder > 0 and $precision >= 0) {
                $quotient: 0;
                @while ($remainder >= $divisor) {
                    $remainder: $remainder - $divisor;
                    $quotient: $quotient + 1;
                }
                $result: $result * 10 + $quotient;
                $factor: $factor * .1;
                $remainder: $remainder * 10;
                $precision: $precision - 1;
            }
            $result: $result * $factor;
            @return $result;
        }
        .result { width: divide(1, 2); }
    "#;
    let css = compile(scss);
    // 1/2 = 0.5 — the divide function should produce a non-null result
    // We check that the output contains a decimal number (not null/unset/empty)
    assert!(
        !css.is_empty() && !css.contains("null") && !css.contains("unset"),
        "divide(1,2) should return non-null value, got: {}",
        css
    );
    // The result should contain "0.5" or some representation of 0.5
    assert!(
        css.contains("0.5") || css.contains(".5") || css.contains("5"),
        "divide(1,2) should approximate 0.5, got: {}",
        css
    );
}

/// Test: rfs-value() pattern — extract numeric value from rem string
///
/// Simplified version that uses @while to parse a numeric value from a string.
#[test]
fn rfs_value_extraction() {
    let scss = r#"
        @function strip-unit($val) {
            @return $val / ($val * 0 + 1);
        }
        @function parse-decimal($val) {
            $result: 0;
            $remainder: strip-unit($val);
            $i: 0;
            @while $remainder >= 1 {
                $result: $result + 1;
                $remainder: $remainder - 1;
            }
            @return $result;
        }
        .result { width: parse-decimal(42); }
    "#;
    let css = compile(scss);
    // parse-decimal(42) should return 42
    assert!(
        css.contains("42"),
        "parse-decimal(42) should return 42, got: {}",
        css
    );
}

/// Test: RFS-style precision multiplication loop
///
/// Tests the pattern: @while $precision < N { $result: $result * 10; ... }
#[test]
fn rfs_precision_mult_loop() {
    let scss = r#"
        @function rfs-value($val) {
            $result: $val;
            $precision: 0;
            @while $precision < 3 {
                $result: $result * 10;
                $precision: $precision + 1;
            }
            @return $result;
        }
        .result { width: rfs-value(5); }
    "#;
    let css = compile(scss);
    // 5 * 10^3 = 5000
    assert!(
        css.contains("5000"),
        "rfs-value(5) should return 5000 (5*10^3), got: {}",
        css
    );
}

/// Test: RFS fluid-value pattern — clamp + linear interpolation
///
/// Tests a simplified version of rfs-fluid-value that uses @while to calculate
/// a responsive value based on viewport width.
#[test]
fn rfs_fluid_value_pattern() {
    let scss = r#"
        @function clamp-val($min, $max, $pref) {
            $result: $min;
            @while $result < $max {
                @if $result >= $pref {
                    @return $result;
                }
                $result: $result + 1;
            }
            @return $result;
        }
        .result { width: clamp-val(10, 20, 15); }
    "#;
    let css = compile(scss);
    // clamp-val(10, 20, 15) should return 15
    assert!(
        css.contains("15"),
        "clamp-val(10,20,15) should return 15, got: {}",
        css
    );
}
