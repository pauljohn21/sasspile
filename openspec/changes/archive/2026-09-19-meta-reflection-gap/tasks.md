## 1. get-function namespace fallback

- [x] 1.1 在 `src/eval/builtin/manual_dispatch.rs` 的 `get-function` 分支中，在 local scope 查找失败后添加 namespace 模块函数查找 fallback（与 `function-exists` 对齐）
- [x] 1.2 确保 namespace 查找不跳过 `is_builtin` 模块（get-function 应能返回内建函数的 FunctionRef）
- [x] 1.3 确保找到 namespace 函数后，`FunctionRefData` 的 `module` 字段设为 `None`（无 module 参数表示全局查找）

## 2. load-css 裸名 mixin 注册

- [x] 2.1 在 `src/eval/mixin.rs` 的 `eval_include` 中添加 `name == "load-css"` 分派 arm，调用 `eval_meta_load_css`
- [x] 2.2 确认当 `@use 'sass:meta'` 后，`@include load-css(...)` 能正确路由到 `eval_meta_load_css`

## 3. 错误消息格式对齐

- [x] 3.1 确认 `get-function(2px)` 产生的错误消息完全匹配 sass-spec 期望格式
- [x] 3.2 确认 `get-function("foo", 1)` 的 `$module: 非字符串` 错误消息格式正确
- [x] 3.3 确认 `get-function("a", "b", "c", "d")` 的 arity 超限消息格式正确
- [x] 3.4 确认 `get-function()` 无参数时消息为 "Missing argument $name."

## 4. 验证

- [x] 4.1 运行 `cargo test --test diagnostic_runner diag_meta -- --nocapture` 确认所有 meta 失败已修复或已分类
- [x] 4.2 运行 `cargo test` 确保核心测试不回归（keyframes 2 fail 为 pre-existing）
- [x] 4.3 meta-specific 错误全部消除（load-css × 2, undefined function c/d）：增量确认
