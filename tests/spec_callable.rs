//! Spec tests for `sass-spec/spec/callable/` domain.
#[macro_use]
#[path = "spec_macro.rs"]
mod spec_macro;

spec_domain_test!(test_callable_otel, "callable", "callable");
