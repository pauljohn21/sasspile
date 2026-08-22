## Why

`im` crate (v15.1.0) 作为不可变 HashMap 被引入，用于支持函数式状态传递。但代码实际从未使用其不可变 API（`update`/`without`/`update_in`），所有修改操作都是 `clone()` + `insert()` 模式——等价于 `std::collections::HashMap`。

更严重的是，`im` crate 已被官方标记为 **unmaintained**（RUSTSEC-2026-0248）：
- GitHub 仓库已于 2026-05-03 归档为只读（archived）
- 最后一次发布是 2022-04-29（v15.1.0），至今超过 4 年未更新
- crates.io 页面明确标注 "This crate has been marked as unmaintained"

继续依赖一个已废弃且存在安全公告的 crate 是不可接受的风险。移除它既消除了未使用的复杂依赖，又解决了安全审计问题。

## What Changes

- 从 `Cargo.toml` 移除 `im = "15"` 依赖
- 全局将 `use im::HashMap;` 替换为 `use std::collections::HashMap;`（涉及 13+ 个源文件）
- 将 `src/eval/builtin/map.rs` 中的 `&im::HashMap::new()` 替换为 `&std::collections::HashMap::new()`
- 验证 `Env` 的 `Clone` derive 在 `std::collections::HashMap` 下行为正确（O(n) clone 替代 O(1) 结构共享）

## Capabilities

### New Capabilities

无。

### Modified Capabilities

无。本次变更是纯内部依赖清理，不改变任何 spec 级别的行为。

## Impact

- **依赖**：移除 `im = "15"`，减少编译依赖链
- **源文件**（13+ 个）：
  - `src/eval/mod.rs` — `Env`/`ModuleExports`/`MixinDef`/`FunctionDef` 结构体字段类型
  - `src/eval/module_dispatch.rs` — `dispatch_builtin_module` 函数签名
  - `src/eval/builtin.rs` — `call_builtin` 函数签名 + `HashMap::new()`
  - `src/eval/builtin/color.rs`、`color_adjust.rs`、`color_gamut.rs`、`color_parse.rs`、`color_space.rs`、`list.rs`、`math.rs`、`math_helpers.rs`、`selector.rs`、`string.rs` — 函数签名中的 `kw_args: &HashMap<String, Value>`
  - `src/eval/builtin/map.rs` — `&im::HashMap::new()` 全路径引用
  - `src/eval/meta_ops.rs`、`src/eval/mixin.rs`、`src/eval/module.rs`、`src/eval/value/mod.rs` — `HashMap::new()` + 类型签名
- **性能**：`Env.clone()` 从 O(1)（im 结构共享）变为 O(n)（std 深拷贝）。但 `Env` 中 `HashMap` 字段通常很小（几十个条目），且已有大量 `Rc` 包装字段，实际影响预计可忽略
- **测试**：无行为变化，所有现有测试应保持通过
