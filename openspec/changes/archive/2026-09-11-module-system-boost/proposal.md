## Why

模块系统（`@use` / `@forward` / `@import` 交互）是 sass-spec 中通过率最低的领域之一：`directives/use` 88.4%、`directives/forward` 93.1%、`directives/at_root` 92.6%、`core_functions/modules` 51.6%。这 4 个目录共 63 个失败 case，涉及转发前缀处理、CSS @import 提升、@import 优先级规则、@extend 跨模块等多个根因。通过系统性修复，预计可将这些目录推至接近 100%，整体 sass-spec 通过率提升约 0.4-0.5%。

## What Changes

- **修复 `@forward as prefix-*` 前缀分隔符**：`bind_exports` 中 `fmt_key` 增加 `-` 分隔符（当前 `d` + `c` → `dc`，应为 `d-c`）
- **实现 CSS @import 从 @use'd 模块提升**：`ModuleExports` 新增 `css_imports` 字段，eval_use 时累积，最终 evaluate 时提升到输出顶部
- **实现 @import 转发优先级语义**：被 @import 的文件中 @forward 转发的成员优先级高于导入文件本地定义
- **增强 @extend 跨模块**：支持菱形依赖合并、伪选择器内 @extend 嵌套
- **修复嵌套 @import 上下文**：CSS rule 内 @import 的文件中 @use 应被允许
- **修复加载解析边界**：.sass 扩展名、index/sass 索引、.sass/.css 优先级

## Capabilities

### New Capabilities

- `forward-prefix-separator`: @forward as prefix-* 前缀正确插入连字符分隔符
- `css-import-hoisting-from-use`: @use'd 模块内的 CSS @import 提升到最终输出顶部
- `import-forward-precedence`: @import 的文件中 forwarded 成员优先级高于本地定义
- `extend-across-modules`: @extend 穿越模块边界（菱形依赖、伪选择器）
- `nested-import-context`: CSS rule 内 @import 的文件中允许 @use

### Modified Capabilities

- 无（当前 openspec/specs/ 中无对应 spec）

## Impact

- **影响文件**：
  - `src/eval/module_helpers.rs` — `bind_exports` 的 `fmt_key` 修复
  - `src/eval/env.rs` — `ModuleExports` 新增 `css_imports` 字段
  - `src/eval/module.rs` — `eval_use` 累积 CSS imports、`load_import` 应用转发优先级
  - `src/eval/import.rs` — 嵌套 @import 上下文重置
  - `src/eval/extend.rs` — 伪选择器内 @extend 处理
  - `src/eval/forward.rs` — `eval_forward` 前缀传递
  - `src/eval/file_resolver.rs` — 候选文件排序
- **影响测试**：63 个失败 case（use 31 + forward 15 + at_root 2 + core_functions/modules 15）
- **无 breaking change**：所有修复均为正确性改进，不改变已有通过 case 的行为
