# sasspile 开发任务

# 默认任务
default:
    @just --list

# 全部测试
test-all:
    cargo test --test compile_test
    cargo test --test stage_test
    cargo test --test ast_test
    cargo test --test common_test
    cargo test --test bs_spec -- --nocapture

# sass-spec 完整验证
test-sass-spec:
    cargo test --test sass_spec_full -- --nocapture

# clippy 检查
clippy:
    cargo clippy --all-targets

# 基准测试
bench:
    cargo bench

# 编译检查
check:
    cargo check --all-targets

# 格式化
fmt:
    cargo fmt

# 构建发布版本
build-release:
    cargo build --release

# 运行 sass-spec 诊断
diag subdir:
    cargo test --test cf_diag diag_{{subdir}} -- --nocapture

# 追踪编译
trace test_name:
    RUST_LOG=info cargo test --test compile_test {{test_name}} -- --nocapture

# 生成文档
doc:
    cargo doc --no-deps --open

# 发布准备（完整验证）:
release-check:
    cargo clippy --all-targets
    cargo test --test compile_test
    cargo test --test stage_test
    cargo test --test ast_test
    cargo test --test common_test
    cargo test --test bs_spec -- --nocapture
    cargo fmt --check
