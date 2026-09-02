## 1. 拆分 eval/builtin.rs（524 → ≤500 行）

- [x] 1.1 创建 `src/eval/builtin/manual_dispatch.rs`，将 `call_builtin` 中的手工 match 分支（rgba/rgb/darken/lighten/mix/if/inspect/type-of 等）提取到 `pub(crate) fn manual_dispatch(...)` 函数
- [x] 1.2 在 `src/eval/builtin.rs` 中声明 `pub(crate) mod manual_dispatch;`，将 `call_builtin` 中的手工分支替换为 `manual_dispatch::manual_dispatch(&name, pos_args, kw_args, env)` 调用
- [x] 1.3 运行 `wc -l src/eval/builtin.rs` 确认 ≤ 500 行
- [x] 1.4 运行 `cargo check` 确认编译通过

## 2. 拆分 parse/expr/prefix.rs（503 → ≤500 行）

- [x] 2.1 创建 `src/parse/expr/literals.rs`，将 `parse_prefix` 中的字面量解析分支（Number/String/Hash/Color/Dollar/Ident 等）提取到 `pub(crate) fn parse_literal(&mut self) -> Result<Value>`
- [x] 2.2 在 `src/parse/expr/mod.rs` 中声明 `mod literals;`，在 `parse_prefix` 中将字面量分支替换为 `self.parse_literal()?` 调用
- [x] 2.3 运行 `wc -l src/parse/expr/prefix.rs` 确认 ≤ 500 行
- [x] 2.4 运行 `cargo check` 确认编译通过

## 3. 重构 combine_selectors — 迭代器链

- [x] 3.1 将 `src/eval/rule.rs` 中 `combine_selectors` 的嵌套 `for+push` 改为 `parents.iter().flat_map(|p| children.iter().map(|c| ...)).collect()` 模式
- [x] 3.2 验证选择器顺序与重构前一致（外层 parent 优先）
- [x] 3.3 运行 `cargo test --test compile_test` 确认零回归

## 4. 消除 call_user_function 的 env.clone()

- [x] 4.1 将 `call_user_function` 签名从 `env: &Env` 改为 `env: Env`（move），保存 `saved_*` HashMap 快照
- [x] 4.2 函数体内用 `env` 直接 bind 链式操作，消除 `env.clone()`，求值后用 `exit_scope` 恢复
- [x] 4.3 更新调用者 `call_function`（mixin.rs）——传入 `env.clone()` 给 `call_user_function`（调用者仍需原 env 后续使用）
- [x] 4.4 运行 `cargo test --test compile_test && cargo test --test ep_full` 确认零回归

## 5. 消除 bind_params 的 env.clone()

- [x] 5.1 将 `bind_params` 签名从 `env: &Env` 改为 `env: Env`（move），返回 `Result<Env>`
- [x] 5.2 函数体内用 `env` 直接 bind 链式操作，消除 `env.clone()`
- [x] 5.3 更新调用者 `exec_mixin`——将 `Self::bind_params(&mixin.params, args, &env)` 改为 `Self::bind_params(&mixin.params, args, env)`
- [x] 5.4 运行 `cargo test --test compile_test && cargo test --test ep_full` 确认零回归

## 6. 全量测试验证

- [x] 6.1 运行 `cargo test --test compile_test`（43 个）
- [x] 6.2 运行 `cargo test --test stage_test`（10 个）
- [x] 6.3 运行 `cargo test --test ast_test`（8 个）
- [x] 6.4 运行 `cargo test --test common_test`（5 个）
- [x] 6.5 运行 `cargo test --test bs_spec`（15 个）
- [x] 6.6 运行 `cargo test --test ep_full`（121 个）
- [x] 6.7 运行 `cargo test --test default_config_test -- --test-threads=1`（9 个）
- [x] 6.8 确认 202/202 全通过，零回归
- [x] 6.9 运行 `wc -l src/**/*.rs | sort -rn | head -5` 确认所有文件 ≤ 500 行
