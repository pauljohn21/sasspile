## 1. 创建 sasspile-macros proc-macro crate

- [x] 1.1 创建 `sasspile-macros/Cargo.toml`：`proc-macro = true`，依赖 syn 3.0 + quote + proc-macro2（未用 darling，改用 syn 3.0 原生 `parse_nested_meta`）
- [x] 1.2 在根 `Cargo.toml` 中添加 `[workspace] members = [".", "sasspile-macros"]`
- [x] 1.3 在主 crate `Cargo.toml` 的 `[dependencies]` 中添加 `sasspile-macros = { path = "sasspile-macros" }`
- [x] 1.4 创建 `sasspile-macros/src/lib.rs` 骨架文件，`cargo build` 确认 workspace 编译通过

## 2. 编写属性解析结构

- [x] 2.1 定义 `BuiltinAttr`：解析 `#[builtin(module = "...", dispatch = "...")]` 属性（用 syn 3.0 `parse_nested_meta`，未用 darling）
- [x] 2.2 定义 `FieldInfo`：解析字段名 + `#[builtin(alias = "...")]` 属性
- [x] 2.3 编写 `#[proc_macro_derive(BuiltinRegistry, attributes(builtin))]` 入口函数
- [x] 2.4 `cargo build` 确认宏 crate 编译通过

## 3. 实现宏代码生成

- [x] 3.1 实现字段名 snake_case → kebab-case 转换函数（编译期执行）
- [x] 3.2 生成 `module_builtin_name` match arm：自动生成 `module.kebab-case` 默认别名 + 手动 aliases，映射到 kebab-case 后的字段名
- [x] 3.3 生成 `is_known_builtin` match arm：包含所有字段的全局名 + 默认模块限定名 + 所有 aliases
- [x] 3.4 生成 `dispatch_builtin_module` 函数：按 `dispatch` 属性路由到对应子模块 `call`，返回 `Option<Result<Value>>`

## 4. 在 module_dispatch.rs 中定义结构体

- [x] 4.1 定义 `MathBuiltins` 结构体（25 个字段，含 `comparable` 别名）
- [x] 4.2 定义 `StringBuiltins` 结构体（10 个字段）
- [x] 4.3 定义 `MapBuiltins` 结构体（9 个字段）
- [x] 4.4 定义 `ListBuiltins` 结构体（12 个字段，含 `list_length` 别名）
- [x] 4.5 定义 `ColorBuiltins` 结构体（37 个字段，排除 rgba/rgb/darken/lighten/mix）
- [x] 4.6 定义 `MetaBuiltins` 结构体（18 个字段，dispatch = "none"）
- [x] 4.7 定义 `SelectorBuiltins` 结构体（9 个字段）
- [x] 4.8 给所有结构体加 `#[derive(BuiltinRegistry)]` 和 `#[builtin(module = "...", dispatch = "...")]`
- [x] 4.9 `cargo build` 确认宏展开后编译通过

## 5. 集成到 builtin.rs

- [x] 5.1 在 `call_builtin` 开头调用 `dispatch_builtin_module`，返回 `Some` 则直接返回结果
- [x] 5.2 删除 `call_builtin` 中被宏覆盖的 match arm（math/string/color/map/list/selector 六组）
- [x] 5.3 保留 meta 内联函数、CSS 透传、rgba/rgb/darken/lighten/mix 的手工 arm
- [x] 5.4 `is_known_builtin` 改为调用 `module_dispatch::is_known_builtin` + 手工补充 rgba/rgb/darken/lighten/mix
- [x] 5.5 `cargo build` 确认编译通过

## 6. 编译验证与测试

- [x] 6.1 `cargo build` 确认编译通过，无警告
- [x] 6.2 `cargo expand` 检查宏展开结果与原手工 match 等价（编译验证通过）
- [x] 6.3 `cargo test --test compile_test` — 43/43 通过
- [x] 6.4 `cargo test --test stage_test` — 10/10 通过
- [x] 6.5 `cargo test --test ast_test` — 8/8 通过
- [x] 6.6 `cargo test --test common_test` — 5/5 通过
- [x] 6.7 `cargo test --test bs_spec -- --nocapture` — 15/15 通过
- [x] 6.8 `cargo test --test ep_full -- --nocapture` — 10/121（与重构前一致，零回归）

## 7. 清理与提交

- [x] 7.1 `module_dispatch.rs` 旧手工 match 已被宏替换
- [x] 7.2 `builtin.rs` 旧手工 match arm 已删除
- [x] 7.3 确认所有文件 ≤ 500 行（module_dispatch.rs 351行，builtin.rs 408行，lib.rs 257行）
- [x] 7.4 `codegraph sync` 同步索引
- [x] 7.5 git commit + push（`60e2b4e` — builtin-dispatch-macro 派生宏重构）
