## 1. Env + ModuleExports 双层结构

- [x] 1.1 将 `Env` 的 `vars`/`mixins`/`functions` 拆分为 `local_vars`/`local_mixins`/`local_functions` + `forwarded_vars`/`forwarded_mixins`/`forwarded_functions`（spec 规则 1/2）
- [x] 1.2 将 `ModuleExports` 同步拆分为 `local_*` + `forwarded_*`
- [x] 1.3 新增 `member_sources: Rc<HashMap<String, Rc<PathBuf>>>` 字段，key 格式 `"fn:bem"`/`"mx:b"`/`"var:ns"`（spec 规则 3/4）
- [x] 1.4 实现 `define_local_function`/`define_local_mixin`/`define_forwarded_function`/`define_forwarded_mixin`
- [x] 1.5 `define_function`/`define_mixin` 路由到 local（`@function`/`@mixin` 节点定义的是当前文件成员）
- [x] 1.6 `get_function`/`get_mixin`/`lookup`/`has_var` 只查 local 表（spec 规则 1：forwarded 不可见）
- [x] 1.7 新增 `all_functions()`/`all_mixins()`/`all_vars()` 合并迭代器（local 优先于 forwarded，供 meta 反射用，spec 规则 2/9）
- [x] 1.8 `Env::clone()` 和 `Env::default()` 初始化所有新字段

## 2. bind_exports 重构

- [x] 2.1 修改 `bind_exports` 签名：新增 `mode: BindMode`（Use/Forward）和 `source_path: &Path` 参数
- [x] 2.2 Use 模式：从 `exports.local_*` 绑定到 `env.local_*`（spec 规则 5/8）
- [x] 2.3 Forward 模式：合并 `exports.local_*` + `exports.forwarded_*`（local 优先）后绑定到 `env.forwarded_*`（spec 规则 2/6）
- [x] 2.4 冲突检测：查 `member_sources`，同来源路径跳过（spec 规则 3），不同来源路径报错即使值相同（spec 规则 4）
- [x] 2.5 Forward 模式应用 show/hide 过滤（spec 规则 7）
- [x] 2.6 新增 `merge_with_local_precedence` 辅助函数

## 3. eval_use / eval_forward 修改

- [x] 3.1 `eval_use` star 模式：调 `bind_exports(mode=Use, source_path=&path)`（spec 规则 8）
- [x] 3.2 `eval_forward`：调 `bind_exports(mode=Forward, source_path=&path)` + show/hide 过滤（spec 规则 6/7）
- [x] 3.3 `load_module` exports 提取：`final_env.local_*` → `exports.local_*`，`final_env.forwarded_*` → `exports.forwarded_*`
- [x] 3.4 `load_module` 内部 env 初始化时设置 `member_sources` 为空 HashMap

## 4. 规则体传播修改 (rule.rs)

- [x] 4.1 `new_env.local_mixins` → 传播到 `return_env.local_mixins`
- [x] 4.2 `new_env.local_functions` → 传播到 `return_env.local_functions`
- [x] 4.3 `new_env.local_vars` → 传播到 `return_env.local_vars`
- [x] 4.4 `new_env.forwarded_*` → 同步传播到 `return_env.forwarded_*`

## 5. mixin.rs 查找路径修改

- [x] 5.1 `call_function` 中 `env.get_function` 只查 local（由决策 6 保证）
- [x] 5.2 `exec_mixin` 注入命名空间函数到 `local_functions`
- [x] 5.3 `call_user_function` 中 `env.get_function` 只查 local（由决策 6 保证）

## 6. meta_ops.rs 修改

- [x] 6.1 `merge_module_cache` 适配新结构
- [x] 6.2 `meta.module-functions` 使用 `all_functions()` 合并（local 优先于 forwarded）
- [x] 6.3 `meta.module-mixins` 使用 `all_mixins()` 合并
- [x] 6.4 `meta.module-variables` 使用 `all_vars()` 合并
- [x] 6.5 `meta.load-css` 的 exports 提取适配

## 7. value/ + builtin.rs 修改

- [x] 7.1 `value/mod.rs` 中 `env.has_var`/`env.lookup` 只查 local（由决策 6 保证）
- [x] 7.2 `value/display.rs` 中 `env.lookup` 只查 local
- [x] 7.3 `builtin.rs` 中 `mixin-exists`/`function-exists`/`variable-exists` 只查 local

## 8. 验证

- [x] 8.1 `cargo clippy --all-targets` 无新 warning
- [x] 8.2 `cargo test --test compile_test` 43/43 通过
- [x] 8.3 `cargo test --test stage_test` 10/10 通过
- [x] 8.4 `cargo test --test ast_test` 8/8 通过
- [x] 8.5 `cargo test --test common_test` 5/5 通过
- [x] 8.6 `cargo test --test bs_spec` 15/15 通过
- [x] 8.7 `cargo test --test ep_full` 从 10/121 提升到 121/121
- [x] 8.8 `RUST_LOG="sass_spec_full=info" cargo test --test sass_spec_full` 总通过率 2811/5362（高于基线 2810）
