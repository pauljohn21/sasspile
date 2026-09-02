## 1. Env 新增 consumed_config 字段

- [x] 1.1 在 `src/eval/mod.rs` 的 `Env` struct 中新增 `consumed_config: HashSet<String>` 字段
- [x] 1.2 实现 `with_consumed_config(self, config: HashSet<String>) -> Self` 链式方法
- [x] 1.3 实现 `add_consumed_config(self, key: String) -> Self` 链式方法
- [x] 1.4 实现 `get_consumed_config(&self) -> &HashSet<String>` 只读方法
- [x] 1.5 在 `Default` 实现中初始化 `consumed_config: HashSet::new()`
- [x] 1.6 在 `mod.rs` 顶部添加 `use std::collections::HashSet`（如未已有）

## 2. eval_variable 消费标记

- [x] 2.1 在 `src/eval/value/mod.rs` 的 `eval_variable` 函数中，`!default` 分支从 `pending_config` 取到值时，调用 `env.add_consumed_config(normalized_name)` 标记消费
- [x] 2.2 确保 `@forward with ($a: val !default)` 的配置变量也能被正确标记为已消费（在 `eval_forward` 传递 config 时跟踪）

## 3. load_module 验证迁移

- [x] 3.1 在 `src/eval/module.rs` 的 `load_module` 中，删除 `eval_nodes` 之前的 `collect_default_vars` 验证块（第 59-68 行）
- [x] 3.2 在 `eval_nodes` 之后，添加消费验证：检查 `config` 中每个 key 是否在 `final_env.get_consumed_config()` 中
- [x] 3.3 未被消费的 config key 报错 `"This variable was not declared with !default in the @used module."`
- [x] 3.4 添加 `#[tracing::instrument(skip(caller_env), fields(path = %path.display(), n_config = config.len()))]` span 覆盖验证逻辑

## 4. 清理旧代码

- [x] 4.1 删除 `src/eval/module_validation.rs` 中的 `collect_default_vars` 函数
- [x] 4.2 如果 `module_validation.rs` 变空，删除文件并移除 `mod.rs` 中的 `mod module_validation` 声明
- [x] 4.3 运行 `cargo check` 确认无编译错误

## 5. @forward 传播消费回传

- [x] 5.1 在 `eval_forward` 中，`load_module` 执行后，将子模块的 `consumed_config` 回传到当前 env
- [x] 5.2 确保 `@forward as prefix-*` 前缀剥离后的变量名正确匹配消费记录
- [x] 5.3 确保 `@forward show/hide` 不影响消费跟踪（只影响成员可见性）

## 6. 测试用例

- [ ] 6.1 在 `tests/compile_test.rs` 中添加 `through_forward_bare` 测试（`@forward "fwd"` 裸转发）
- [ ] 6.2 添加 `through_forward_transitive` 测试（多跳 `@forward` 链）
- [ ] 6.3 添加 `through_forward_with_default` 测试（`@forward with !default` 中间覆盖）
- [ ] 6.4 添加 `through_forward_as_prefix` 测试（`@forward as b-*` 前缀映射）
- [ ] 6.5 添加 `through_forward_show_hide` 测试（`@forward show/hide` 过滤）
- [ ] 6.6 添加 `distributed_vars` 测试（多文件 `_index.scss` 聚合验证）
- [ ] 6.7 添加 `forward_and_local_mixed` 测试（`@forward with` + 本地 `!default` 混合）

## 7. 回归验证

- [ ] 7.1 运行 `cargo test --test compile_test` — 43+7=50 通过
- [ ] 7.2 运行 `cargo test --test ep_full` — 121/121 通过
- [ ] 7.3 运行 `cargo test --test stage_test` — 10/10 通过
- [ ] 7.4 运行 `RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture` — 验证 !default 错误从 214 降至 <30
- [ ] 7.5 运行 `cargo clippy --workspace` — 无新 warning
