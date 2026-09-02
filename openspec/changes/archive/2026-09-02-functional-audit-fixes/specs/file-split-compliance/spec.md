## ADDED Requirements

### Requirement: All source files MUST not exceed 500 lines

每个 `src/` 下的 Rust 源文件（`.rs`）MUST 不超过 500 行（源码和测试分别计算）。超出时 MUST 拆分为多个模块文件。

#### Scenario: eval/builtin.rs exceeds 500 lines
- **WHEN** `src/eval/builtin.rs` 当前为 524 行
- **THEN** MUST 拆分为 `builtin.rs`（入口 + 模块声明）和 `builtin/manual_dispatch.rs`（手工分派函数），每个文件 ≤ 500 行

#### Scenario: parse/expr/prefix.rs exceeds 500 lines
- **WHEN** `src/parse/expr/prefix.rs` 当前为 503 行
- **THEN** MUST 拆分为 `prefix.rs`（控制流 + 运算符解析）和 `expr/literals.rs`（字面量解析），每个文件 ≤ 500 行

#### Scenario: No file exceeds 500 lines after refactor
- **WHEN** 拆分完成后运行 `wc -l src/**/*.rs`
- **THEN** 所有文件 MUST ≤ 500 行

#### Scenario: Module declarations updated
- **WHEN** 新增 `manual_dispatch.rs` 和 `literals.rs` 文件
- **THEN** `src/eval/builtin.rs` MUST 声明 `pub(crate) mod manual_dispatch;`，`src/parse/expr/mod.rs` MUST 声明 `mod literals;`

#### Scenario: All existing tests pass after split
- **WHEN** 运行 `cargo test --test compile_test && cargo test --test stage_test && cargo test --test ast_test && cargo test --test common_test && cargo test --test bs_spec && cargo test --test ep_full`
- **THEN** 202/202 测试 MUST 全通过，零回归
