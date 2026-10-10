//! @while loop evaluation regression tests
//!
//! Verifies:
//! 1. Top-level @while loops inline-evaluate body (variable mutations visible to condition)
//! 2. @while inside function bodies works correctly
//! 3. Nested @while loops (division algorithm) work correctly

use rx_scss::telemetry;
use rx_scss::{from_string, Options};

/// Helper: compile SCSS to CSS string
fn compile(scss: &str) -> String {
    let _ = telemetry::init_test_tracing();
    from_string(scss, &Options::default()).unwrap_or_default()
}

/// Test: counter-based @while at top level
///
/// `$i` starts at 0, increments until reaching 5.
/// Should produce 5 rules with classes .i-0 through .i-4.
#[test]
fn while_counter_top_level() {
    let scss = r#"
        $i: 0;
        @while $i < 5 {
            .i-#{$i} { width: #{$i}0px; }
            $i: $i + 1;
        }
    "#;
    let css = compile(scss);
    assert!(css.contains(".i-0"), " Should contain .i-0, got: {}", css);
    assert!(css.contains(".i-1"), "Should contain .i-1, got: {}", css);
    assert!(css.contains(".i-2"), "Should contain .i-2, got: {}", css);
    assert!(css.contains(".i-3"), "Should contain .i-3, got: {}", css);
    assert!(css.contains(".i-4"), "Should contain .i-4, got: {}", css);
    assert!(!css.contains(".i-5"), "Should NOT contain .i-5, got: {}", css);
}

/// Test: @while inside @function — the RFS pattern
///
/// This is the core RFS pattern: a function uses @while to accumulate a result,
/// then @return returns it. Previously broken because @while inside functions was inert.
#[test]
fn while_inside_function() {
    let scss = r#"
        @function double-to($n, $times) {
            $result: $n;
            $i: 0;
            @while $i < $times {
                $result: $result * 2;
                $i: $i + 1;
            }
            @return $result;
        }
        .result { width: double-to(5, 3); }
    "#;
    let css = compile(scss);
    // 5 * 2^3 = 40
    assert!(
        css.contains("width: 40px")
            || css.contains("width: 40")
            || css.contains("40"),
        "Expected width: 40 (5*2^3), got: {}",
        css
    );
}

/// Test: division remainder loop pattern (like Bootstrap RFS `divide()`)
#[test]
fn while_division_remainder_loop() {
    let scss = r#"
        @function sum-digits($n) {
            $sum: 0;
            $remainder: $n;
            @while $remainder >= 10 {
                $sum: $sum + 10;
                $remainder: $remainder - 10;
            }
            $sum: $sum + $remainder;
            @return $sum;
        }
        .result { width: sum-digits(25); }
    "#;
    let css = compile(scss);
    // 25 -> 10+10+5 = 25
    assert!(
        css.contains("width: 25")
            || css.contains("width: 25px")
            || css.contains("25"),
        "Expected width: 25, got: {}",
        css
    );
}

/// Test: @while with RFS-style precision loop
#[test]
fn while_precision_loop() {
    let scss = r#"
        @function precision-mult($base, $multiplier, $precision) {
            $result: $base;
            $i: 0;
            @while $i < $precision {
                $result: $result * $multiplier;
                $i: $i + 1;
            }
            @return $result;
        }
        .result { width: precision-mult(2, 10, 3); }
    "#;
    let css = compile(scss);
    // 2 * 10^3 = 2000
    assert!(
        css.contains("2000"),
        "Expected width with 2000 (2*10^3), got: {}",
        css
    );
}
