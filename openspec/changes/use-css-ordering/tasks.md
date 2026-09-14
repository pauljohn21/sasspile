## 1. CSS 节点占位符

- [ ] 1.1 在 `CssNode` 枚举中添加 `PendingImport` 变体（或复用现有结构）
- [ ] 1.2 `parse_import`/`parse_use` 阶段生成占位节点
- [ ] 1.3 占位节点在 serialize 阶段正确输出（eval 后已被替换）

## 2. @import 嵌套展开

- [ ] 2.1 修改 `eval_import` 返回 `Vec<CssNode>` 而非修改 env in-place
- [ ] 2.2 在 `eval_rule` 中处理嵌套 `@import`：将返回节点注入到当前 rule 的 children
- [ ] 2.3 验证 `a { @import "other" }` 输出正确包含 other.scss 的内容

## 3. 输出排序保持

- [ ] 3.1 修改 `eval_hoist` 不强制前置 @use/@import 的 CSS 输出
- [ ] 3.2 CSS at-rule（@import/@charset）保持在原始 AST 位置
- [ ] 3.3 多个 at-rule 的相对顺序不变

## 4. 验证

- [ ] 4.1 运行核心测试 202/202 无回归
- [ ] 4.2 运行 `directives/use/css/order/*` 验证修复
- [ ] 4.3 运行 `directives/import/css*` 验证修复
- [ ] 4.4 全量 sass-spec 无其他目录回归
