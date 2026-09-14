## Why

`@import` 指令的变量作用域传播未完全实现——importing context 的变量对 imported file 不可见，且 `!default` 变量无法被 importing context 正确覆盖。这是 directives 目录中最大的失败来源（32/100 fails = 32%）。

## What Changes

- `@import "file"` 时，importing context 中已定义的变量对 imported file 可见
- `$var: value; @import "other";` 后，`other.scss` 中同名的 `!default` 变量被 importing context 的值覆盖
- 嵌套 `@import`（a @import b @import c）形成正确的作用域链
- `@import` 与 `@forward` 混合作用时的变量传播

## Capabilities

### New Capabilities

- `import-variable-visibility`: importing context 变量在 imported file 中可见
- `import-default-override`: importing context 中已定义变量覆盖 imported file 的 `!default` 值
- `import-nested-scope`: 嵌套 import 形成链式作用域传播

### Modified Capabilities

- 无（现有 spec 不变，只新增能力）

## Impact

- 受影响源文件：`src/eval/import.rs`、`src/eval/hoist.rs`、`src/eval/env_impl.rs`
- 受影响测试：`directives/import/configuration/*`（约 20 cases）、`directives/import/with*`（约 12 cases）
- API 兼容性：无 breaking change，只扩展行为
