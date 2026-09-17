## Why

代码库中所有"业务逻辑"函数（evaluate、parse、serialize 的内部 helper）至今仍在用 GC 语言思维写 Rust——大量 `while i < len` 手动索引循环、`for x in items { out.push(f(x)) }` 累积、`Rc<RefCell<HashMap>>` + `borrow_mut()` 模拟共享可变状态、`subscribe(|ch| refcell.borrow_mut().push(ch))` 手动收集流值。

这违反了项目的根本约束：**Rust 所有权 + rxrust 算子驱动**。rxrust 本身就是所有权系统的流体表达：
- `scan_map(State, reducer)` = Move（消费旧状态,产出新状态）
- `flat_map(expand)` = 所有权转移展开
- `.last()` / `.collect()` = Consume 消费整个流
- `.tap()` = Borrow 只读副作用

当前代码"pipeline 边界套一层 rxrust,里面全是 GC 思维"的模式,导致：
1. 维护者难以追踪状态变更点（散布在 `borrow_mut()` 调用中）
2. 借用错误频出——说明设计违反所有权而非编译器太严
3. 无法利用 rxrust 的惰性求值、背压、取消等特性
4. 代码无法线性阅读——状态隐藏在 `RefCell` 后面

## What Changes

按优先级逐层重构,从外到内替换违反所有权的写法:

### P0 — `lib.rs` 终点收集法 (2 处)

**Before**: `subscribe(move |ch| refcell.borrow_mut().push(ch))` 手动偷取
**After**: `pipeline.build().last().subscribe(|result| ...)` 消费整个流

### P1 — Helper 函数内部循环 → 迭代器链 / scan

替换以下函数中的命令式循环:
- `evaluate_dst/mod.rs`: `substitute_vars` (125行), `resolve_vars_only` (55行), `split_args` (32行)
- `evaluate_dst/builtins.rs`: `split_list` (50行), `arg_to_map` (13行)
- `evaluate_dst/eval_ctx.rs`: `evaluate_for`, `evaluate_if`, `evaluate_mixin_call`, `builtin_get_css_var`
- `parse_dst/declarations.rs`: `merge_negative_numbers`, `parse_declarations`, `parse_block`, `try_flush_variable`
- `parse_dst/directives.rs`: `parse_arg_list_at`, `parse_param_list`, `parse_arg_list`
- `parse_dst/mod.rs`: `parse_rule_body`

模式: `while i < len { out.push(items[i]); i += 1 }` → `iter.map(f).collect()`
或: 嵌套状态用 `scan_map(State::new(), reducer).last()`

### P2 — `CompilerContext` 内 `borrow_mut` 消除

**问题**: `CompilerContext` 承载了所有编译状态（变量、mixin、缓存、路径栈），通过 `Rc<RefCell<_>>` 共享。

**方案（渐进式,不破坏现有 API）**:
- 保持 `CompilerContext` 作为"配置/句柄"结构
- 流状态（变量、mixin 注册）随 `scan_map` reducer 传递而不是存入 RefCell
- `borrow()` 只读访问保留（模块缓存查找等语义上确实是共享只读）
- `borrow_mut()` 用于 hot-path 变量注册,改为 reducer 内部消费-替换模式

### P3 — 副作用隔离到 `.tap()`

**现状**: 某些 `match` 分支内有 `tracing::debug/warn` 调用
**改为**: 在上游 pipe 中加 `.tap(|item| tracing!(...))`，reducer 闭包纯净

### P4 — `evaluate_for` / `evaluate_if` / `evaluate_mixin_call` → flat_map

**现状**: 这些函数内部用 `for node in body { out.extend(evaluate_node(ctx, node)?); }` 手动展平
**改为**: 在 `evaluate_node` 的 `match` 分支中返回 `Local::from_iter(expand)`，让外层 pipe 的 `flat_map` 统一处理

## Capabilities

### New Capabilities
- `ownership-rxrust-core`: 所有 helper 函数严格遵循所有权+算子范式,无 `Rc<RefCell>` / `while` / `subscribe+push` / `for+extend`

### Modified Capabilities
- `mixin-variable-scoping`: 局部作用域消除 `borrow_mut` + `for+extend`，改用 `flat_map` 展开
- `builtin-polish`: 所有 builtin helper（split_list, arg_to_map 等）改为迭代器链

## Impact

- **受影响文件**: `src/lib.rs`, `src/evaluate_dst/mod.rs`, `src/evaluate_dst/builtins.rs`, `src/evaluate_dst/eval_ctx.rs`, `src/evaluate_dst/builtins_math.rs`, `src/parse_dst/mod.rs`, `src/parse_dst/declarations.rs`, `src/parse_dst/directives.rs`
- **不破坏**: 所有重构是纯内部实现替换,公开 API `compile()` / `compile_at()` / `compile_file()` 签名不变
- **不破坏**: Bootstrap / Element Plus 编译路径不变,tap 观测保留
- **测试**: 运行 `cargo test` + `cargo clippy` 全量验证
- **风险**: 低。每次只改一个函数,改完跑全部测试
