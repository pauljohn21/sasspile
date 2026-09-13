## Why

sasspile 当前是纯同步编译器，但存在以下问题：

1. **模块加载串行阻塞**：`load_module` / `load_import` 使用 `std::fs::read_to_string` 同步读文件，多模块场景下 IO 串行浪费时间
2. **Reactor 携带大量死代码**：`ModuleCacheEntry`、`IoRecord`、`Warning`、`ReactorIO` trait+impls、`io_log`、`warnings`、`modules` 等字段定义了但从未使用
3. **代码风格不完全函数式**：仍有 `for+push`、`if-else` 链、`match Err(e) => return` 等命令式模式
4. **Parser 使用了 tokio-stream 但无异步运行时**：`ParseStream` 实现了 `tokio_stream::Stream` 却通过 `futures::executor::block_on_stream` 同步驱动，引入 async 原语但无实际收益

## What Changes

### 内部 Async（不改变对外 API）

- **模块加载层 async 化**：`load_module` / `load_import` / `load_import` 改为 `async fn`，内部使用 `tokio::fs::read_to_string` 异步读文件
- **Evaluator::evaluate_with_env 改为 async**：因为需 await 模块加载
- **Reactor::evaluate() 改为 async**：作为内部 async 入口
- **公开 API 保持同步**：`compile()` / `compile_file()` 等函数内部通过全局 tokio runtime 的 `block_on` 桥接
- **`tokio-stream` + `futures` 依赖替换为 `tokio`**：async 原生支持，不再需要兼容层

### 删除死代码

- `ModuleCacheEntry` 结构体（`reactor_types.rs`）
- `IoRecord` 结构体（`reactor_types.rs`）
- `Warning` 结构体（`reactor_types.rs`）
- `ReactorIO` trait + `DefaultReactorIO` + `MockReactorIO`（`reactor_types.rs` + `reactor_test.rs`）
- `Reactor.modules` 字段
- `Reactor.io_log` 字段
- `Reactor.warnings` 字段
- `Reactor.io` 字段
- `Reactor.env` 字段（仅设 `Some(Env::default())`，无实际作用）
- `ModuleExports::css_imports` 的 `#[allow(dead_code)]` 标注
- `ReactorTrace::advance()` 中丢失 `entered_at` 的 bug

### 代码风格优化

- `for + push` 模式 → `map/filter/collect` 或 `try_fold`
- `if-else` 链 (≥3 分支) → `match`
- `match Err(e) => return Err(e)` → `?`
- `bool` 判等 + 早返回 → `matches!` / `if let`

## Capabilities

### New Capabilities

- **`internal-async-module-loading`**：模块加载层使用 `tokio::fs` 异步读文件，内部 `tokio::task::spawn_blocking` 执行 lex+parse 密集 CPU 任务，允许并行模块编译
- **`sync-public-api`**：公开 API 函数内部通过全局 tokio runtime `block_on` 桥接，调用方无需任何改动

### Modified Capabilities

- **`reactor-pipeline`**：移除死字段后 Reactor 结构从 14 字段精简到 8 字段，`evaluate()` 变为 async

## Impact

| 组件 | 影响 |
|------|------|
| `Cargo.toml` | `tokio-stream` + `futures` → `tokio = { version = "1", features = ["full"] }` |
| `src/lib.rs` | 添加全局 tokio runtime，`compile()` 系列函数内部 `block_on` |
| `src/main.rs` | 文件读取改用 `tokio::fs`（可选优化，保留同步也可） |
| `src/parse/mod.rs` | `ParseStream` 回归标准 `Iterator`，删除 `tokio_stream::Stream` / `FusedStream` impl |
| `src/eval/mod.rs` | `evaluate_with_env` → `async fn` |
| `src/eval/module.rs` | `load_module` / `load_import` → `async fn` |
| `src/eval/import.rs` | inherits async |
| `src/eval/forward.rs` | inherits async |
| `src/eval/reactor.rs` | 删死字段，`evaluate()` → `async fn` |
| `src/eval/reactor_types.rs` | 删 `ModuleCacheEntry` / `IoRecord` / `Warning` / `ReactorIO` trait+impls |
| `tests/reactor_test.rs` | 删除 `MockReactorIO` 相关测试，调整 `evaluate()` → `.await` |
