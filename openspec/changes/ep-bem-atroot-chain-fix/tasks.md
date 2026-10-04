## 1. SPAN 插桩 — 确认根因证据链

- [x] 1.1 在 `mixin.rs::eval_at_root` 入口加 tracing span，记录 `incoming_sel`, `incoming_chain`, `parent_sel`, `wrapper_check result`
- [ ] 1.2 在 `rule.rs::nest_rule_in_children` 入口加 tracing span，记录 `parent`, `child_selector`, `already_has_prefix`, `combined`
- [ ] 1.3 在 `mixin.rs::exec_mixin` 展开 body 时记录 env.selector_chain 状态
- [x] 1.4 编译 checkbox.scss / dialog.scss / alert.scss，收集 trace，定位 `&` 链断裂位置

## 2. 修复 eval_at_root 的 parent_sel 构造逻辑

- [x] 2.1 修改 eval_rule 中 `&` 解析使用 chain → `env.get_selector_chain()` (rule.rs:411-419)
- [ ] 2.2 确保 `reset_selector` 调用不会清空已正确构建的 at-root parent_sel
- [ ] 2.3 验证 wrapper-skip 逻辑与新的 parent_sel 构造不冲突（`at_root_match` / `outer_only` 判断不受影响）

## 3. 加强 nest_rule_in_children 防重复

- [x] 3.1 descendant 前缀检测已存在于 nest_rule_in_children + push_atroot_direct
- [ ] 3.2 增加 compound + descendant 组合检测（parent 同时是 child 的前缀，无论是否有空格分隔符）
- [x] 3.3 already_has_prefix 检测已实现（nest_rule_in_children + push_atroot_direct child loop）

## 4. mixin.rs chain 传播修复

- [ ] 4.1 验证 `exec_mixin` 在展开含 `@at-root` 的 mixin body 时，env.selector_chain 正确传给 at-root handler
- [ ] 4.2 确保 b() → when() → e() 嵌套时，e() 展开的 @at-root 中 `&` 能展开为 `.el-block.is-state.el-block--mod` 完整形式
- [ ] 4.3 检查 `m()` modifier mixin 在嵌套上下文中，`$selector: &` 是否能正确捕获完整链

## 5. 验证修复效果

- [ ] 5.1 运行 `cargo test --test ep_normalized_test -- --nocapture` 验证 EP 一致性从 78/121 提升（目标 ≥ 93/121）
- [ ] 5.2 运行 `cargo test --test ep_full -- --nocapture` 确认 121/121 EP full 维持
- [ ] 5.3 运行 `cargo test` 全量确认核心测试 202/202 维持
- [ ] 5.4 sass-spec 全量运行，确认无回归（基线 7877/12133）
- [ ] 5.5 逐个验证修复的 EP 文件（checkbox, dialog, alert, carousel, check-tag, input-otp, radio-button, rate, table-v2 等），确认 CSS 语义与 EP dist 一致

## 6. 清理与收尾

- [ ] 6.1 移除临时 debug span，或降级为 trace! / debug!
- [ ] 6.2 更新 CHANGELOG.md Unreleased 节，记录修复内容和统计提升
- [ ] 6.3 更新 AGENTS.md EP normalized 基线数字
- [ ] 6.4 运行 `codegraph sync` 更新代码导航索引
- [ ] 6.5 等待用户确认后 commit + push
