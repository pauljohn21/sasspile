## 1. 诊断确认

- [ ] 1.1 运行 `cargo test --test failures_detail` 记录四类缺陷的具体 expected vs actual
- [ ] 1.2 确认 408 个 selector extend 失败的分类数量（no_op/format/complex_with/complex_without）

## 2. NO-OP 命名空间修复

- [ ] 2.1 在 `extend_selector` no-op 分支确保 `Namespace::Empty` 序列化保留
- [ ] 2.2 修复 `unify_extendee_list` 中合并过程丢失空命名空间的问题
- [ ] 2.3 验证 `extend/no_op/conflict/universal/*` case 输出正确的 `|*.c` 格式

## 3. Format 多余选择器修复

- [ ] 3.1 在 `build_extended_complex` 中增加"extender 必须真正出现"校验
- [ ] 3.2 修复 extend 后引入未预期组合（如 `d e`）的问题
- [ ] 3.3 验证 `extend/format/*` case 不再生成多余 selector

## 4. Complex 统合全排列修复

- [ ] 4.1 修改 extend 循环遍历统合结果（当前只处理单一统合）
- [ ] 4.2 为每种统合方式执行 extend 并合并去重
- [ ] 4.3 验证 `extend/complex/with_unification/*` 生成全部排列（2→3 种）
- [ ] 4.4 验证 `extend/complex/without_unification/*` 同样生成全部排列

## 5. Tail Combinator 重复修复

- [ ] 5.1 修复 tail combinator 分支中 remaining 与 extender tail 的合并逻辑
- [ ] 5.2 消除 suffix 连接时的重复 compound 附加
- [ ] 5.3 验证 `extend/complex/trailing_combinator/*` 无重复 `.d`

## 6. 验证

- [ ] 6.1 运行 `cargo test --test failures_json` 重新导出 JSON
- [ ] 6.2 对比前后 JSON，确认 selector 失败数 ≤30（90%+ 通过率）
- [ ] 6.3 运行核心测试 `cargo test --test compile_test` 确认无回归
- [ ] 6.4 验证 css 和 directives 中 selector 相关失败也减少
