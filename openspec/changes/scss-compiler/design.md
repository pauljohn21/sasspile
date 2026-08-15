## Context

当前 Rust 生态中的 Sass 编译器都是 FFI 封装：
- `sass-rs` → LibSass C++ 绑定（已废弃）
- `grass` → 直接调用 Dart Sass 二进制

这些方案存在：部署困难（需要外部二进制）、启动慢、无法嵌入 Tokio 异步管道、无法热重载。

本项目用纯 Rust 从零实现，前提是**充分利用 Tokio 的并发特性**，以函数响应式风格设计编译管道。

## Hard Constraints

| 约束 | 说明 |
|------|------|
| **单文件 ≤ 500 行** | 所有源码文件的目标上限 |
| **单文件 ≤ 1000 行** | 绝对上限（测试文件可放宽至此），超限必须拆分 |
| **零 `println!`/`eprintln!`** | 全部使用 `tracing` 宏 |
| **tests/ 与 src/ 分离** | src/ 保持纯生产代码 |

## Goals / Non-Goals

**Goals:**
- 纯 Rust 实现，零外部依赖（除 tokio/tracing/futures）
- Tokio 异步管道：每个编译阶段是独立 Task，通过 channel 通信
- 函数响应式：不可变数据 + watch channel 实现自动更新
- 100% 兼容 sass-spec（除 CSS 4.0 颜色 462 个文件）
- 支持并行编译多个入口文件
- 支持流式输入/输出（`Stream<Item = Result<CssOutput>>`）

**Non-Goals:**
- 不实现 CSS Lint / 后处理
- 不实现 CSS 4.0 颜色特性（本期跳过）
- 不实现 JS API 兼容层（仅 Rust API）
- 不实现 CSS 自定义属性的运行时求值

## System Architecture

### Directory Structure with Line Budget

```
sasslipe/
├── Cargo.toml                         # 依赖声明
├── README.md
├── src/
│   ├── lib.rs                         # 公共 API ≤ 100 行
│   ├── main.rs                        # CLI 入口 ≤ 200 行
│   │
│   ├── source/                        # 源码位置与 span
│   │   ├── mod.rs                     # ≤ 50 行
│   │   ├── span.rs                    # SourceSpan ≤ 80 行
│   │   └── position.rs                # SourcePosition ≤ 60 行
│   │
│   ├── diagnostics/                   # 错误与诊断
│   │   ├── mod.rs                     # ≤ 50 行
│   │   ├── level.rs                   # Error/Warn/Info ≤ 50 行
│   │   ├── diagnostic.rs              # Diagnostic 结构 ≤ 120 行
│   │   ├── builder.rs                 # DiagnosticBuilder ≤ 100 行
│   │   └── renderer.rs                # 输出格式化 ≤ 150 行
│   │
│   ├── value/                         # 值系统（核心，拆分多文件）
│   │   ├── mod.rs                     # Value 枚举定义 ≤ 100 行
│   │   ├── number.rs                  # 数值与单位 ≤ 150 行
│   │   ├── color.rs                   # srgb 颜色 ≤ 150 行
│   │   ├── list.rs                    # List / Map / ArgList ≤ 150 行
│   │   ├── cmp.rs                     # 等值性判断 ≤ 120 行
│   │   ├── ops.rs                     # 算术/比较/逻辑运算 ≤ 200 行
│   │   ├── coerce.rs                  # 类型转换 ≤ 100 行
│   │   └── ser.rs                     # CSS 序列化 ≤ 150 行
│   │
│   ├── lexer/                         # 词法分析
│   │   ├── mod.rs                     # ≤ 50 行
│   │   ├── token.rs                   # Token 枚举定义 ≤ 150 行
│   │   ├── lexer.rs                   # 主词法分析器 ≤ 200 行
│   │   ├── interpolation.rs           # 插值处理 ≤ 150 行
│   │   └── indented.rs                # .sass 缩进语法 ≤ 150 行
│   │
│   ├── parser/                        # 语法分析
│   │   ├── mod.rs                     # ≤ 50 行
│   │   ├── ast.rs                     # AST 节点枚举 ≤ 200 行
│   │   ├── parser.rs                  # 主解析器 ≤ 250 行
│   │   ├── selectors.rs               # 选择器解析 ≤ 150 行
│   │   ├── atrules.rs                 # @指令解析 ≤ 200 行
│   │   └── declarations.rs            # 属性声明解析 ≤ 150 行
│   │
│   ├── semantic/                      # 语义分析
│   │   ├── mod.rs                     # ≤ 50 行
│   │   ├── symbol_table.rs            # 作用域栈 ≤ 150 行
│   │   ├── module.rs                  # @use/@forward 依赖图 ≤ 200 行
│   │   ├── extend.rs                  # @extend 验证 ≤ 100 行
│   │   └── definitions.rs             # mixin/function 注册 ≤ 120 行
│   │
│   ├── eval/                          # 表达式求值
│   │   ├── mod.rs                     # ≤ 50 行
│   │   ├── evaluator.rs               # 求值上下文 ≤ 150 行
│   │   ├── ops.rs                     # 内置运算符实现 ≤ 200 行
│   │   ├── functions.rs               # 函数调用逻辑 ≤ 150 行
│   │   └── collections.rs             # list/map 访问 ≤ 120 行
│   │
│   ├── builtin/                       # 内置模块
│   │   ├── mod.rs                     # 模块注册入口 ≤ 80 行
│   │   ├── sass_color.rs              # sass:color ≤ 250 行
│   │   ├── sass_math.rs               # sass:math ≤ 250 行
│   │   ├── sass_list.rs               # sass:list ≤ 200 行
│   │   ├── sass_map.rs                # sass:map ≤ 250 行
│   │   ├── sass_string.rs             # sass:string ≤ 200 行
│   │   └── sass_meta.rs               # sass:meta ≤ 200 行
│   │
│   ├── css/                           # CSS 生成
│   │   ├── mod.rs                     # ≤ 50 行
│   │   ├── ast.rs                     # CSS AST 定义 ≤ 150 行
│   │   ├── generator.rs               # 主生成器 ≤ 100 行
│   │   ├── rules.rs                   # 规则展开 ≤ 200 行
│   │   ├── atrules.rs                 # at-rules 输出 ≤ 150 行
│   │   ├── selectors.rs               # 选择器输出 ≤ 100 行
│   │   └── sourcemap.rs               # source map v3 ≤ 200 行
│   │
│   ├── incremental/                   # 增量编译与响应式
│   │   ├── mod.rs                     # ≤ 50 行
│   │   ├── env.rs                     # 响应式环境 watch channel ≤ 150 行
│   │   ├── depgraph.rs                # 依赖图 ≤ 200 行
│   │   ├── cache.rs                   # 基于 span 的缓存 ≤ 120 行
│   │   ├── propagate.rs               # 变更传播 ≤ 120 行
│   │   ├── debounce.rs                # 文件防抖器 ≤ 150 行
│   │   └── watcher.rs                 # fsnotify 监听 ≤ 100 行
│   │
│   └── pipeline/                      # Tokio 管道编排
│       ├── mod.rs                     # Pipeline 入口 ≤ 80 行
│       ├── stage.rs                   # PipelineStage trait ≤ 80 行
│       ├── orchestrator.rs            # 7-stage 协调整合 ≤ 200 行
│       ├── backpressure.rs            # bounded channel & 取消 ≤ 100 行
│       └── concurrent.rs              # 多文件并发编译 ≤ 150 行
│
└── tests/                             # 集成测试（目标 ≤ 800 行）
    ├── lib.rs                         # 测试入口
    ├── fixtures/                      # 测试 fixtures
    ├── lexer_spec.rs                  # 词法测试
    ├── parser_spec.rs                 # 语法测试
    ├── eval_spec.rs                   # 求值测试
    ├── css_spec.rs                    # CSS 输出测试
    ├── incremental_spec.rs            # 增量编译测试
    ├── sass_spec.rs                   # sass-spec 运行器
    └── hrx_loader.rs                  # HRX 文件读取
```

### 拆分原则

当任意 `.rs` 文件接近 500 行时：
1. **按职责切分**：将数据定义 / 算法实现 / 序列化 拆为不同文件
2. **按场景切分**：将主逻辑 / 错误处理 / 工具函数分离
3. **variant 分组**：枚举的不同 variant 实现分到独立文件（如 `atrules.rs` 中 `@mixin/@include` 一组、`@if/@else/@for` 一组）

### 关键 size 校验规则

- `mod.rs` 仅用于 `pub use` 重导出 + 简短文档，≤ 100 行
- 单个函数 ≤ 40 行；超出时提取子函数
- impl 块超 200 行时拆到 `*_ext.rs` 或 `*_impl.rs`
- 测试断言每个 `#[test]` ≤ 30 行；复杂场景提取 helper

## Decisions

### 1. 管道架构：Tokio spawn + mpsc channel

**决策**: 每个编译阶段作为独立 `tokio::spawn` 任务，通过 `mpsc::channel` 连接管道。

**替代方案考虑**:
- `rayon` 数据并行 → 不适合 IO 密集型 + 流式场景
- `async fn` 链式调用 → 无法利用多核并行
- `Generator` / `coroutine` → 不稳定特性

**选择理由**:
- `tokio::spawn` 充分利用多核
- `mpsc` 提供自然背压（channel 满时自动阻塞上游）
- 各阶段可独立测试、替换

### 2. 值系统：不可变数据结构 + Arc

**决策**: 所有 `Value` 类型实现 `Clone`，使用 `Arc<T>` 共享大数据。

**替代方案考虑**:
- `Rc<T>` → 非 Send，无法跨 Task
- 可变引用 `&mut` → 违反借用规则（多阶段并行）
- Copy-on-write (`Cow`) → 过度复杂

**选择理由**:
- `Arc` 是 Tokio 跨 Task 共享的标准方式
- 不可变性消除数据竞争
- 符合函数响应式核心原则

### 3. 模块系统：有向图 + 拓扑排序

**决策**: `@use`/`@import` 依赖构建有向图，Kahn 算法拓扑排序确定编译顺序。

**理由**:
- 支持循环依赖检测
- 入度为 0 的模块可并行编译
- 增量编译时只重新编译变更模块

### 4. 内置模块注册：async trait + 函数映射表

**决策**: 内置函数注册为 `HashMap<&str, Box<dyn SassFn>>`，trait 为 `async fn call(args: &[Value], env: &Env) -> Result<Value>`。

**理由**:
- 允许用户自定义函数
- `async` 支持 IO 密集操作（如文件读取）
- 函数名查找 O(1)

### 5. 错误处理：沿途收集 + 源码位置

**决策**: 错误不中断管道，收集所有错误后统一报告。每个 AST 节点附带 `SourceLocation`。

**理由**:
- 用户希望一次看到所有错误，而非逐个修复
- 源码位置对诊断至关重要

### 6. 文件防抖：事件合并 + 可配置静默窗口

**决策**: 使用 `tokio::select!` + `tokio::time::sleep` 实现基于时间的 debouncer。fsnotify 事件进入 mpsc channel，debouncer 在每轮静默期后批量 flush。

**关键参数**:
- `debounce_ms: u64` — 默认 200ms
- `CompileMode::Watch | CompileMode::Ci` — CI 模式下禁用防抖

**理由**:
- 编辑器保存通常触发 1-3 个事件（内容变更、元数据变更）
- 防抖避免单次保存触发多次无效编译
- 批量处理复用已编译模块缓存，比单次串行更高效
- 去重同一路径的多次事件，避免对同一文件重复编译

**Debounce 实现要点**:
```rust
// incremental/debounce.rs — ≤ 150 行
pub struct FileDebouncer {
    rx: mpsc::Receiver<FsEvent>,
    tx: mpsc::Sender<Vec<PathBuf>>,
    duration: Duration,
    buffer: HashMap<PathBuf, Instant>,
}
impl FileDebouncer {
    pub async fn run(mut self) { /* select + sleep 逻辑 */ }
}
```

### 7. 输出格式：CSS AST → Formatter

**决策**: 先产生中间 CSS AST，再通过 Formatter 产生最终文本。

**替代方案考虑**:
- 直接拼字符串 → 无法支持多种输出格式
- Display trait → 过于耦合

**选择理由**:
- 支持 `expanded`/`compressed`/`compact` 等多种格式
- CSS AST 可做后续优化（如去重、合并）

## Architecture Diagram

```
┌──────────────────────────────────────────────────────────────────────────┐
│                     sasslipe Pipeline                                    │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────┐    ┌────────┐    ┌─────────┐    ┌───────────┐             │
│  │  Watch   │───▶│ Debounce│───▶│  Lex    │───▶│  Parse    │             │
│  │ Service  │    │  200ms  │    │         │    │           │             │
│  └──────────┘    └────────┘    └─────────┘    └───────────┘             │
│       │                                │               │                │
│       │ (incremental/)                │    ┌──────────▼──────────┐     │
│       │                               └───▶│  Semantic Analysis  │     │
│       │                                    └──────────┬──────────┘     │
│       │                                               │                │
│       │              ┌──────────┐    ┌──────────┐     │                │
│       │              │  Module  │◀──│  Graph   │◀────┘                │
│       │              │ Resolver │    │ Analysis │                       │
│       │              └──────────┘    └──────────┘                       │
│       │                   │                                             │
│       │                   ▼                                             │
│       │              ┌──────────┐    ┌──────────┐                      │
│       └─────────────▶│ Evaluate │───▶│   CSS    │                      │
│   (watch channel     │  Engine  │    │   Gen    │                      │
│    propagation)      └──────────┘    └──────────┘                      │
│                              │               │                          │
│                              ▼               ▼                          │
│                       ┌──────────┐    ┌──────────┐                     │
│                       │  Cache   │───▶│  Format  │                     │
│                       │  Layer   │    │  Output  │                     │
│                       └──────────┘    └──────────┘                     │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘

Data Flow:
  FileEvents → Debounce(200ms, dedup) → Tokens → AST → Resolved → Eval → CSS AST → String
```

## Key Design Patterns

### Pattern 1: Pipeline Stage (trait)

```rust
// pipeline/stage.rs — ≤ 80 行
#[async_trait]
pub trait PipelineStage<Input, Output> {
    async fn process(&self, input: Input) -> Result<Output>;
}

pub struct Pipeline<A, B> {
    stages: Vec<Box<dyn PipelineStage<A, B>>>,
}
```

### Pattern 2: Watch-based Reactive State

```rust
// incremental/env.rs — ≤ 150 行
pub struct ReactiveEnv {
    vars: watch::Sender<Map<String, Value>>,
}

impl ReactiveEnv {
    pub fn subscribe(&self) -> watch::Receiver<Map<String, Value>> {
        self.vars.subscribe()
    }
    pub fn set_var(&self, name: &str, value: Value) {
        self.vars.send_modify(|map| { map.insert(name.into(), value); });
    }
}
```

### Pattern 3: Async Builtin Function

```rust
// builtin/sass_color.rs 等 — 每个模块 ≤ 250 行
#[async_trait]
pub trait SassFn: Send + Sync {
    async fn call(&self, args: &[Value], env: &Env) -> Result<Value>;
}

// 注册入口在 builtin/mod.rs ≤ 80 行
let mut registry: HashMap<String, Box<dyn SassFn>> = HashMap::new();
registry.insert("lighten".into(), Box::new(LightenFn));
```

### Pattern 4: Split-by-Responsibility Module Layout

每个子模块目录遵循统一拆分：
- `mod.rs` (≤ 100 行) — 仅 re-export + 简短注释
- `types.rs` 或 `<name>.rs` — 核心数据定义
- `<operation>.rs` — 操作实现（如 `ops.rs`、`ser.rs`、`coerce.rs`）
- `*_ext.rs` 或 `*_impl.rs` — 辅助实现（当主文件接近上限时）

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| 性能不及 Dart Sass（C++ 优化） | 使用 `moka` 缓存编译结果；并行编译多文件；JIT 缓存常用表达式 |
| 内存使用过高（不可变数据） | 使用 `Arc` 共享；对大文件启用流式处理 |
| sass-spec 边缘 case 不兼容 | 每日 CI 运行 sass-spec；自动标记新发现的不兼容 case |
| CSS 4.0 颜色接口不稳定 | 使用 `#[allow(unstable_features)]` 标注抽象层 |
| 异步管道调试困难 | 使用 `tracing` 为每个阶段添加 span；提供 `--trace` 标志 |
| 文件数过多增加编译时间 | 合理聚合相关内容到同一文件（但严守 500/1000 行限制） |

## Open Questions

1. **正则表达式引擎选择**: 是否需要 `regex` crate 做选择器解析？还是纯手写解析器？
2. **Bigint 支持**: Sass 任意精度数值是否需要 `num-bigint`？
3. **WASM 编译**: 是否需要在 WASM 目标下可用？（Tokio 在 WASM 下受限）
4. **notify crate vs 自研 poller**: 跨平台文件监听是否引入 `notify` 还是自己轮询？
