## ADDED Requirements

### Requirement: spec_store 不使用 println!
spec-store CLI 工具 SHALL use `tracing::info!` for output instead of `println!`.

#### Scenario: cmd_stats uses tracing::info!
- **WHEN** `cmd_stats` renders markdown stats
- **THEN** it SHALL call `tracing::info!("{md}")` 而非 `println!("{md}")`

#### Scenario: cmd_trend uses tracing::info!
- **WHEN** `cmd_trend` renders ASCII chart
- **THEN** it SHALL call `tracing::info!("{chart}")` 而非 `println!("{chart}")`

#### Scenario: cmd_link uses tracing::info!
- **WHEN** `cmd_link` renders link output
- **THEN** it SHALL call `tracing::info!("{output}")` 而非 `println!("{output}")`

#### Scenario: cmd_diff uses tracing::info!
- **WHEN** `cmd_diff` renders diff output
- **THEN** it SHALL call `tracing::info!("{output}")` 而非 `println!("{output}")`

#### Scenario: spec_store test builds without clippy error
- **WHEN** running `cargo clippy --test spec_store`
- **THEN** it SHALL compile with zero errors related to `print_stdout`

### Requirement: 单文件行数上限合规
Each source file under `src/` SHALL contain at most 500 lines of code.

#### Scenario: all source files within line limit
- **WHEN** counting lines of every `.rs` file under `src/`
- **THEN** no file SHALL exceed 500 lines

### Requirement: unused code清理
The project SHALL not contain unused imports, variables, functions, or fields flagged by clippy.

#### Scenario: no unused imports
- **WHEN** running `cargo clippy --lib`
- **THEN** it SHALL NOT report any `unused_imports` warnings

#### Scenario: no unused variables
- **WHEN** running `cargo clippy --lib`
- **THEN** it SHALL NOT report any `unused_variables` warnings

#### Scenario: no dead code in src/
- **WHEN** running `cargo clippy --lib`
- **THEN** it SHALL NOT report `dead_code` warnings for functions or fields in `src/`

### Requirement: format! 内联变量
All `format!` and `writeln!` invocations SHALL use inline variable syntax where applicable.

#### Scenario: inline variables in error messages
- **WHEN** reviewing any `format!("{}", var)` in `src/`
- **THEN** it SHALL use `format!("{var}")` where `var` is a local binding
