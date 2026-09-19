## Context

当前 `core_functions/meta` 模块（src/eval/builtin/manual_dispatch.rs + src/eval/mixin.rs）存在三个不对称/遗漏：

1. **`get-function` 查找路径不完整**：无 module 参数时只查 local_functions，不查 namespace 模块或 builtins。而 `function-exists` 三条路径（local + namespace + builtin）都查。  
   代码差异：`function-exists`（lines 163-172）vs `get-function`（lines 309-326）。

2. **`meta.load-css` 裸名不可调用**：`eval_include` 只分派 `name == "meta.load-css"`，`@use 'sass:meta'` 后调用 `@include load-css(...)` 会走常规 mixin 查找失败。

3. **错误消息格式**：部分 arity 检查和类型错误的提示格式偏离 sass-spec 规范。

约束：
- 不能参照 dart-sass，所有权语义完全不同
- 禁止 `env.clone()`（除 `@content` 上下文快照）
- 所有测试代码放 tests/，src/ 保持纯生产代码
- 跨函数管道必须用 tracing span

## Goals / Non-Goals

**Goals:**
- `get-function(name)` 在无 module 时能正确找到 namespace 模块的函数
- `@use 'sass:meta'` 后 `@include load-css(...)` 可正常调用
- 错误消息格式对齐 sass-spec 规范
- meta 通过率从 76% 提升到 ~82%

**Non-Goals:**
- 不修改 Reactor 管线架构
- 不重构 Env/Scope 体系
- 不触及颜色系统

## Decisions

### Decision 1: get-function namespace fallback 复用 `call_function` 的命名空间查找逻辑

`call_function`（mixin.rs:239-251）已实现 namespace 模块 + builtins 的 fallback。  
`get-function` 应保持一致，在同一位置查找 namespace 函数并返回 `Value::FunctionRef`。

**替代方案 A**：在 `get-function` 中重复 namespace 遍历代码 → 拒绝，违反 DRY  
**替代方案 B**：提取公共查找函数 `find_function_in_env(name, env) -> Option<FunctionDef>` → 值得考虑，但改动面略大

→ 选择：在 get-function 中添加 namespace fallback（直接方案），与 `call_function` 对齐

### Decision 2: load-css 注册方案

`meta.load-css` 需要同时支持 `meta.load-css(...)` 和 `load-css(...)`（裸名）两种调用方式。

**替代方案 A**：在 `eval_include` 中增加 `name == "load-css"` 分支 → 可行，但硬编码  
**替代方案 B**：将 meta mixin 在 @use 注册时注入 local_mixins → 更优雅，但需改 @use 流程

→ 选择：方案 A（在 eval_include 增加 `load-css` 分派 arm），改动最小且不影响其他路径

### Decision 3: 错误消息格式对齐

根据 diag 输出的实际失败，逐个对齐：
- `get-function()` 无参数 → sass-spec 期望 `"Missing argument $name."`（已有）  
- `get-function(2px)` → 需确认实际 spec 期望消息格式
- `get-function("a", "b", "c", "d")` → arity 检查消息需对齐

→ 选择：写完代码后跑诊断 + 对比 expected error logs 再逐个修正

## Risks / Trade-offs

- **get-function 返回值一致性** → 找到 namespace 函数后，`FunctionRefData` 的 `module` 字段应填 `None`（全局名查找）还是填模块名？根据 spec，无 module 参数时应填 `None`
- **eval_include 中 load-css 检测冲突** → 若用户定义了名为 `load-css` 的 mixin，当前设计优先走 meta.load-css。sass-spec 中 sass:meta 的 mixin 优先级如何？需查阅确认（推测用户定义优先）
- **namespace 函数 is_builtin 过滤** → `call_function` 的 namespace 查找有 `!exports.is_builtin` 过滤（line 242），但 get-function 查找时不应过滤（应该能返回内建函数的 FunctionRef）→ **修正**：不过滤
