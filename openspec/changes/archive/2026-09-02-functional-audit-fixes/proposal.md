## Why

架构审核发现两个文件超过 500 行限制（`eval/builtin.rs` 524 行、`parse/expr/prefix.rs` 503 行），`combine_selectors()` 使用嵌套 `for+push` 违反函数式迭代器规则，且 `call_user_function`/`bind_params` 中的 `env.clone()` 可优化为 move 语义。这些违规虽不影响功能，但与项目函数式 Rust 设计哲学不一致，长期会积累技术债务。

## What Changes

- 拆分 `src/eval/builtin.rs`（524 行）：将手工 match 分派分支（rgba/rgb/darken/lighten/mix/if/inspect/type-of 等）提取到 `src/eval/builtin/manual_dispatch.rs`
- 拆分 `src/parse/expr/prefix.rs`（503 行）：将字面量解析（Number/String/Hash/Color 等）提取到 `src/parse/expr/literals.rs`
- 重构 `combine_selectors()`：嵌套 `for+push` 改为迭代器笛卡尔积链（`flat_map` + `map` + `collect`）
- 优化 `call_user_function()`：`env.clone()` 改为 `env` move + `exit_scope` 恢复模式（与 `eval_rule` 一致）
- 优化 `bind_params()`：`env.clone()` 改为 `env` move 传入、返回新 Env 模式

## Capabilities

### New Capabilities
- `file-split-compliance`: 拆分超 500 行文件，确保所有源文件符合行数限制规范
- `functional-iterator-refactor`: 将遗留的命令式 `for+push` 模式重构为函数式迭代器链
- `env-clone-elimination`: 消除 `call_user_function` 和 `bind_params` 中的架构性 `env.clone()`

### Modified Capabilities
<!-- 无现有 spec 级别需求变更 -->

## Impact

- `src/eval/builtin.rs` → 拆分为 `builtin.rs` + `builtin/manual_dispatch.rs`
- `src/eval/builtin.rs` 模块声明新增 `pub(crate) mod manual_dispatch;`
- `src/parse/expr/prefix.rs` → 拆分为 `prefix.rs` + `expr/literals.rs`
- `src/parse/expr/mod.rs` 模块声明新增 `mod literals;`
- `src/eval/rule.rs` — `combine_selectors()` 重构
- `src/eval/mixin.rs` — `call_user_function()` 和 `bind_params()` 签名变更
- 测试基线：compile_test 43 + stage_test 10 + ast_test 8 + common_test 5 + bs_spec 15 + ep_full 121 = 202/202 不可回归
