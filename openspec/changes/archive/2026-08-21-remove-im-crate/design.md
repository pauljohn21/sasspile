## Context

sasspile 使用 `im = "15"` crate 提供不可变 `HashMap`，用于 `Env` 结构体的函数式状态传递。原始设计意图是利用 `im::HashMap` 的 O(1) 结构共享 clone，使每次 `Env::bind()`/`define_mixin()` 等操作无需深拷贝。

然而，代码实际从未使用 `im` 的不可变 API。所有修改操作采用 `let mut new = self.clone(); new.field.insert(k, v); new` 模式——这与 `std::collections::HashMap` + `Clone` derive 完全兼容。`im` crate 作为一个未发挥价值的依赖，增加了编译时间和二进制体积。

**安全背景**：`im` crate 已被 RustSec 标记为 unmaintained（RUSTSEC-2026-0248）。GitHub 仓库于 2026-05-03 归档为只读，最后一次发布是 2022-04-29（v15.1.0），超过 4 年未更新。继续依赖该 crate 会在 `cargo audit` 中产生安全警告。

当前 `im::HashMap` 出现在 13+ 个源文件中，用途分两类：
1. **结构体字段类型**（`Env`、`ModuleExports`、`MixinDef`、`FunctionDef`）— 4 个结构体共 11 个 `HashMap` 字段
2. **函数签名参数**（`kw_args: &HashMap<String, Value>`）— 只需 `.get()`/`.len()` 方法

## Goals / Non-Goals

**Goals:**
- 完全移除 `im` crate 依赖
- 所有 `im::HashMap` 替换为 `std::collections::HashMap`
- 所有现有测试保持通过（零行为变化）

**Non-Goals:**
- 不重构 `Env` 的 clone 模式（不改为 `Rc<Env>` 或其他共享方案）
- 不优化 `Env` 的 clone 性能（如果后续 benchmark 显示问题，另开 change）
- 不修改任何函数式编程风格（`bind`/`define_mixin` 等仍返回新 `Env`）

## Decisions

### 决策 1：直接全局替换 `use im::HashMap` → `use std::collections::HashMap`

**理由**：代码只使用了 `HashMap` 的 `new()`/`get()`/`insert()`/`contains_key()`/`keys()`/`iter()`/`len()` 方法，这些都是 `std::collections::HashMap` 和 `im::HashMap` 的公共 API。无需任何代码逻辑改动。

**替代方案考虑**：
- ❌ 逐文件迁移 + 测试 — 不必要，替换是机械性的，无行为差异
- ❌ 保留 `im` 但改用其不可变 API — 不可行：`im` crate 已被 RustSec 标记为 unmaintained（RUSTSEC-2026-0248），仓库已归档只读，必须移除

### 决策 2：接受 `Env.clone()` 从 O(1) 变为 O(n)

**理由**：
- `Env` 的 `HashMap` 字段通常很小（几十个变量/mixin/function 定义）
- `Env` 已有 6 个 `Rc` 包装字段（`content`、`content_env`、`extends`、`loaded_modules`、`module_cache`、`namespaces`），这些字段的 clone 已经是 O(1)
- `module_cache` 是 `Rc<HashMap<PathBuf, ModuleExports>>`，`Rc` clone 是 O(1)，不受影响
- 如果性能成为问题，后续可以通过将 `vars`/`mixins`/`functions` 包装在 `Rc` 中来解决

**替代方案考虑**：
- ❌ 将 `vars`/`mixins`/`functions` 改为 `Rc<HashMap>` — 增加复杂度，本次只做清理
- ❌ 将 `Env` 改为 `Rc<Env>` — 改变所有函数签名，风险过大

### 决策 3：`map.rs:285` 的 `&im::HashMap::new()` 特殊处理

**理由**：这是唯一一处使用 `im::` 全路径的代码。替换为 `&HashMap::new()`（文件已有 `use im::HashMap`，替换 import 后自动生效）。

## Risks / Trade-offs

- **[Env.clone() 性能回退]** → `Env.clone()` 从 O(1) 变为 O(n)。如果 benchmark 显示显著回退（>10%），后续 change 可将热路径 HashMap 字段包装在 `Rc` 中
- **[安全审计合规]** → 移除 `im` 后 `cargo audit` 不再报告 RUSTSEC-2026-0248 警告
- **[遗漏的 im 引用]** → 使用 `grep -r "im::" src/` 在迁移后验证零残留
- **[HashMap 迭代顺序差异]** → `im::HashMap` 和 `std::collections::HashMap` 的迭代顺序不同，可能影响某些依赖顺序的测试输出。但 sasspile 的 `Value::Map` 使用 `Vec<(Value, Value)>` 而非 `HashMap` 存储 map 数据，所以不受影响
