## ADDED Requirements

### Requirement: 诊断工具统一入口

所有 sass-spec 诊断和统计测试 MUST 集中在单一文件 `tests/diagnostic_runner.rs` 中。MUST NOT 存在多个独立诊断文件（cf_diag.rs、cfs_diag.rs、css_diag.rs、diag_detail.rs、diag_directives.rs、expr_diag.rs、use_diag.rs 等）。

#### Scenario: 单一诊断文件
- **WHEN** 运行 `cargo test --test diagnostic_runner -- --nocapture`
- **THEN** 所有 sass-spec 目录（core_functions、directives、values、css 等）的诊断和统计测试均可执行

#### Scenario: 按主题运行子集
- **WHEN** 运行 `cargo test --test diagnostic_runner diag_math -- --nocapture`
- **THEN** 仅运行 math 目录的诊断，不影响其他目录

### Requirement: 共享 HRX 编译逻辑

`diagnostic_runner.rs` MUST 只定义一次 `parse_case()`、`compile_case()`、`collect_hrx_files()` 函数。MUST NOT 在多个文件中重复这些函数。

#### Scenario: 无重复 parse_hrx
- **WHEN** 搜索 `fn parse_hrx` 或 `fn parse_case` 在 tests/ 目录
- **THEN** 仅 diagnostic_runner.rs 中存在一份定义

### Requirement: compile_test 聚焦核心管线

`compile_test.rs` MUST ≤ 500 行，仅包含编译器核心管线功能测试（变量、嵌套、@for、@mixin、@use、@extend 等）。CSS 色彩空间序列化测试 MUST 位于独立的 `compile_color_test.rs`。

#### Scenario: compile_test 不超限
- **WHEN** 执行 `wc -l tests/compile_test.rs`
- **THEN** 行数 ≤ 500

#### Scenario: 色彩空间测试独立
- **WHEN** 运行 `cargo test --test compile_color_test`
- **THEN** lab/lch/oklab/oklch 相关序列化测试全部通过

### Requirement: 无临时文件

tests/ 目录 MUST NOT 包含调试用临时文件（extend_debug.rs、trace_table.rs、diag_hsl2.rs、diag_hsl3.rs 等）。MUST NOT 包含已被替代的工具文件（failures_inspect.rs、failures_analyze.rs）。

#### Scenario: 无未跟踪临时文件
- **WHEN** 执行 `git status tests/`
- **THEN** 无未跟踪的 `*_debug.rs` 或 `*_diag_*.rs` 临时文件

#### Scenario: 无超限文件
- **WHEN** 执行 `wc -l tests/*.rs | awk '$1 > 500'`
- **THEN** 输出为空

## MODIFIED Requirements

### Requirement: 诊断测试调用方式

原来通过 `cargo test --test cf_diag` 等方式调用的诊断测试 MUST 改为 `cargo test --test diagnostic_runner`。

#### Scenario: 旧调用路径不再可用
- **WHEN** 执行 `cargo test --test cf_diag`
- **THEN** Cargo 报告 "can't find test target cf_diag"

#### Scenario: 新调用路径可用
- **WHEN** 执行 `cargo test --test diagnostic_runner diag_math -- --nocapture`
- **THEN** 输出与原来 `cargo test --test cf_diag diag_math -- --nocapture` 相同的结果

### Requirement: file_size_check 整合

`file_size_check.rs` MUST 删除，其检查测试 MUST 并入 `compile_test.rs`（作为编译管线回归的一部分）。

#### Scenario: 文件行数检查仍可用
- **WHEN** 运行 `cargo test --test compile_test check_file_size_limits`
- **THEN** 触发同样的 >500 行文件检查 panic
