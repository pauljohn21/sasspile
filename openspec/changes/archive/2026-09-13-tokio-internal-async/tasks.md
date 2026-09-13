# Tasks: tokio-internal-async

## Phase 1: Dependency & Infrastructure (基础层)

- [x] **1.1 更新 Cargo.toml 依赖**
  - 移除 `tokio-stream = "0.1"` 和 `futures = "0.3"`
  - 添加 `tokio = { version = "1", features = ["rt-multi-thread", "fs"] }`
  - 全量 `cargo build` 确认无 break

- [x] **1.2 添加全局 tokio runtime (src/runtime.rs)**
  - 新建 `src/runtime.rs`: `static RUNTIME: OnceLock<Runtime>` + `block_on()` helper
  - `lib.rs` 注册 `mod runtime` (私有模块)

## Phase 2: Parser 回归 Iterator

- [x] **2.1 ParseStream 回归标准 Iterator**
  - 删除 `tokio_stream::Stream` impl
  - 删除 `futures::stream::FusedStream` impl
  - 添加 `impl Iterator for ParseStream<'_>`
  - 更新 `Parser::parse` 使用标准 `.collect()`
  - 删除 `Pin`, `Context`, `Poll`, `tokio_stream` 导入
  - 更新 `parse/nodes.rs` 文档注释

## Phase 3: Reactor 死代码清理

- [x] **3.1 删除 reactor_types.rs 死类型**
  - 删除 `ModuleCacheEntry` 结构体
  - 删除 `IoRecord` 结构体
  - 删除 `Warning` 结构体
  - 删除 `ReactorIO` trait
  - 删除 `DefaultReactorIO` 结构体 + impl
  - 删除 `MockReactorIO` 结构体 + impl

- [x] **3.2 删除 Reactor 死字段**
  - 删除 `env: Option<Env>` 字段
  - 删除 `modules: HashMap<PathBuf, ModuleCacheEntry>` 字段
  - 删除 `io_log: Vec<IoRecord>` 字段
  - 删除 `warnings: Vec<Warning>` 字段
  - 删除 `io: Arc<dyn ReactorIO>` 字段
  - 更新 `new()`, `from_file()`, 删除 `with_io()`, 全部状态转换 struct 构造
  - Reactor: 14 字段 → 9 字段

- [x] **3.3 修复 ReactorTrace::advance()**
  - `entered_at` 字段从 `self.entered_at` 获取而非 `Instant::now()`

- [x] **3.4 更新 lib.rs exports**
  - 移除 `ReactorIO` 从 `pub use` 导出列表
  - `ReactorSnapshot` 精简为 2 字段 (`stage` + `n_css_nodes`)

- [x] **3.5 更新 reactor_test.rs 测试**
  - 删除 `test_mocked_io_basic` 测试（依赖 MockReactorIO）
  - 替换为 `test_pipeline_result` 简单管线测试
  - Reactor::evaluate 保持同步 API

## Phase 4: 模块加载 IO 层 Tokio 化 (block_on 方案)

> **设计调整**: 采用 `block_on` 桥接方案而非全链路 async fn。
> 理由: SCSS 编译本质顺序管线, async 签名传播增加复杂度无实质回报。
> eval 管线保持纯 sync, 仅在 IO 层用 `tokio::fs::read_to_string` + `block_on`。

- [x] **4.1 IO 层迁移: load_module 使用 tokio::fs + block_on**
  - `std::fs::read_to_string` → `block_on(tokio::fs::read_to_string(path))`
  - 通过 `crate::runtime::block_on` 桥接

- [x] **4.2 IO 层迁移: load_import 使用 tokio::fs + block_on**
  - 同步替换 `std::fs::read_to_string` 调用

- [x] **4.3 eval_use 无需改 (同步调用 block_on IO)**
  - 内部 load_module 通过 runtime block_on 完成, eval_use 本身保持 sync

- [x] **4.4 eval_import / eval_forward 无需改 (sync)**
  - 内部各函数通过 block_on 桥接

- [x] **4.5 evaluate_with_env 保持 sync (IO 已 tokio)**
  - 函数签名不变, 模块加载通过 block_on 桥接

- [x] **4.6 Reactor::evaluate 同步化**
  - `pub fn evaluate(self)` (非 async)
  - `from_file()` 同样改用 `block_on(tokio::fs::read_to_string(path))`

## Phase 5: 代码风格优化

- [x] **5.1 消除 for + push 模式** — 已在 functional-cleanup (2026-09-04) 清理
- [x] **5.2 消除 if-else 链** — 已在 functional-cleanup 清理
- [x] **5.3 消除 match Err → return Err** — 已在 functional-cleanup 清理
- [x] **5.4 cargo clippy 验证** — 修复 `unwrap_err()` → `expect_err()` (clippy deny)

## Phase 6: 验证

- [x] **6.1 核心测试套件全绿**
  - `cargo test --test compile_test` — 46 通过
  - `cargo test --test reactor_test` — 14 通过
  - `cargo test --test stage_test` — 8 通过
  - `cargo test --test common_test` — 5 通过
  - `cargo test --test interp_test` — 15 通过

- [x] **6.2 模块系统测试全绿**
  - `cargo test --test bs_spec` — 15 通过
  - `cargo test --test ep_full` — 1 通过 (121/121)

- [x] **6.3 sass-spec 回归基线**
  - 7592/12133 = 62% — **零回归**

## 验证清单

```bash
cargo build                              # 零错误 ✓
cargo test --test compile_test           # 46 通过 ✓
cargo test --test reactor_test           # 14 通过 ✓
cargo test --test bs_spec -- --nocapture # 15 通过 ✓
cargo test --test ep_full -- --nocapture # 1 通过 ✓
SPEC_STORE_CMD=run cargo test --test spec_store -- --nocapture  # 无 regression ✓
```
