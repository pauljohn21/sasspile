## ADDED Requirements

### Requirement: 内建函数模块自带注册

每个 `builtin/` 子模块（math/string/map/list/color/selector/meta）MUST 自带三个 `pub(crate)` 函数：`builtin_name`（模块限定名 → 全局名映射）、`is_known`（已知函数检查）、`dispatch`（分派到 call 函数）。MUST NOT 依赖外部 proc-macro 生成这些函数。

#### Scenario: math 模块自带注册

- **WHEN** `dispatch.rs` 需要检查 `math.abs` 是否为已知内建函数
- **THEN** 调用 `math::is_known("math.abs")` 返回 `true`，不通过 proc-macro 生成的代码

#### Scenario: meta 模块自带注册

- **WHEN** `dispatch.rs` 需要将 `meta.type-of` 映射到全局名
- **THEN** 调用 `meta::builtin_name("meta.type-of")` 返回 `Some("type-of")`

### Requirement: 删除 proc-macro 依赖

`sasspile-macros` crate MUST 从 workspace 中删除。`Cargo.toml` MUST NOT 包含 `sasspile-macros` 成员。`sasspile-macros` 的依赖（`syn`、`quote`、`proc-macro2`）MUST NOT 出现在 workspace 依赖树中。`eval/module_dispatch.rs` MUST NOT 使用 `#[derive(BuiltinRegistry)]` 或 `use sasspile_macros::BuiltinRegistry`。

#### Scenario: 无 proc-macro 依赖

- **WHEN** 执行 `cargo tree` 查看 sasspile 依赖树
- **THEN** 依赖树中不包含 `sasspile-macros`、`syn`、`quote`、`proc-macro2`

#### Scenario: 编译不依赖宏

- **WHEN** 执行 `cargo check` 编译 sasspile
- **THEN** 编译过程不触发任何 proc-macro 展开

### Requirement: dispatch.rs 纯转发

`builtin/dispatch.rs`（替代 `module_dispatch.rs`）MUST 仅做转发——依次调用各子模块的注册函数。MUST NOT 包含自己的 match 逻辑或函数名硬编码（rgba/rgb/darken/lighten/mix 等手工保留名 MUST 移入对应模块的注册函数）。

#### Scenario: module_builtin_name 转发

- **WHEN** 调用 `module_builtin_name("color.rgba")`
- **THEN** `dispatch.rs` 调用 `color::builtin_name("color.rgba")` 返回 `Some("rgba")`，不自己做 match

#### Scenario: is_known_builtin 转发

- **WHEN** 调用 `is_known_builtin("abs")`
- **THEN** `dispatch.rs` 调用各模块的 `is_known` 函数，不自己做 `matches!` 宏

### Requirement: 可见性统一

`eval/` 内部所有函数 MUST 使用 `pub(crate)` 可见性。颜色转换函数（`color_conv.rs` 的 `srgb_to_oklab` 等）MUST 从 `pub fn` 改为 `pub(crate) fn`。

#### Scenario: 内部函数不可 pub

- **WHEN** 检查 `eval/builtin/color_conv.rs` 的函数可见性
- **THEN** 所有函数标记为 `pub(crate) fn` 而非 `pub fn`
