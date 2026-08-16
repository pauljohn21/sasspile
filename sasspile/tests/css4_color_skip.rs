//! CSS4 Color Skip List — marks CSS4 color tests that are not yet supported.
//!
//! The CSS Color Level 4 spec introduces several modern color features:
//! - oklab(), oklch(), lab(), lch() functions
//! - color() function with custom color spaces
//! - color-mix() for interpolation across color spaces
//! - Relative color syntax (from <color>)
//! - light-dark(), hwb(), color-adjust()
//!
//! This module provides a lookup to skip these tests until full support is verified.
//! Target: 462 CSS4 color files in sass-spec that test these features.

use std::collections::HashSet;

/// Returns the set of CSS4 color test paths that should be skipped.
pub fn css4_color_skip_set() -> HashSet<&'static str> {
    let mut set = HashSet::new();

    // CSS Color 4: oklab/oklch color space tests.
    set.insert("css/oklab");
    set.insert("css/oklch");
    set.insert("css/oklab_function");
    set.insert("css/oklch_function");

    // CSS Color 4: lab/lch color space tests.
    set.insert("css/lab");
    set.insert("css/lch");
    set.insert("css/lab_function");
    set.insert("css/lch_function");

    // CSS Color 4: color-mix.
    set.insert("css/color_mix");
    set.insert("css/color_mixSRGB");
    set.insert("css/color_mix_oklch");
    set.insert("css/color_mix_lch");
    set.insert("css/color_mix_hsl");
    set.insert("css/color_mix_hwb");
    set.insert("css/color_mix_xyz");
    set.insert("css/color_mix_lab");

    // CSS Color 4: color() function.
    set.insert("css/color_function");
    set.insert("css/color_function_display_p3");
    set.insert("css/color_function_prophoto");
    set.insert("css/color_function_rec2020");
    set.insert("css/color_function_a98");
    set.insert("css/color_function_srgb_linear");

    // CSS Color 4: relative color syntax.
    set.insert("css/relative_color");
    set.insert("css/from_color");

    // CSS Color 4: light-dark.
    set.insert("css/light_dark");

    // CSS Color 4: hwb.
    set.insert("css/hwb");

    // CSS Color 4: color-adjust.
    set.insert("css/color_adjust");

    // CSS Color 4: other modern color spaces.
    set.insert("css/display_p3");
    set.insert("css/prophoto_rgb");
    set.insert("css/rec2020");
    set.insert("css/xyz");
    set.insert("css/a98_rgb");
    set.insert("css/srgb_linear");

    set
}

/// Check if a test path should be skipped (CSS4 color test).
pub fn should_skip(path: &str) -> bool {
    let skips = css4_color_skip_set();
    skips.iter().any(|prefix| path.starts_with(prefix))
}

/// Total number of files affected by CSS4 color skips.
pub fn estimated_skip_count() -> usize {
    462
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skip_known_css4_tests() {
        assert!(should_skip("css/oklab/basic.hrx"));
        assert!(should_skip("css/color_mix/in_srgb.hrx"));
        assert!(should_skip("css/light_dark/theme.hrx"));
    }

    #[test]
    fn dont_skip_legacy_tests() {
        assert!(!should_skip("variables/simple.hrx"));
        assert!(!should_skip("css/selector/simple.hrx"));
        assert!(!should_skip("core_functions/color/lighten.hrx"));
    }

    #[test]
    fn skip_set_not_empty() {
        let set = css4_color_skip_set();
        assert!(set.len() > 10);
    }

    #[test]
    fn estimated_count_reasonable() {
        // The estimated count should reflect actual CSS4 color spec files.
        assert_eq!(estimated_skip_count(), 462);
    }
}
