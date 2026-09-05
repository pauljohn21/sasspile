## Why

`@extend` 是 SCSS 的核心特性之一，sasspile 当前在 `directives/extend` 目录的 sass-spec 通过率仅为 29%（5 pass / 12 fail）。
根因分析发现：**最简单的 `d {@extend a}` 都无法工作**——extend 根本没有生效。
通过 tracing 证据链定位到 module scope 检查误判：非模块化编译时 `module` 为 `Some(base_path)` 但 `module_selectors` 为空，导致所有 extend 被跳过。
此外还发现多个算法缺口：逗号分隔多目标 `@extend .a, .b` 未拆分、复杂选择器/复合选择器的 extend 校验缺失、`:is()` 伪类内 extend 不传播等。

## What Changes

- 修复 module scope 检查：当 `module_selectors` 为空或 `module_path` 不在缓存中时，不跳过 extend（降级为全局匹配）
- 拆分 `@extend` 逗号分隔多目标：`@extend .a, .b` → 分别生成 `.a` 和 `.b` 两个 extend target
- 新增复杂选择器 extend 校验：`@extend a b` 报错 "complex selectors may not be extended"
- 新增复合选择器 extend 校验：`@extend a:hover` 报错 "compound selectors may no longer be extended"
- 新增空 `@extend` 校验：报错 "expected selector"
- 完善 `:is()`/`:where()` 伪类内的 extend 传播
- 重构 `extend_complex` 支持中间位置匹配（不仅仅是后缀匹配）
- 修复占位符选择器 `%foo` 在嵌套 extend 中的过滤逻辑
- 重构 `apply_extends` 中的 `fold` 去重为函数式 `fold` + `contains`

## Capabilities

### New Capabilities

- `extend-validation`: `@extend` 参数校验——复杂选择器、复合选择器、空选择器的错误检测

### Modified Capabilities

- `extend-cross-file`: 修复 module scope 检查逻辑——非模块化编译时降级为全局匹配，逗号分隔多目标拆分

## Impact

- `src/eval/extend.rs` — module scope 检查修复 + fold 去重重构
- `src/eval/mod.rs` — `eval_extend_node` 逗号分隔拆分
- `src/parse/at_rules.rs` — `parse_extend` 保持原始选择器文本（已有）
- `src/css/selector_ops.rs` — `extend_complex` 中间位置匹配
- `src/css/selector_ast.rs` — 无变更
- `tests/` — 新增 extend 相关测试用例
