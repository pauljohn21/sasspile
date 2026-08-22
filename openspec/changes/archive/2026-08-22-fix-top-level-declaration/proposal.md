## Why

sasspile 当前无法检测 `@import` 导入文件中出现在顶层的 CSS 声明（非规则体内），导致 `sass-spec` 的 `directives/import/error/top_level_declaration` 测试组（2 个 expected_error）失败。这是 directives-100 change 中任务 1.13 尚未实现的部分。

## What Changes

- 在求值器层面对 `Node::Decl` 增加顶层声明检测：当 `env.current_selector` 为 `None` 且非 plain CSS 模式时，报错 `expected "{".`
- 在 `eval_include` 返回后检查 mixin 产生的 CSS 输出：当调用上下文 `env.current_selector` 为 `None` 且返回的 CSS 包含 `CssNode::Declaration` 时，报错 `Declarations may only be used within style rules.`
- 上述检测仅针对 SCSS 模式，plain CSS 模式不受影响

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `error-detection-coverage`: 增加顶层声明错误检测能力——当 CSS 声明（`property: value`）出现在文件顶层而非规则体内时，以及当 `@include` 在顶层调用 mixin 且 mixin 产生 CSS 声明时，系统 SHALL 报错

## Impact

- **`src/eval/mod.rs`**：`eval_node` 的 `Node::Decl` 分支增加 `current_selector` 检查
- **`src/eval/mixin.rs`**：`eval_include` 或 `exec_mixin` 返回后增加 Declaration 输出检查
- **风险**：需确保不影响 `@at-root` 等合法顶层输出场景，以及 plain CSS 模式下顶层声明合法的行为
