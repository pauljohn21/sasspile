## Why

sasspile 的内建函数名映射存在于三处重复代码中：`module_dispatch.rs`（`module_builtin_name` 的 130 条 match arm）、`builtin.rs`（`call_builtin` 的 130 条 match arm）、`builtin.rs`（`is_known_builtin` 的 130 条 match arm）。添加一个新内建函数需要同步修改三处，极易遗漏。通过派生宏（syn 3.0 + darling 0.24）将函数注册集中为单一数据源，宏展开自动生成三处代码，消除重复并防止遗漏。

选择 syn 3.0 + darling 0.24 而非 `macro_rules!` 的理由：长期维护中，派生宏提供更好的 IDE 支持（字段名补全、符号跳转、rename symbol）、更精确的编译错误定位（直接指向出错的字段/属性行而非宏调用处），以及更符合 Rust 生态习惯的 derive 模式。

## What Changes

- 新增 `sasspile-macros` proc-macro crate（workspace 成员），依赖 syn 3.0 + darling 0.24 + quote + proc-macro2
- 主 crate `sasspile` 通过 path 依赖 `sasspile-macros`
- 在 `module_dispatch.rs` 中定义各模块的结构体（如 `MathBuiltins`、`StringBuiltins` 等），用 `#[derive(BuiltinRegistry)]` 派生
- 每个结构体字段代表一个函数，通过 `#[aliases = [...]]` 属性声明模块限定名和别名
- 派生宏自动生成 `module_builtin_name`、`is_known_builtin`、模块分派 match arm
- 删除三处手工维护的重复 match arm
- `call_builtin` 中 meta 内联函数（type-of/inspect/if 等）和 CSS 透传分支保留原样
- 不改变任何运行时行为，不改变公共 API

## Capabilities

### New Capabilities
- `builtin-registry`: 派生宏驱动的内建函数注册表，从单一结构体声明生成名称映射、已知函数检查、模块分派三组代码

### Modified Capabilities

（无——此变更不改变任何 spec 级别行为，纯重构）

## Impact

- 新增 `sasspile-macros/` 目录和 crate（proc-macro = true）
- `Cargo.toml` 改为 workspace 模式，主 crate 依赖 `sasspile-macros`
- `src/eval/module_dispatch.rs`：从手工 match 改为结构体 + derive
- `src/eval/builtin.rs`：`call_builtin` 和 `is_known_builtin` 的 match arm 改为宏生成
- 新增编译依赖：syn 3.0、darling 0.24、quote、proc-macro2
- 无公共 API 变更
- 所有现有测试应零回归通过
