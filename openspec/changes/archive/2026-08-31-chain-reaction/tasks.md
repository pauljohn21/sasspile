## 1. eval_nodes try_fold 改造

- [x] 1.1 将 `src/eval/mod.rs` 的 `eval_nodes` 从 for 循环改为 `try_fold((Vec::new(), env), |(mut css, env), node| { ... })`
- [x] 1.2 验证：`cargo test --test compile_test` + `cargo test --test stage_test` 全通过

## 2. control_flow try_fold 改造

- [x] 2.1 将 `src/eval/control_flow.rs` 的 `eval_for` 从 while 循环改为 range + `try_fold`（处理正反向 step）
- [x] 2.2 将 `eval_each` 从 for 循环改为 `items.iter().try_fold(...)`
- [x] 2.3 `eval_while` 保留 loop 结构，但消除 `let mut css`（改用迭代内直接累积的模式）
- [x] 2.4 验证：`cargo test --test compile_test` 全通过（重点跑 @for/@each/@while 相关测试）

## 3. hoist_css_imports partition 改造

- [x] 3.1 将 `src/eval/mod.rs` 的 `hoist_css_imports` 从 for 循环改为 `into_iter().map(递归处理).collect()` + `partition`
- [x] 3.2 验证：`cargo test --test compile_test`（重点跑 @import 提升相关测试）

## 4. eval_rule RuleBuilder + fold 改造

- [x] 4.1 在 `src/eval/rule.rs` 创建 `struct RuleBuilder { result, current_decls, root_nodes }` + `new()` + `push(node)` + `build()` 方法
- [x] 4.2 将 `eval_rule` 的 for 循环改为 `css.into_iter().fold(RuleBuilder::new(), |b, n| b.push(n)).build()`
- [x] 4.3 将 `nest_rule_in_children` 同样改为 `fold` 或 `partition` 模式
- [x] 4.4 验证：`cargo test --test compile_test` + `cargo test --test stage_test` 全通过

## 5. CSS 序列化器链式化

- [x] 5.1 将 `src/css/mod.rs` 的 `flatten_nodes` 从递归 for 改为 `iter().flat_map(|n| ...).collect()`
- [x] 5.2 将 `merge_at_rules` 从 for 循环改为 `fold`（需查看前一个结果，fold 合适）
- [x] 5.3 验证：`cargo test --test compile_test` 全通过

## 6. Evaluated::serialize 签名改造

- [x] 6.1 将 `src/stage/evaluated.rs` 的 `serialize` 从 `&self` 改为 `self`
- [x] 6.2 检查 `src/lib.rs` 的 `compile` / `compile_file` / `compile_file_with_load_paths` 是否需要适配（大概率不用，已链式）
- [x] 6.3 验证：`cargo test --test stage_test` + `cargo test --test compile_test` 全通过

## 7. 全量验证

- [x] 7.1 运行 `cargo test --test compile_test`（43 个）
- [x] 7.2 运行 `cargo test --test stage_test`（10 个）
- [x] 7.3 运行 `cargo test --test ast_test`（8 个）
- [x] 7.4 运行 `cargo test --test common_test`（5 个）
- [x] 7.5 运行 `cargo test --test bs_spec -- --nocapture`（15 个）
- [x] 7.6 运行 `cargo test --test ep_full -- --nocapture`（121 个）
- [x] 7.7 运行 `RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture`（sass-spec 基线 2828/5362 不变）
- [x] 7.8 `cargo clippy --workspace` 无新增 warning
- [x] 7.9 `codegraph sync` 更新索引
