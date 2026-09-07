## Why

`sass-spec` 覆盖率卡在 54.3%，trace 日志暴露了一个系统性 bug：当模块限定名（如 `string.unquote`）被调用时，`string`/`map`/`list`/`math`/`selector` 五个模块的 dispatch 函数未把限定名转换为全局名，导致内部 match 失败返回 `Ok(None)`，最终被上层判定为"Undefined function"。只有 `color_dispatch` 做了正确转换。这造成大量 `string.X`/`map.X` 等规范调用失败。

## What Changes

- **修复 `src/eval/builtin/dispatch.rs`** 中五个模块的 dispatch 函数，使其在调用底层 `call_xxx_builtin` 前，先将模块限定名（如 `string.unquote`、`map.get`）转换为全局名（如 `unquote`、`map-get`）
- 修改范围：`string_dispatch`、`map_dispatch`、`list_dispatch`、`math_dispatch`、`selector_dispatch`
- `color_dispatch` 已有正确实现，作为参考模板
- 不影响任何公共 API 签名，纯内部修复

## Capabilities

### New Capabilities

- `module-dispatch-name-resolution`: 模块限定名 → 全局名的统一转换，确保 `string.X`/`map.X`/`list.X`/`math.X`/`selector.X` 在 `@use "sass:xxx"` 上下文中被正确分派到内建函数实现

### Modified Capabilities

- 无（这是 bug 修复，未改变任何已有需求规范）

## Impact

- **受影响代码**：`src/eval/builtin/dispatch.rs`（5 个函数，约 10 行改动）
- **测试预期**：核心测试 202/202 不变，sass-spec 通过率预估从 54.3%（6427/11824）提升 30~80 个测试
- **风险**：极低 — 已有 `color_dispatch` 的相同模式可参考；修改仅在 name→global_name 转换层，不影响运行时逻辑
- **依赖**：无新增依赖
