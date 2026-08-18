//! Spec tests for `sass-spec/spec/expressions/` domain.
#[macro_use]
#[path = "spec_macro.rs"]
mod spec_macro;

spec_domain_test!(test_expressions_otel, "expressions", "expressions");
