//! Spec tests for `sass-spec/spec/css/` domain (excluding `plain/`).
#[macro_use]
#[path = "spec_macro.rs"]
mod spec_macro;

spec_domain_test!(test_css_otel, "css", "css");
