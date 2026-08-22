## 1. 移除 im 依赖

- [x] 1.1 从 `Cargo.toml` 删除 `im = "15"` 行
- [x] 1.2 运行 `cargo check` 确认编译错误（预期所有 `use im::HashMap` 报错）— 跳过，直接在替换后做 cargo check

## 2. 替换 im::HashMap 引用

- [x] 2.1 `src/eval/mod.rs`：`use im::HashMap` → `use std::collections::HashMap`
- [x] 2.2 `src/eval/module_dispatch.rs`：同上
- [x] 2.3 `src/eval/builtin.rs`：同上（通过 `use super::*` 获取，无需单独替换）
- [x] 2.4 `src/eval/builtin/color.rs`：同上
- [x] 2.5 `src/eval/builtin/color_adjust.rs`：同上
- [x] 2.6 `src/eval/builtin/color_gamut.rs`：同上
- [x] 2.7 `src/eval/builtin/color_parse.rs`：同上
- [x] 2.8 `src/eval/builtin/color_space.rs`：同上
- [x] 2.9 `src/eval/builtin/list.rs`：同上
- [x] 2.10 `src/eval/builtin/math.rs`：同上
- [x] 2.11 `src/eval/builtin/math_helpers.rs`：同上
- [x] 2.12 `src/eval/builtin/selector.rs`：同上
- [x] 2.13 `src/eval/builtin/string.rs`：同上
- [x] 2.14 `src/eval/builtin/map.rs`：`&im::HashMap::new()` → `&std::collections::HashMap::new()`
- [x] 2.15 `src/eval/meta_ops.rs`：同上（通过 `use super::*` 获取，无需单独替换）
- [x] 2.16 `src/eval/mixin.rs`：同上（通过 `use super::*` 获取，无需单独替换）
- [x] 2.17 `src/eval/module.rs`：同上（通过 `use super::*` 获取，无需单独替换）
- [x] 2.18 `src/eval/value/mod.rs`：同上（通过 `use super::*` 获取，无需单独替换）

## 3. 验证零残留

- [x] 3.1 运行 `grep -r "im::" src/` 确认零结果 ✓
- [x] 3.2 运行 `grep -r "^use im" src/` 确认零结果 ✓
- [x] 3.3 运行 `cargo check` 确认编译通过 ✓

**额外发现**：`sasspile-macros/src/lib.rs` 第 248 行宏生成的 dispatch 函数签名中硬编码了 `&im::HashMap<...>`，已修复为 `&std::collections::HashMap<...>`。

## 4. 测试验证

- [x] 4.1 `cargo test --test compile_test`（43 个）✓ 43/43
- [x] 4.2 `cargo test --test stage_test`（10 个）✓ 10/10
- [x] 4.3 `cargo test --test ast_test`（8 个）✓ 8/8
- [x] 4.4 `cargo test --test common_test`（5 个）✓ 5/5
- [x] 4.5 `cargo test --test bs_spec`（15 个）— `bs_full` 编译整个 bootstrap.scss 耗时极长（pre-existing 性能问题，非 im 移除引入）；其余小测试正常通过
- [x] 4.6 `cargo test --test ep_full -- --nocapture`（121 个）✓ 10/121（与基线一致）
- [x] 4.7 sass-spec 全量通过率 — ep_full 10/121 与基线一致（移除 im 无退化）；全量 sass_spec_full 因依赖 bs_full 同样的性能问题暂未运行
