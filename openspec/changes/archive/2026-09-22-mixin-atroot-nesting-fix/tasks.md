## 1. RuleBuilder build() 嵌套 @at-root 结构保留

- [x] 1.1 修改 `src/eval/rule.rs` build() 函数：当 `self.selector` 含 `&` 且 `root_nodes` 为空时，调用 `wrap_children_under_parent()` 将子节点包装进父 Rule
- [x] 1.2 添加 `wrap_children_under_parent` 方法：安全地将 self.result 中首个匹配 self.selector 的 Rule 以外的兄弟节点收集到该 Rule 的 children
- [x] 1.3 Declaration 节点保持原样（build 逻辑不变）

## 2. 序列化器 AtRootDirect 修复

- [x] 2.1 修复 `src/css/serialize_write.rs` `write_node_expanded` 中 AtRootDirect 处理：将 `serialize_expanded` 结果写入 `buf`（之前被丢弃导致输出缺失）

## 3. 测试验证

- [x] 3.1 运行 `cargo test --test ep_normalized_test -- --nocapture`
- [x] 3.2 验证其他 EP 文件无新增 DIFF：20 DIFF 保持不变，无回归
- [x] 3.3 运行 `cargo test --test ep_full -- --nocapture` — 121/121 通过
- [x] 3.4 运行核心测试 — compile 64 + reactor 14 + stage 8 + ast 8 + common 5 + interp 15 + bs_spec 15 = 129/129 全通过
- [ ] 3.5 运行 `SPEC_STORE_CMD=run cargo test --test spec_store -- --nocapture` 确认 sass-spec 无负向变化（待执行）

## 4. 边界情况处理

- [ ] 4.1 验证 `@at-root` 内部同时包含 Declaration 和 Rule 时的正确处理
- [ ] 4.2 验证空 `@content`（mixin 无内容）不产生多余空规则
- [ ] 4.3 验证深层嵌套（b→m→e 三层）选择器组合正确性
- [ ] 4.4 验证 `@each` 循环中多次 `@include` 的输出顺序与源码一致

## 5. 文档与归档

- [ ] 5.1 更新 openspec proposal.md 中的 Impact 部分（反映实际修改范围）
- [ ] 5.2 更新 AGENTS.md 中关于 AtRoot 处理的描述
- [ ] 5.3 提交 commit 并等待用户确认后推送
- [ ] 5.4 归档 `ep-consistency-phase7-all-diff-fix` 中 task 2 相关内容

## 实际修改总结

### 根因分析
1. **序列化器 bug**：`write_node_expanded` 中 `AtRootDirect` 分支调用了 `serialize_expanded` 但未将结果写入 `buf`，导致 AtRootDirect 内容丢失
2. **build() 逻辑缺陷**：当 `self.selector` 含 `&` 且 `root_nodes` 为空时，`self.result` 中的子节点未被包装进父 Rule

### 修改文件
- `src/eval/rule.rs`：添加 `wrap_children_under_parent` 方法，修改 `build()` 调用之
- `src/css/serialize_write.rs`：修复 AtRootDirect 序列化

### 未解决问题
`descriptions.scss` DIFF 主要因 `Incompatible units 1px and 1%` 错误导致输出不完整，非嵌套 @at-root 问题
