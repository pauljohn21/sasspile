## 1. 伪元素格式规范化（pseudo-element-format）

- [x] 1.1 分析 :before/:before 差异的代码路径：在 extend.rs transform_nodes 和 serialize 层加 span，定位 `: before` 空格来源
- [x] 1.2 在 CssSerializer 层实现伪元素输出规范化（单/双冒号格式控制）
- [x] 1.3 修复影响文件：date-picker-panel.scss, index.scss, popper.scss, table.scss（DIFF 消除验证）
- [ ] 1.4 运行 ep_normalized_test 验证 +10 文件 IDENTICAL，确认无 sass-spec 回归

## 2. Placeholder Extend 嵌套层级修复（placeholder-extend-nested）

- [ ] 2.1 在 eval/extend.rs 中实现嵌套规则内的 extend 层级隔离（增量快照 + 独立应用）
- [ ] 2.2 修复 descriptions.scss, form-item.scss, input-number.scss, checkbox.scss, checkbox-button.scss 的 DIFF
- [ ] 2.3 修复 carousel.scss, cascader.scss, image-viewer.scss, image.scss 的嵌套占位符 DIFF
- [ ] 2.4 修复 button.scss, descriptions-item.scss, dropdown.scss, empty.scss 的 extend 相关 DIFF
- [ ] 2.5 运行 ep_normalized_test 验证 +15 文件 IDENTICAL

## 3. @keyframes 内容求值修复（keyframes-content）

- [ ] 3.1 在 eval_at_rule 中为 @keyframes 子节点添加 in_keyframes 上下文标记
- [ ] 3.2 抑制 in_keyframes 上下文中的 @at-root 提升逻辑
- [ ] 3.3 修复 dialog.scss, drawer.scss 的 @keyframes 内部 @include/@at-root 顺序 DIFF
- [ ] 3.4 确保空 keyframe 块（100% {}）输出格式正确
- [ ] 3.5 运行 ep_normalized_test 验证 +3 文件 IDENTICAL

## 4. 模块变量解析增强（module-var-resolution）

- [ ] 4.1 用 CodeGraph 定位 descriptions-item.scss, carousel.scss, cascader.scss 的变量不可见场景
- [ ] 4.2 在 @use 模块变量绑定时增加"提升回溯"到 mixin include 入口作用域
- [ ] 4.3 修复 anchor.scss, base.scss 的变量/模块引用 DIFF
- [ ] 4.4 修复 input-otp.scss 的模块变量 DIFF
- [ ] 4.5 运行 ep_normalized_test 验证 +8 文件 IDENTICAL

## 5. 选择器组合并修复（selector-group-merge）

- [ ] 5.1 分析 dart-sass 对分组 extend 的选择器合并语义（逗号分隔选择器的声明合并）
- [ ] 5.2 在 transform_nodes 中实现：同一 %placeholder 被多个选择器 extend 时生成组合选择器组
- [ ] 5.3 在 CssSerializer 层确保合并后的输出格式与 EP 一致
- [ ] 5.4 修复剩余 files（可能涉及 color-picker.scss, color-picker-panel.scss 等）的 DIFF
- [ ] 5.5 运行 ep_normalized_test 验证 +8 文件 IDENTICAL

## 6. 最终验证与清理

- [ ] 6.1 全量 ep_normalized_test 通过：121/121 IDENTICAL
- [ ] 6.2 全量 ep_full 通过（无 panic/hang）
- [ ] 6.3 sass-spec 全量回归测试（SPEC_STORE_CMD=run）确认无负向变化
- [ ] 6.4 核心测试 202/202 通过
- [ ] 6.5 归档 ep-consistency-phase7-all-diff-fix 到 archive
