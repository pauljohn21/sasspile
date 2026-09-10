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

mod specstore;

#[test]
fn spec_store_cli() {
    specstore::run();
}
