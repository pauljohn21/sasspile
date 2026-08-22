## Why

sasspile 的模块成员管理存在系统性架构缺陷：`Env` 中的 `functions`、`mixins`、`vars` 是单个扁平 HashMap，**不区分成员来源**。通过深度分析 sass-spec `.hrx` 文件的 input/output，发现以下 7 条精确语义规则当前无法满足：

1. `@forward` 成员在当前文件不可见（`inaccessible.hrx: local/`）
2. local 定义遮蔽 forwarded 定义（`shadowed.hrx`）
3. 同一模块 forward 两次不冲突（`bare.hrx: no_conflict/`）
4. 不同模块同值也冲突（`conflict.hrx: same_value/`）—冲突基于来源路径而非值
5. `@use as *` 同一模块两次不冲突（`use/member/global.hrx: no_conflict/`）
6. `@forward show/hide` 过滤（`visibility.hrx`）
7. `@forward as prefix-*` 前缀重映射（`as.hrx`）

当前症状：ep_full 仅 10/121 (8%)，全部 111 个失败为 `bind_exports` 误报冲突。

## What Changes

- **重构 `Env` 成员管理为双层结构**：`local_*`（当前文件定义 + `@use as *` 导入，当前文件可见）和 `forwarded_*`（`@forward` 导出，当前文件不可见，只传递给下游）
- **新增 `member_sources: Rc<HashMap<String, Rc<PathBuf>>>`**：追踪每个成员的来源模块路径，用于冲突判定（规则 3/4）
- **重构 `bind_exports`**：Use 模式写入 local 表，Forward 模式写入 forwarded 表；冲突判定基于来源路径而非值比较（规则 3/4）
- **重构成员查找**：`get_function`/`get_mixin`/`lookup` 只查 local 表（规则 1：forwarded 不可见）
- **重构 `@forward` 传递**：合并上游 local + forwarded（local 优先）后写入当前 forwarded 表（规则 2）
- **重构 `ModuleExports`**：新增 `forwarded_*` 字段，模块导出携带双层结构
- **新增 `all_functions()`/`all_mixins()`/`all_vars()`**：合并迭代器（local 优先于 forwarded），供 meta 反射使用（规则 2/9）

## Capabilities

### New Capabilities

（无——`forward-conflict-detection` 和 `module-member-access` 已存在）

### Modified Capabilities

- `forward-conflict-detection`: 修改冲突检测从"同名即报错"到"基于来源路径判定：同来源不冲突，不同来源即使同值也报错"
- `module-member-access`: 修改成员查找为只查 local 表（forwarded 不可见）；新增 local 遮蔽 forwarded 和 show/hide 过滤

## Impact

- **代码**：
  - `src/eval/mod.rs` — `Env` + `ModuleExports` 结构体重构（核心改动）
  - `src/eval/module.rs` — `bind_exports`/`eval_use`/`eval_forward`/`load_module` 重构
  - `src/eval/rule.rs` — 规则体成员传播修改
  - `src/eval/mixin.rs` — `call_function`/`exec_mixin` 查找路径修改
  - `src/eval/meta_ops.rs` — `merge_module_cache`/反射函数修改
  - `src/eval/builtin.rs` — `mixin-exists`/`function-exists` 查找路径修改
  - `src/eval/value/mod.rs` — 变量查找路径修改
  - `src/eval/value/display.rs` — 变量查找路径修改
- **测试**：ep_full 从 10/121 预期提升到 101+/121；sass-spec `no_conflict/`、`shadowed.hrx`、`inaccessible.hrx` 通过
- **无破坏性变更**：真正的冲突（不同来源路径的同名成员）仍然报错
