## ADDED Requirements

### Requirement: 不依赖 im crate

sasspile SHALL NOT 依赖 `im` crate。所有 `HashMap` 使用 SHALL 来自 `std::collections::HashMap`。

#### Scenario: 编译时无 im 依赖

- **WHEN** 运行 `cargo build`
- **THEN** 编译产物中不包含 `im` crate 的任何代码

#### Scenario: 所有 HashMap 来自 std

- **WHEN** 在源代码中搜索 `im::`
- **THEN** 返回零结果

#### Scenario: 测试全部通过

- **WHEN** 运行 `cargo test --test compile_test`、`cargo test --test stage_test`、`cargo test --test ast_test`、`cargo test --test common_test`、`cargo test --test bs_spec`、`cargo test --test ep_full`
- **THEN** 所有测试通过率与移除 `im` 之前相同
