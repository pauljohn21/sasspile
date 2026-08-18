//! Spec tests for `sass-spec/spec/css/plain/` domain.
#[macro_use]
#[path = "spec_macro.rs"]
mod spec_macro;

spec_domain_test!(test_plain_otel, "css/plain", "css_plain");
