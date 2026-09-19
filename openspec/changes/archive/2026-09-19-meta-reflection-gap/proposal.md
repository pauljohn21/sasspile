## Why

`core_functions/meta` 模块通过率仅 76%（372/489），存在 117 个失败。诊断发现三个系统性漏洞：(1) `get-function(name)` 缺少 namespace 查找 fallback，与 `function-exists` 不对称；(2) `meta.load-css` 在 `@use 'sass:meta'` 后不可裸名调用；(3) 错误消息格式偏离 sass-spec 规范。这些都是可定点修复的不对称/遗漏，不需要架构重构。

## What Changes

- **修复 get-function 命名空间查找**：在 `get-function(name)` 无 module 参数时，添加 namespace 模块的函数查找 fallback，与 `function-exists` 对齐
- **注册 load-css 裸名 mixin**：在 `eval_include` 中注册 `load-css` → `eval_meta_load_css` 的分派，使 `@use 'sass:meta'` 后可直接调用
- **对齐错误消息格式**：修正 `get-function`/`function-exists`/`module-functions` 等函数的 arity 检查和类型错误消息格式

## Capabilities

### New Capabilities

*无新增能力*

### Modified Capabilities

- `meta-reflection`: `get-function` 的查找路径现在包含 namespace 模块，`load-css` 可在 `@use 'sass:meta'` 后裸名调用

## Impact

- **受影响文件**：`src/eval/builtin/manual_dispatch.rs`（get-function 修复）、`src/eval/mixin.rs`（load-css 分派）
- **受影响测试**：`sass-spec/spec/core_functions/meta/` 目录下的失败 case
- **API 兼容性**：无 breaking change，所有修改为向后兼容的行为补全
- **预估收益**：+15~25 cases（meta 76% → ~82%）
