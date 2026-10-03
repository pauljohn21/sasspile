## Why

EP 一致性停在 73/121 (60.3%)——Phase 5 (AtRootDirect) 和 Phase 6 (@extend %placeholder) 消除了结构性差异后，剩余 44 文件 DIFF 由多种次级因素导致。这些差异分散在选择器分组、变量解析、伪元素格式、keyframes 结构等方面，需要逐一分类攻克以实现 100% 目标。

## What Changes

按差异根因分 5 个 capability 修复：

1. **placeholder-extend-nested** — 复杂 placeholder extend 模式（嵌套规则内的 extend、多个 placeholder 交互、extendee 是复合选择器链）
2. **module-var-resolution** — @use 模块变量/Mixin 在特定上下文不可见或求值错误
3. **pseudo-element-format** — `:before`/`:after`/`:hover` 等伪类输出格式规范化（空格、`::` vs `:`）
4. **keyframes-content** — @keyframes 内部 @include/@extend 的求值顺序和输出格式
5. **selector-group-merge** — 选择器分组/合并语义与 dart-sass 一致（逗号分隔选择器的声明合并）

## Capabilities

### New Capabilities

- `placeholder-extend-nested`: 复杂 %placeholder extend 场景的完整支持
- `pseudo-element-format`: 伪元素/伪类输出格式规范化
- `keyframes-content`: @keyframes 内部 mixin/extend 求值

### Modified Capabilities

- `module-var-resolution`: 改进 @use 模块在嵌套/循环/条件上下文中的变量可见性
- `selector-group-merge`: 扩展选择器分组逻辑以匹配 dart-sass 语义

## Impact

- **代码文件**: src/eval/extend.rs, src/eval/mixin.rs, src/eval/rule.rs, src/eval/selector.rs 等
- **测试**: tests/ep_normalized_test.rs（基线 73 → 目标 121）
- **风险**: 伪元素格式修复可能影响 sass-spec 通过率（需验证）
- **工作量**: 约 44 文件 × 平均 2 种差异 = ~88 个独立修复点
