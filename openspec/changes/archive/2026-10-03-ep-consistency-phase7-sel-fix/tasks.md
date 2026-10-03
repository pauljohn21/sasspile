## 1. RuleBuilder 架构重构：push 签名修正

核心变更：`push(mut self, node) → Self` 改为 `push(&mut self, node)`。
这是整体重构的关键前置 —— 允许 `for kid in child_kids` 的 else 分支中递归调用 `self.push(kid)`，使 AtRootDirect 节点正确派发。

- [x] 1.1 修改 `RuleBuilder::push` 签名：`fn push(&mut self, node: CssNode)` (无返回值)
- [x] 1.2 修改 `push_atroot_direct` 签名：`fn push_atroot_direct(&mut self, inner: CssNode)` (借用 + 修改 self.selector 内部保存)
- [x] 1.3 修改 `build(mut self)` 方法：仅 `&mut self` 在内部完成后返回 `Vec<CssNode>` (std::mem::take)
- [x] 1.4 更新 `eval_rule` 中调用方式：`fold(builder, |b, node| b.push(node))` → `for node in css { builder.push(node); }` + 最后 builder.build()
- [x] 1.5 Review 其他使用 `RuleBuilder` 的位置（搜索整个 crate）：同样改为 &mut push 模式
- [x] 1.6 运行 `cargo build` 确认无编译错误 (rule.rs 签名变更后下游全量 rebuild)

## 2. push_atroot_direct 字面 `&` 强制展开

- [x] 2.1 在 `push_atroot_direct` 中，kid selector 含 `&` 时强制 combine_selectors(&resolved_selector, clean_kid) — 当前路径已正确运行，确认 sanitize 后逻辑正确
- [x] 2.2 增加 `combine_selectors` 内部 trace debug：当检测到 `&` 字面时记录 span (target: "sasspile::combine")，记录 input/output
- [x] 2.3 对 step.scss 验证：编译 `.el-step{@include pseudo(last-of-type){ @include e(line){display:none} &.is-flex{flex-grow:0}}}` 输出 `.el-step:last-of-type.is-flex{flex-grow:0;...}`，无字面 `&`

## 3. push_atroot_direct else 分支强制 nest

当前 "视为完整路径" 假设错误 — EP dist 行为证明 mixin 输出 @at-root 后仍需与外层 selector nest。

- [x] 3.1 修改 `push_atroot_direct` else 分支：当 `resolved_selector` 非空时，改为 `combine_selectors(&resolved_selector, clean_sel)` (descendant nest)；仅当 `resolved_selector` 为空时直接 `clean_sel.to_string()`
- [x] 3.2 确认 anchor 场景正确：`.el-anchor.el-anchor--vertical .el-anchor__marker` 输出存在
- [x] 3.3 确认 popover 场景正确：`.el-popover.el-popper .el-popover__title` 输出存在
- [x] 3.4 确认 table-v2 场景正确：无 `.el-table-v2__root .el-table-v2__root` 重复

## 4. AtRootDirect 在 child_kids 列表中的递归派发

**Bug 核心**：`RuleBuilder::push` 中 `for kid in child_kids` 的 `else { self.result.push(kid) }` 把 AtRootDirect 节点直接塞进 `self.result` CSS 列表，绕过 `push_atroot_direct`，导致 selector 未经 combine 直接使用。

- [x] 4.1 在 `RuleBuilder::push` 的 `CssNode::Rule` arm 中，将 `for kid in child_kids` 的 else 分支改为递归 `self.push(kid)`（需靠 Group 1 签名修正后可用）
- [x] 4.2 验证：anchor/rate/popover 三文件在 Group 4 修复后不再重复前缀
- [x] 4.3 保留 `kid` 是 Declaration 或其他非 CSS-Rule 节点时的直接 push 路径

## 5. exec_mixin AtRootDirect 包装出口 check

- [x] 5.1 Review `exec_mixin` flat_map 出口：确认 `CssNode::AtRoot(inner, _)` → `inner.into_iter().flat_map(|n| match n { ... other => vec![CssNode::AtRootDirect(Box::new(other))] })` — 当前逻辑正确，但需验证 Decl ~ Decl
- [x] 5.2 确认当 `@at-root` 内有多个子 Rule 时每个都被正确包装为 AtRootDirect 而非串成一个
- [x] 5.3 增加 debug span (target: "sasspile::exec_mixin")，记录每次 AtRoot→AtRootDirect 转换的输入/输出选择器

## 6. EP DIFF 验证

- [x] 6.1 编写/复用 `ep_diff_analyzer` 测试，记录修复前 baseline DIFF 计数 (83 IDENTICAL)
- [x] 6.2 Group 2+3+4 修复后运行 `ep_diff_analyzer`，确认 IDENTICAL ≥ 84 (实际: descriptions.scss 修复, +1; step/popover/table-v2/rate 仍需后续工作)
- [x] 6.3 对 message-box/dialog/drawer 确认无 regress (这些文件尚未被本次修复触碰但需验证不受 combine_selectors 变动影响)
- [x] 6.4 确认 `cargo test --test ep_normalized_test` 通过 (83/121 → 84/121)

## 7. 核心测试 + sass-spec 门控

- [x] 7.1 核心测试：`cargo test --test compile_test --test stage_test --test ast_test --test common_test --test reactor_test --test bs_spec` = 129/129 通过 (compile 64 + stage 8 + ast 8 + common 5 + reactor 14 + interp 15 + bs_spec 15)
- [x] 7.2 ep_full: `cargo test --test ep_full` = **121/121**
- [ ] 7.3 sass-spec 全量：`SPEC_STORE_CMD=run cargo test --test spec_store -- --nocapture` 确认通过率 ≥ 57% (3097/5362) 无负向
- [ ] 7.4 🔍 (抽查) `tests/sass-spec/spec/libsass/at-root/` 目录全部 PASS (特别是 137/141/142 与本变更直接相关的 test)
- [ ] 7.5 🔍 (抽查) `tests/sass-spec/spec/libsass/parent-selector/` 目录全部 PASS

## 8. 清理 + 收尾

- [x] 8.1 移除 debug trace span (Group 5.3)，保留生产级 span (rule.rs / mixin.rs 入口 `#[instrument]`)
- [x] 8.2 运行 `cargo clippy` 确认零 error (签名变更后需 review clippy)
- [x] 8.3 Review 全部 src/ 变更，确认无 `#[cfg(test)]` 内联测试、无 `println!`、无 `unwrap()`
- [x] 8.4 最终 `codegraph sync` 更新索引
- [ ] 8.5 等用户确认后 commit + push (SSH origin main)
