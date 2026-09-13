## Why

sasspile 在设计初始参照了 dart-sass（被 AGENTS.md 明确禁止），导致架构中残留了 SCSS/CSS 双模式思维。同时，测试体系对 sass-spec 的核心语义——"input 永远 SCSS，output 永远 CSS"——缺乏形式化约束，导致部分路径将 `.css` 文件错误地当作 SCSS 解析。本次变更旨在：(1) 消除双模式残留，确保编译器严格遵循 SCSS→CSS 单向语义；(2) 重构测试体系，形式化 sass-spec 的 input/output/asset 文件角色。

## What Changes

- **修复 `eval_import` 对 `.css` 文件的短路逻辑**：确保 `@import "foo.css"` 解析器不走 `load_import` 管线
- **增加 `@use`/`.css` 的明确错误**：`@use "foo.css"` 应报错 "CSS files can't be @used"
- **合并 `ScssEvaluator` 到 `Evaluator`**：消除暗示双模式的残留命名（`ScssEvaluator` 仅作为内部转发层存在）
- **测试框架文件角色分类**：在 `hrx_support.rs` 中区分 input/output/asset/css-asset 文件，output.css 不写入 VFS，避免误引用
- **增加 sass-spec 语义验证测试**：确保 `output.css` 不会被解析，`@use "foo.css"` 正确报错

## Capabilities

### New Capabilities

- **`css-import-semantics`**: 形式化 SCSS 对 `.css` 文件的 import 处理规则——`@import "x.css"` 输出 `@import url("x.css")`，`@use "x.css"` 报错，`@forward "x.css"` 报错

- **`test-file-roles`**: 测试 HRX 解析时区分文件角色——input（编译入口）、output（对比目标，不写入 VFS）、scss-asset（模块部分文件）、css-asset（仅可被 @import 引用）

- **`evaluator-unify`**: 消除 `ScssEvaluator` 类型，统一为 `Evaluator`，消除 SCSS/CSS 对偶暗示

### Modified Capabilities

- 无修改已有 spec 的需求（现有 spec 属于已归档的历史 delta spec，本次创建新 spec）

## Impact

| 组件 | 影响 |
|------|------|
| `src/eval/import.rs` | 修复 `.css` 短路逻辑，确保 @import "x.css" 不走 load_import |
| `src/eval/module.rs` | `load_module` 拒绝 `.css` URL 并返回明确错误 |
| `src/eval/scss_evaluator.rs` | 删除，合并入 `eval/mod.rs` |
| `tests/hrx_support.rs` | 区分 output.css 与其他文件角色，output 不写入 VFS |
| `tests/compile_test.rs` | 添加 CSS 文件 role 测试、@use .css 错误测试 |
| `src/lib.rs` | 移除 `scss_evaluator` 重导出 |
