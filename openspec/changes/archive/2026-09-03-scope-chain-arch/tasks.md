## 1. Scope 结构体引入（不改变现有 Env）

- [x] 1.1 在 `src/eval/env.rs` 新增 `Scope` 结构体，包含 `local_vars`、`local_mixins`、`local_functions`、`forwarded_vars`、`forwarded_mixins`、`forwarded_functions`、`global_writes` 字段和 `parent: Option<Rc<Scope>>`
- [x] 1.2 实现 `Scope::new()` — 创建 root scope（`parent = None`）
- [x] 1.3 实现 `Scope::new_child(parent: Rc<Scope>)` — 创建子 scope（`parent = Some(parent)`）
- [x] 1.4 实现 `Scope::lookup(name)` — 沿 parent 链向上查找变量
- [x] 1.5 实现 `Scope::get_mixin(name)` / `Scope::get_function(name)` — 沿链查找 mixin/function
- [x] 1.6 验证 `cargo check` 通过（新结构不影响现有代码）

## 2. Env 重构为 Rc<Scope> + 全局字段

- [x] 2.1 将 `Env` 的 6 个 HashMap 字段（`local_vars` 等）和 `global_writes` 移入 `Scope`，`Env.current` 改为 `Rc<Scope>`
- [x] 2.2 修改 `Env::new_env()` — 创建 root scope
- [x] 2.3 修改 `Env::bind(name, value)` — 写入 `current` scope 的 `local_vars`
- [x] 2.4 修改 `Env::lookup(name)` — 委托 `current.lookup(name)`
- [x] 2.5 修改 `Env::has_var(name)` — 委托 `current.lookup(name).is_some()`
- [x] 2.6 修改 `Env::define_local_mixin/define_forwarded_mixin` — 写入 `current` scope
- [x] 2.7 修改 `Env::define_local_function/define_forwarded_function` — 写入 `current` scope
- [x] 2.8 修改 `Env::get_mixin/get_function` — 委托 scope 链查找
- [x] 2.9 修改 `Env::get_mixin_ref_data` — 从 scope 链获取 mixin 数据
- [x] 2.10 修改 `Env::add_global_write(name, value)` — 写入 `current` scope 的 `global_writes`
- [x] 2.11 修改所有 `get_local_vars/get_local_mixins` 等 getter 方法 — 从 `current` scope 获取
- [x] 2.12 修改 `Env::enter_scope()` — 创建子 scope，设置 `parent` 指向当前 scope
- [x] 2.13 修改 `Env::exit_scope()` — 恢复 parent scope，传播 `!global` 写入和新增 mixin/function
- [x] 2.14 修改 `Env::merge_forwarded_to_local` — 在 `current` scope 上操作
- [x] 2.15 修改 `Env::with_namespace_var` — 保持 namespace exports 操作不变
- [x] 2.16 修改 `Env::remove_star_imported` — 在 `current` scope 上操作
- [x] 2.17 验证 `cargo check` 通过

## 3. 调用方迁移 — eval_rule

- [x] 3.1 修改 `eval_rule` — 用 `enter_scope()` 替代 6 次 HashMap clone
- [x] 3.2 修改 `eval_rule` — 用 `exit_scope()` 替代手动 `exit_scope(saved_*, ...)` 调用
- [x] 3.3 运行 `cargo test --test compile_test` 验证 43 个测试通过
- [x] 3.4 运行 `cargo test --test stage_test` 验证 10 个测试通过
- [x] 3.5 运行 `cargo test --test ast_test` 验证 8 个测试通过

## 4. 调用方迁移 — mixin/function 调用

- [x] 4.1 修改 `call_user_function` — 用 `enter_scope()` 替代 env clone
- [x] 4.2 修改 `eval_content` — 从 `content_env` 的 scope 快照恢复
- [x] 4.3 修改 `set_content` — 保存 `Rc<Scope>` 指针替代整个 `Env` clone
- [x] 4.4 修改 `get_content` — 从 scope 快照返回 content env
- [x] 4.5 运行 `cargo test --test common_test` 验证 5 个测试通过
- [x] 4.6 运行 `cargo test --test interp_test` 验证 15 个测试通过
- [x] 4.7 运行 `cargo test --test bs_spec -- --nocapture` 验证 15 个测试通过

## 5. 全量测试验证

- [x] 5.1 运行 `cargo test --test compile_test` — 43/43 通过
- [x] 5.2 运行 `cargo test --test stage_test` — 10/10 通过
- [x] 5.3 运行 `cargo test --test ast_test` — 8/8 通过
- [x] 5.4 运行 `cargo test --test common_test` — 5/5 通过
- [x] 5.5 运行 `cargo test --test interp_test` — 15/15 通过
- [x] 5.6 运行 `cargo test --test bs_spec -- --nocapture` — 15/15 通过
- [x] 5.7 运行 `cargo test --test ep_full -- --nocapture` — 121/121 通过
- [x] 5.8 运行 `cargo test --test default_config_test -- --test-threads=1` — 9/9 通过
- [x] 5.9 运行 sass-spec 全量 — 3327/5624 = 59%（基线 3324/5624 = 59%，+3）

## 6. Clippy + 格式化 + 提交

- [x] 6.1 运行 `cargo clippy --workspace` — 无新增 warning
- [x] 6.2 运行 `cargo fmt` — 格式化
- [x] 6.3 运行 `codegraph sync` — 同步索引
- [x] 6.4 Git commit — `feat: scope-chain-arch — Scope 结构体 + Rc<Scope> 父链零 clone 作用域管理 — 总计 202/202`
