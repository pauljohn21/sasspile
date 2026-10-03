## Context

EP（element-plus）官方构建全量对比发现 **111/121 (92%) 文件与官方 dist 不一致**。这是 sasspile 进入企业级项目的系统级障碍。

### 问题链定位

```
.scss source → Lexer → Parser → Evaluator (call_user_function) → Serializer → CSS
                                                                    ↑ @function 调用未分派
```

追踪 `eval_call` 路径：
1. `Value::Call(name, args)` → `eval_call()` → `dispatch_function()` → `call_function()` (src/eval/value/mod.rs:326)
2. `call_function()` (src/eval/mixin.rs:177) 依次尝试：
   - CSS 保留函数（url/element/expression）`is_css_reserved_function`
   - 用户函数精确匹配 `env.get_function(name)`
   - 用户函数大小写不敏感匹配 `env.get_function_ci(name)`
   - CSS 原生函数（calc/env/var）— 仅无用户覆盖时走内建
   - 命名空间函数搜索（跳过 is_builtin）— `get_namespaces()` 遍历
   - 模块限定函数 `math.abs`
   - `call_builtin`
3. 全部未命中 → `dispatch_function` fallback → CSS 透传（输出 `joinVarName(...)` 字面量）

### 根因假设

自定义 `@function`（如 `joinVarName`、`getCssVar`、`bem`）定义在 `@use`/加载的模块中，但调用时 `env.get_function(name)` 和命名空间遍历均未命中。可能原因：

1. `@function` 定义未正确写入 `Env.local_functions`（同一文件内定义）
2. `@use` 加载模块后，函数的 `exports.all_functions()` 未包含该函数
3. `get_namespaces()` 查找逻辑存在大小写/命名空间匹配问题
4. 函数调用参数（List 类型）的 `Arg` → `Value` 转换丢失或格式错误

### 约束

- 不可参照 dart-sass 实现（GC 依赖，与 Rust 所有权不兼容）
- 修复不能破坏现有 202/202 核心测试 + sass-spec 66% 基线
- 所有状态变更通过 move 语义（`self -> Self`），无 `env.clone()`

## Goals / Non-Goals

**Goals:**
- EP 全量文件对比通过率从 ~0% 提升至 ≥80%
- 自定义 `@function` 在 `@import`/`@use` 后正确分派
- List 类型参数正确传递并展开

**Non-Goals:**
- 不影响内建函数分派链（已稳定）
- 不改变 CSS 透传 fallback 行为
- 不引入 JIT/反射机制

## Decisions

### Decision 1: 修复用户函数查找链

**现状**: `call_function()` 的 find path 中，命名空间遍历（mixin.rs:244-256）使用了 `filter(|exports| !exports.is_builtin)`，正确跳过了内建 sass:* 模块。但候选函数需确认 `@use` 加载模块后函数确实注册在 `exports.all_functions()` 中。

**选择**: 
- 在 `mixin.rs` 添加 trace span 记录每次 `call_function` 的完整 find path（从精确匹配到 namespace 遍历）
- 在 `env.rs` 验证 `local_functions` / `get_namespaces()` 插入逻辑
- 确保 `eval_func_def` 执行后函数同时写入 `Env.local_functions`（用于同文件调用）

**备选**: 
- 方案 A：在 `call_function` 中添加全局 fallback 扫描所有命名空间（含 is_builtin 除外）— 已存在，但可能匹配失败原因在其他环节
- 方案 B：修改 parser 的 `Value::Call` 生成逻辑 — 不在 call site 修复

**最终采用方案 A + 诊断插桩** — 先 trace 确认哪一步 miss。

### Decision 2: List 参数传递

**现状**: `collect_args` 将 `Value::List` 整体作为单一 pos_arg 传入。EP 的 `joinVarName(('color', 'primary'))` 需要 List 被 `bind_params` 正确拆分为多参数。

**选择**: 在 `call_user_function` 的参数绑定阶段检测 ArgList 类型并展开，或确保 `eval_value` 在函数调用上下文中展开 List。

### Decision 3: 诊断插桩优先

所有修复前必须：
1. 在 `call_function` / `call_user_function` 添加 `#[instrument]` span
2. 采集至少一个 EP 文件的完整 trace
3. 基于 trace 证据定位根因后再修改逻辑

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| 修复影响 sass-spec 现有行为 | 全量 sass-spec 回归对比 |
| trace 插桩降低性能 | 插桩仅在 debug/trace 级别 |
| List 参数展开语义变化影响现有 core tests | 专项测试覆盖 |

## Migration Plan

1. **诊断阶段**: 插桩 + trace 采集 + 根因定位
2. **修复阶段**: 修改 `eval_func_def` / `Env.bind_function` / `call_user_function` 参数绑定
3. **回归验证**: `cargo test --tests` 全量 + `cargo test --test ep_dist_test`
4. **清理阶段**: 降级 debug span 为 trace

## Open Questions

1. `@function` 定义的解析是否生成了正确的 `FunctionDef` 结构？
2. EP 中 `@use 'utils'` 加载时，模块注册顺序是否在 `@import` 之前完成？
3. `ModuleExports.all_functions()` 是否返回正确的函数列表？
4. `bind_params` 实现与 `call_user_function` 的参数绑定逻辑是否一致？（存在两个不同的参数绑定实现）
