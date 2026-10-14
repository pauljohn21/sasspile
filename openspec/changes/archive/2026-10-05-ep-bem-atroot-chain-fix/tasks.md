# Tasks

## 1. 修复 eval_at_root outer_only 误 unwrap

- [x] 1.1 在 `eval_at_root` 的 `outer_only` 分支前添加子 Rule 检测：`let has_rule_children = first_body.iter().any(|n| matches!(n, Node::Rule { .. }));`，当 `outer_only && has_rule_children` 时 fall-through 到 `Self::eval_nodes(body, env)`
- [x] 1.2 用 `/tmp/test_dialog_full.scss` 验证：`cargo run -- /tmp/test_dialog_full.scss` 应输出 `.el-dialog.is-draggable .el-dialog__header { cursor: move; user-select: none; }`

## 2. 修复 eval_at_root chain 重置

- [x] 2.1 在 `eval_at_root` 中将 `let env = match selector { Some(_) => env.reset_selector(...), None => env };` 改为无条件调用 `env.reset_selector(parent_sel.clone())`
- [x] 2.2 用 `/tmp/test_collapse.scss` 验证：`cargo run -- /tmp/test_collapse.scss` 应输出 `.el-collapse-item__arrow.is-active { transform: rotate(90deg); }`（无 `.el-collapse-item__header` 前缀）

## 3. 回归测试

- [x] 3.1 运行 `cargo test --test compile_test` + `cargo test --test ep_full -- --nocapture` 确认核心测试 + EP 基线不退化
- [x] 3.2 运行 `cargo test --test selector_combine_test`（如已存在）确认选择器组合逻辑无退化

## 4. EP 分类验证

- [x] 4.1 运行 `cargo test --test ep_classify_test -- --nocapture` 统计 sasspile-bug 数量，修复前 37 → 修复后 27 (-10)
- [x] 4.2 确认无新 bug（总数 = 83 一致 + 7 autoprefixer + 27 bug = 121/121，测试通过）
