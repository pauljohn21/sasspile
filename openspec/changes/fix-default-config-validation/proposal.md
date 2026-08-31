## Why

sasspile 的 `@use with` 配置变量验证过早——在 `eval_nodes` 执行之前就检查 `!default` 声明，导致通过 `@forward` 链转发的配置变量无法通过验证。这是 sass-spec 中 **214 次失败**的最大单点错误来源（`"This variable was not declared with !default in the @used module."`），占 `directives/use` 目录 109 次失败的全部。

## What Changes

- 将 `load_module` 中的 `!default` 验证从 `eval_nodes` **之前**移到**之后**
- 在 `Env` 中新增 `consumed_config: HashSet<String>` 字段，跟踪哪些 `pending_config` key 被 `!default` 变量赋值消费
- `eval_variable` 处理 `!default` 时，如果从 `pending_config` 取到值，标记该 key 为已消费
- `eval_nodes` 完成后，检查 `pending_config` 中未被消费的 key，报 `"not declared with !default"` 错误
- 删除 `module_validation.rs` 中的 `collect_default_vars` 函数（不再需要静态 AST 遍历）
- 正确处理 `@forward with ($a: val !default)` 的配置变量消费跟踪

## Capabilities

### New Capabilities

无新能力。

### Modified Capabilities

- `module-config-validation`: 配置变量验证从静态 AST 检查改为运行时消费跟踪——验证时机从 `eval_nodes` 前移到后，利用 `pending_config` 的消费记录判断变量是否声明了 `!default`

## Impact

- `src/eval/module_validation.rs` — 删除 `collect_default_vars`，新增消费跟踪验证逻辑
- `src/eval/module.rs` — `load_module` 中移动验证调用点
- `src/eval/mod.rs` — `Env` 新增 `consumed_config` 字段 + `add_consumed_config` / `get_consumed_config` 方法
- `src/eval/value/mod.rs` — `eval_variable` 中 `!default` 分支增加消费标记
- `tests/compile_test.rs` — 新增 `@forward` 链式配置测试用例
- 预期 sass-spec 提升 +150~200 通过（2902 → ~3100）
