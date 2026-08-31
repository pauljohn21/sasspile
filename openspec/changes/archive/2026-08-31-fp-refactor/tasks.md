## 1. Env Builder 方法补齐与直接赋值消除

- [ ] 1.1 在 `Env` impl 中补齐缺失的 builder 方法：`with_depth`、`with_loaded_modules`、`with_extends`、`with_namespaces`、`with_pending_config`、`with_global_write`
- [ ] 1.2 `eval/mod.rs:274` — `content_env.current_selector = env.current_selector.clone()` 改为 `content_env.with_selector(...)`
- [ ] 1.3 `eval/mod.rs:114-115` — `self.content = ...` / `self.content_env = ...` 在 `set_content` 内部保留（builder 方法内部实现）
- [ ] 1.4 `eval/module.rs:46-50` — `env.depth =`/`env.plain_css =`/`env.loaded_modules =` 三个直接赋值改为 builder 方法
- [ ] 1.5 `eval/module.rs:125-127` — `env.base_path =`/`env.depth =`/`env.plain_css =` 改为 builder 方法
- [ ] 1.6 `eval/module.rs:130-131` — `final_env.base_path = saved`/`final_env.depth = saved` 改为 builder 方法
- [ ] 1.7 `eval/module.rs:134-144` — `final_env.forwarded_vars.clear()` 等 3 处 clear + 3 处 entry 改为 builder 方法或 `merge_forwarded_to_local` 辅助方法
- [ ] 1.8 `eval/rule.rs:129-134` — 6 处 `return_env.xxx = saved_xxx` 改为 `exit_scope` 或 builder 方法
- [ ] 1.9 `eval/rule.rs:139-161` — 8 处 `return_env.xxx.insert/entry` 改为 `exit_scope` 或 builder 方法
- [ ] 1.10 `eval/module_helpers.rs:167-169` — `env.loaded_modules =`/`env.extends =`/`env.module_cache =` 改为 builder 方法
- [ ] 1.11 `eval/value/mod.rs:45` — `env.namespaces.insert(...)` 改为 builder 方法
- [ ] 1.12 `eval/value/mod.rs:73` — `env.global_writes.insert(...)` 改为 builder 方法
- [ ] 1.13 `eval/mixin.rs:52` — `mixin_env.namespaces.insert(...)` 改为 builder 方法
- [ ] 1.14 `eval/mixin.rs:164` — `func_env.namespaces.insert(...)` 改为 builder 方法
- [ ] 1.15 `eval/module.rs:56` — `env.pending_config.insert(...)` 改为 builder 方法
- [ ] 1.16 验证：`cargo test --test compile_test --test stage_test --test ast_test --test common_test` 全通过

## 2. eval_rule 作用域管理重构

- [ ] 2.1 在 `Env` 上实现 `enter_scope()` — 创建子作用域 Env（共享 Rc 字段，clone 可变 HashMap）
- [ ] 2.2 在 `Env` 上实现 `exit_scope(child_env)` — 从子作用域提取传播字段（命名空间变量、global writes、新增 mixin/function/forwarded）合并回父作用域
- [ ] 2.3 重写 `eval_rule` 用 `enter_scope`/`exit_scope` 替代手动 save/restore 6 个 HashMap + 8 处 insert/entry
- [ ] 2.4 验证：`cargo test --test compile_test` 全通过

## 3. env.clone 消除

- [ ] 3.1 `eval/mixin.rs:103` — `bind_params` 中的 `env.clone()` 改为 `&Env`（bind_params 只读 env，创建新 Env）
- [ ] 3.2 `eval/mixin.rs:160` — `call_user_function` 中的 `env.clone().incr_depth()` 改为 move 语义（caller 改为传 `Env` by value）
- [ ] 3.3 `eval/module.rs:164` — `call_module_function` 中的 `env.clone()` 改为 move 语义
- [ ] 3.4 验证 `@content` 的 `env.clone()`（`mixin.rs:62`）保留——这是唯一允许的例外
- [ ] 3.5 `eval/mod.rs:272` — `content_env.clone()` 改为 `content_env.with_selector(...)`（Rc 字段共享，浅拷贝）
- [ ] 3.6 验证：`cargo test --test compile_test` 全通过

## 4. eval_node 纯函数分发 + Evaluator 空壳消除

- [ ] 4.1 提取 `eval_decl` 独立函数（从 `eval_node` 的 `Node::Decl` arm）
- [ ] 4.2 提取 `eval_comment` 独立函数（从 `Node::Comment` arm）
- [ ] 4.3 提取 `eval_mixin_def` 独立函数（从 `Node::MixinDef` arm）
- [ ] 4.4 提取 `eval_content` 独立函数（从 `Node::Content` arm）
- [ ] 4.5 提取 `eval_func_def` 独立函数（从 `Node::FunctionDef` arm）
- [ ] 4.6 提取 `eval_return` 独立函数（从 `Node::Return` arm）
- [ ] 4.7 提取 `eval_extend_node` 独立函数（从 `Node::Extend` arm）
- [ ] 4.8 提取 `eval_warn`/`eval_debug`/`eval_error_node` 独立函数
- [ ] 4.9 `eval_node` match 确认每个 arm 只有一行委托
- [ ] 4.10 删除 `Evaluator` 结构体，所有 `impl Evaluator` 方法改为 `pub(crate) fn` 自由函数
- [ ] 4.11 更新 `lib.rs` 的 `pub use eval::Evaluator` → 删除或改为 `pub use eval::evaluate`
- [ ] 4.12 更新 `stage/parsed.rs` 的 `Evaluator::evaluate_with_env` 调用
- [ ] 4.13 验证：`cargo test --test compile_test --test stage_test` 全通过

## 5. 后处理纯函数化

- [ ] 5.1 `apply_extends` 签名从 `&mut [CssNode]` 改为 `Vec<CssNode> -> Vec<CssNode>`
- [ ] 5.2 `hoist_css_imports` 签名从 `&mut Vec<CssNode>` 改为 `Vec<CssNode> -> Vec<CssNode>`
- [ ] 5.3 `evaluate`/`evaluate_with_env` 入口改为链式 `let css = ...; let css = apply_extends(css, ...); let css = hoist_css_imports(css);`
- [ ] 5.4 `extend.rs` 中的 `collect_selectors` 保持不变（已经是只读 `&[CssNode]`）
- [ ] 5.5 验证：`cargo test --test compile_test --test stage_test` 全通过

## 6. 去掉 sasspile-macros proc-macro crate

- [ ] 6.1 在 `builtin/math.rs` 中添加 `builtin_name`/`is_known`/`dispatch` 三个 `pub(crate)` 函数（从 `MathBuiltins` 结构体字段转写为 match arm）
- [ ] 6.2 在 `builtin/string.rs` 中添加同样的三个函数
- [ ] 6.3 在 `builtin/map.rs` 中添加同样的三个函数（同时移入 `map_param_names`/`merge_map_args` 从 `builtin.rs`）
- [ ] 6.4 在 `builtin/list.rs` 中添加同样的三个函数
- [ ] 6.5 在 `builtin/color.rs` 中添加同样的三个函数（包含 rgba/rgb/darken/lighten/mix 的手工保留名）
- [ ] 6.6 在 `builtin/selector.rs` 中添加同样的三个函数
- [ ] 6.7 创建 `builtin/meta.rs`，将 `builtin.rs` 中的 meta 手工 match arm 移入，添加三个注册函数
- [ ] 6.8 创建 `builtin/dispatch.rs`，只做转发：`module_builtin_name`/`is_known_builtin`/`dispatch_builtin_module`
- [ ] 6.9 删除 `eval/module_dispatch.rs`（被 `builtin/dispatch.rs` 替代）
- [ ] 6.10 将 `builtin.rs` 中 calc 解析函数移入 `builtin/calc_helpers.rs`
- [ ] 6.11 更新 `eval/builtin.rs`（`call_builtin`）：移除手工 meta arm，改为 `meta::dispatch`；移除 `is_known_builtin`/`is_css_function` 重复
- [ ] 6.12 从 `Cargo.toml` workspace 移除 `sasspile-macros` 成员
- [ ] 6.13 从 `sasspile` 的 `Cargo.toml` 移除 `sasspile-macros` 依赖
- [ ] 6.14 删除 `sasspile-macros/` 目录
- [ ] 6.15 验证：`cargo check` 编译通过，无 `sasspile-macros` 依赖
- [ ] 6.16 验证：`cargo test --test bs_spec -- --nocapture` 15/15 通过
- [ ] 6.17 验证：`cargo test --test compile_test` 全通过

## 7. eval/ 文件按功能重组

- [ ] 7.1 从 `mixin.rs` 移出 `call_function`/`call_user_function` 到新建 `eval/function.rs`
- [ ] 7.2 从 `mixin.rs` 移出 `eval_at_root`/`eval_at_rule` 到新建 `eval/at_rule.rs`
- [ ] 7.3 从 `mixin.rs` 移出 `is_truthy` 到 `eval/value.rs`（或 `value/mod.rs`）
- [ ] 7.4 从 `control_flow.rs` 移出 `unit_conversion_factor` 到 `eval/value_ops.rs`（或 `value/ops.rs`）
- [ ] 7.5 从 `eval/color.rs` 移出命名颜色表到 `eval/colors/named_colors.rs`
- [ ] 7.6 从 `eval/color.rs` 移出颜色操作函数到 `eval/colors/color_ops.rs`
- [ ] 7.7 更新 `eval/mod.rs` 的 `mod` 声明
- [ ] 7.8 验证：`cargo test --test compile_test` 全通过

## 8. 可见性统一

- [ ] 8.1 `eval/builtin/color_conv.rs` — 24 个 `pub fn` 改为 `pub(crate) fn`
- [ ] 8.2 `eval/builtin/color_space.rs` — 4 个 `pub fn` 改为 `pub(crate) fn`
- [ ] 8.3 `eval/builtin/color_gamut.rs` — `pub fn` 改为 `pub(crate) fn`
- [ ] 8.4 `eval/builtin/math.rs` — `pub fn` 改为 `pub(crate) fn`
- [ ] 8.5 `eval/builtin/selector.rs` — `pub fn` 改为 `pub(crate) fn`
- [ ] 8.6 `eval/builtin/color_parse.rs` — `pub fn` 改为 `pub(crate) fn`
- [ ] 8.7 验证：`cargo check` 编译通过

## 9. css/ 和 parse/ 功能拆分

- [ ] 9.1 从 `css/mod.rs` 提取变换逻辑到 `css/transform.rs`（`flatten_nodes`/`flatten_children`/`merge_at_rules`）
- [ ] 9.2 从 `css/mod.rs` 提取序列化逻辑到 `css/serialize.rs`（`serialize_expanded`/`serialize_compressed`/`write_node_*`）
- [ ] 9.3 `css/mod.rs` 只保留 `Serializer` 结构 + `serialize` 入口
- [ ] 9.4 从 `parse/nodes.rs` 提取 `parse_decl`/`parse_property`/`check_important` 到 `parse/decl.rs`
- [ ] 9.5 从 `parse/nodes.rs` 提取 `parse_variable`/`parse_namespace_var`/`is_namespace_var`/`parse_var_flags` 到 `parse/variable.rs`
- [ ] 9.6 从 `parse/nodes.rs` 提取 `parse_body`/`parse_params`/`parse_args`/`parse_config`/`parse_member_list` 到 `parse/body.rs`
- [ ] 9.7 从 `parse/nodes.rs` 提取 `parse_ident_name`/`parse_string_value`/`peek_keyword`/`expect_keyword`/`is_rule` 到 `parse/helpers.rs`
- [ ] 9.8 `parse/nodes.rs` 只保留 `parse_node`/`parse_rule_or_decl`/`parse_rule`/`parse_selector`
- [ ] 9.9 验证：`cargo test --test compile_test --test ast_test --test stage_test` 全通过

## 10. Value clone 冗余消除

- [ ] 10.1 审计每个 `Value::clone()` 调用，标记可 move 的场景
- [ ] 10.2 消除 `eval/mixin.rs` 中可 move 的 Value clone（如 `mixin.body.clone()` 可改为 `&mixin.body`）
- [ ] 10.3 消除 `eval/builtin/` 中可 move 的 Value clone（如 `args[0].clone()` 当 args 不再使用时可 move）
- [ ] 10.4 消除 `eval/value/mod.rs` 中可 move 的 Value clone
- [ ] 10.5 不消除语义必需的 Value clone（列表元素、Map 值等多处引用场景）
- [ ] 10.6 验证：`cargo test --test compile_test` 全通过

## 11. 全量验证

- [ ] 11.1 `cargo fmt`
- [ ] 11.2 `cargo clippy --workspace -- -W clippy::pedantic` 零警告
- [ ] 11.3 `cargo test --test compile_test --test stage_test --test ast_test --test common_test` — 66/66 通过
- [ ] 11.4 `cargo test --test bs_spec -- --nocapture` — 15/15 通过
- [ ] 11.5 `cargo test --test ep_full -- --nocapture` — 121/121 通过（约 38 秒）
- [ ] 11.6 `RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture` — 2828 基线不回归
- [ ] 11.7 `cargo tree` 确认无 `sasspile-macros`/`syn`/`quote`/`proc-macro2` 依赖
- [ ] 11.8 `codegraph sync` 更新代码导航索引
