//! Spec tests for `sass-spec/spec/directives/` domain.
#[macro_use]
#[path = "spec_macro.rs"]
mod spec_macro;

spec_domain_test!(test_directives_otel, "directives", "directives");
