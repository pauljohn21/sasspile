//! spec-store: sass-spec 数据管理工具（集成测试入口）。
//!
//! 用法：
//!   SPEC_STORE_CMD=index  cargo test --test spec_store -- --nocapture
//!   SPEC_STORE_CMD=run     cargo test --test spec_store -- --nocapture
//!   SPEC_STORE_CMD=stats   cargo test --test spec_store -- --nocapture
//!   SPEC_STORE_CMD=trend FN=math.sin  cargo test --test spec_store -- --nocapture
//!   SPEC_STORE_CMD=link FN=math.sin   cargo test --test spec_store -- --nocapture
//!   SPEC_STORE_CMD=bisect FN=math.sin GOOD=1 BAD=10  cargo test --test spec_store -- --nocapture
//!   SPEC_STORE_CMD=diff FROM=1 TO=10  cargo test --test spec_store -- --nocapture

#[allow(
    clippy::used_underscore_binding,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::trivially_copy_pass_by_ref,
    clippy::needless_raw_string_hashes,
    clippy::redundant_closure,
    clippy::needless_pass_by_value,
    clippy::useless_format,
    clippy::let_underscore_untyped,
    dead_code
)]
mod specstore;

#[test]
fn spec_store_cli() {
    specstore::run();
}
