//! Spec tests for `sass-spec/spec/variables/` domain.
#[macro_use]
#[path = "spec_macro.rs"]
mod spec_macro;

spec_domain_test!(test_variables_otel, "variables", "variables");
