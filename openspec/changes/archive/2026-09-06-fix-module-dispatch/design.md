## Context

`sass-spec` 54.3% 覆盖率。通过 trace 日志（`RUST_LOG=trace`）定位到根因：

`src/eval/builtin/dispatch.rs` 中，模块内建函数分派采用两层架构：
1. `xxx_is_known(name)` — 接受模块限定名（`string.unquote`）和全局名（`unquote`）
2. `call_xxx_builtin(name, ...)` — 内部 `match name` 只匹配全局名

**bug 位置**：5 个 dispatch 函数在 `true` 分支直接把模块限定名传给 `call_xxx_builtin`，导致 match 没有对应分支，返回 `Ok(None)`。

```rust
// 当前错误模式 (string/map/list/math/selector)
match string_is_known(name) {       // "string.unquote" → true ✓
    true => call_string_builtin(name, ...) // "string.unquote" → match "unquote" ✗ None
```

**正确模板**：`color_dispatch`（line 296）已实现转换模式。

## Goals / Non-Goals

**Goals:**
- 让 `string`/`map`/`list`/`math`/`selector` 的 dispatch 函数在调用底层前转换为全局名
- 纯内部修复，零 API 变更
- 不破坏现有 54.3% 通过率

**Non-Goals:**
- 不修改任何现有通过的行为
- 不新增内建函数
- 不触及 `color_dispatch`（已正确）

## Decisions

### Decision 1: 在 dispatch 函数内转换 vs 在底层 call 函数内转换？

**选择：在 dispatch 函数内转换**（与 `color_dispatch` 一致）

| 方案 | 优点 | 缺点 |
|------|------|------|
| **A. dispatch 内转换** | 各模块职责清晰；与 color 一致 | 5 处重复 |
| B. 底层 call 内转换 | 统一逻辑 | 每个 call 函数需额外匹配逻辑；打破现有简洁结构 |

**Rationale**：一致性优先。`color_dispatch` 已验证模式 A 可行，且 `dispatch.rs` 本身就是分派层，转换逻辑归属此处语义正确。

### Decision 2: 转换实现方式

使用各模块已有的 `xxx_builtin_name(name) -> Option<&'static str>` 函数 + `unwrap_or(name)` 回退：

```rust
let global_name = string_builtin_name(name).unwrap_or(name);
call_string_builtin(global_name, pos_args, kw_args)
```

**Rationale**：`xxx_builtin_name` 同时匹配限定名和全局名，找不到时 `unwrap_or(name)` 保持原名传入，安全无副作用。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| `xxx_builtin_name` 的查找遗漏某个合法名称 | 函数逻辑与 `xxx_is_known` 使用同一 NAME 数组，不会遗漏 |
| 转换影响手写测试中直接调用全局名的路径 | `unwrap_or(name)` 保证全局名原样传入，完全透明 |
