## ADDED Requirements

### Requirement: Clippy deny 级 lint 配置
`Cargo.toml` 或 `clippy.toml` SHALL 配置以下 deny 级 lint：
- `clippy::unwrap_used = "deny"` — 禁止生产代码中的 unwrap
- `clippy::todo = "deny"` — 禁止 todo!()
- `clippy::unimplemented = "deny"` — 禁止 unimplemented!()
- `clippy::expect_used = "warn"` — 允许 expect() 但给出警告

#### Scenario: CI 中 clippy 拒绝新 unwrap
- **WHEN** 新增代码包含 `.unwrap()`
- **THEN** `cargo clippy -- -D warnings` SHALL 失败并指出 `clippy::unwrap_used`

#### Scenario: CI 中 clippy 拒绝 todo!()
- **WHEN** 新增代码包含 `todo!()` 或 `unimplemented!()`
- **THEN** `cargo clippy` SHALL 失败

### Requirements: 文件行数检测测试
新增 `tests/file_size_check.rs` SHALL 包含 `check_file_size_limits` 测试函数。

#### Scenario: 测试在 CI 中运行
- **WHEN** 执行 `cargo test --test file_size_check`
- **THEN** SHALL 输出通过/失败结果

### Requirement: 测试通过标准
所有 202 个核心测试 + 新增 `file_size_check` SHALL 在本次变更后 100% 通过。

#### Scenario: 完整核心测试套件
- **WHEN** 运行 `cargo test --test compile_test && cargo test --test stage_test && cargo test --test ast_test && cargo test --test common_test`
- **THEN** SHALL 全部通过（无回归）

#### Scenario: bootstrap + element-plus 兼容
- **WHEN** 运行 `cargo test --test bs_spec && cargo test --test ep_full`
- **THEN** SHALL 100% 通过（15/15 + 121/121）
