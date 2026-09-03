## ADDED Requirements

### Requirement: 源文件行数限制

所有 `src/` 目录下的 `.rs` 文件 MUST 不超过 500 行。组件/模块文件 SHOULD 不超过 300 行。业务逻辑文件 SHOULD 不超过 200 行。超出限制的文件 MUST 拆分为子模块。

#### Scenario: eval/mod.rs 不超过 500 行

- **WHEN** 检查 `src/eval/mod.rs` 文件行数
- **THEN** 文件行数 ≤ 500，超出部分拆分到 `eval/scope.rs`（Env + exit_scope）、`eval/hoist.rs`（hoist_css_imports）等子文件

#### Scenario: parse/ast/display.rs 不超过 500 行

- **WHEN** 检查 `src/parse/ast/display.rs` 文件行数
- **THEN** 文件行数 ≤ 500，超出部分按类型拆分到 `display_value.rs`、`display_node.rs` 等子文件

#### Scenario: eval/builtin/color_adjust.rs 不超过 500 行

- **WHEN** 检查 `src/eval/builtin/color_adjust.rs` 文件行数
- **THEN** 文件行数 ≤ 500，超出部分拆分到 `color_adjust_ops.rs` 等子文件

#### Scenario: eval/module.rs 不超过 500 行

- **WHEN** 检查 `src/eval/module.rs` 文件行数
- **THEN** 文件行数 ≤ 500，超出部分拆分到 `module_load.rs`、`module_bind.rs` 等子文件

#### Scenario: eval/builtin/color.rs 不超过 500 行

- **WHEN** 检查 `src/eval/builtin/color.rs` 文件行数
- **THEN** 文件行数 ≤ 500，超出部分拆分到 `color_create.rs` 等子文件

#### Scenario: css/mod.rs 不超过 500 行

- **WHEN** 检查 `src/css/mod.rs` 文件行数
- **THEN** 文件行数 ≤ 500，超出部分拆分到 `flatten.rs`、`serialize_expanded.rs` 等子文件

### Requirement: 测试文件行数限制

所有 `tests/` 目录下的 `.rs` 文件 MUST 不超过 500 行。超出限制的测试文件 MUST 按功能拆分为多个测试文件。

#### Scenario: 测试文件不超过 500 行

- **WHEN** 检查 `tests/` 目录下任何 `.rs` 文件行数
- **THEN** 文件行数 ≤ 500（当前所有测试文件均未超标，保持即可）
