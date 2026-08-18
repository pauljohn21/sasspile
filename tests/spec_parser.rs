//! Spec tests for `sass-spec/spec/parser/` domain.
#[macro_use]
#[path = "spec_macro.rs"]
mod spec_macro;

spec_domain_test!(test_parser_otel, "parser", "parser");
