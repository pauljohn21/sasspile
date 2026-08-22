## Why

`Env` 设计模仿 GC 语言的不可变模式（`&self -> Self` + `clone()`），而非 Rust 的 `&mut` 原地修改。`Rc` COW 只是缓解（O(n)→O(1) clone），但仍有引用计数开销和 `make_mut` 深拷贝问题。正确的 Rust 方向是用 `&mut` 原地修改，消除所有 clone。

同时 `@import` 没有 AST 缓存——同一文件被 import 多次时重复 read + lex + parse，是 `bs_spec` 慢的根因。

## What Changes

### 1. Env 改 `&mut` 原地修改

- `Env` 字段去掉所有 `Rc` 包裹，恢复为 `HashMap` / `Vec`
- 所有 `eval_xxx` 方法签名从 `&Env -> (Vec<CssNode>, Env)` 改为 `&mut Env -> Vec<CssNode>`
- `bind()` / `define_mixin()` 等方法从 `&self -> Self` 改为 `&mut self`
- `eval_nodes` 从 `try_fold` + clone 改为 `for` 循环 + `&mut`
- `load_import` 里的 `caller_env.clone()` 改为 `std::mem::take` 或 `&mut` 传递

### 2. @import AST 缓存

- 新增 `AstCache: HashMap<PathBuf, Rc<Ast>>` 字段到 `Env`
- `load_import` 和 `load_module` 先查缓存，命中跳过 read + lex + parse
- AST 是不可变的（解析后不修改），安全共享

### 3. 回退 Rc 改动

- `Env` 字段从 `Rc<HashMap>` 恢复为 `HashMap`
- `ModuleExports` 字段同样恢复
- 删除所有 `Rc::make_mut` 调用

## Capabilities

### New Capabilities

- `mut-env-ownership`: Env 用 `&mut` 原地修改，零 clone，Rust 原生所有权模式
- `import-ast-cache`: @import 文件 AST 缓存，避免重复 read + lex + parse

### Modified Capabilities

- `rc-env-cow`（回退）：Rc COW 方案废弃，改为 &mut 直接修改

## Impact

- **代码**：`src/eval/` 全部文件（mod.rs、mixin.rs、module.rs、rule.rs、control_flow.rs、value/mod.rs、module_helpers.rs、import.rs）
- **依赖**：无新增
- **性能**：ep_full 预期 ≤15 秒（消除 Rc 开销 + make_mut 深拷贝），bs_spec 预期 ≤10 秒（AST 缓存）
- **测试**：所有现有测试通过率不变
